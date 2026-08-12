use fenestra_ui_runtime::prototype::SchedulerTick;
use fenestra_ui_spatial::prototype::ReferenceRasterLimitsV2;

use super::fixture::{
    commit_root_property, headless_scheduler, ordinary_scheduler, reject, request_and_offer,
    spatial_scheduler, spatial_scheduler_with_source, take_offer,
};
use crate::spatial_support::VIEWPORT;
use crate::spatial_support::program::SourcePlan;

#[test]
fn ordinary_and_headless_offers_have_no_paint_frame() {
    let mut ordinary = ordinary_scheduler();
    commit_root_property(&mut ordinary, crate::support::WIDTH, 121, 10);
    let ordinary_offer = request_and_offer(&mut ordinary, 10);
    assert!(ordinary_offer.paint_frame().is_none());

    let mut headless = headless_scheduler();
    commit_root_property(&mut headless, crate::support::headless::WIDTH, 101, 20);
    let headless_offer = request_and_offer(&mut headless, 20);
    assert!(headless_offer.paint_frame().is_none());
}

#[test]
fn spatial_offer_has_a_frame_even_when_its_paint_table_is_empty() {
    let mut scheduler = spatial_scheduler();
    commit_root_property(&mut scheduler, crate::support::headless::WIDTH, 101, 10);
    let offer = request_and_offer(&mut scheduler, 10);
    let frame = offer
        .paint_frame()
        .expect("spatial offer should publish a paint frame");
    let direct = offer
        .snapshot()
        .spatial()
        .expect("spatial snapshot should exist")
        .snapshot()
        .paint_frame();

    assert_eq!(offer.accounted_bytes(), 40);
    assert_eq!(frame.generation(), offer.generation());
    assert_eq!(frame.spatial().viewport(), VIEWPORT);
    assert!(frame.spatial().paint_items().is_empty());
    assert_identity(frame.spatial().images(), direct.images());
    assert_identity(frame.spatial().resolved_paints(), direct.resolved_paints());
}

#[test]
fn old_and_noop_offers_retain_their_exact_generation_and_paint_data() {
    let mut scheduler = spatial_scheduler();
    commit_root_property(&mut scheduler, crate::support::headless::WIDTH, 101, 10);
    let old_offer = request_and_offer(&mut scheduler, 10);
    let old_frame = old_offer
        .paint_frame()
        .expect("old spatial offer should have a frame");
    let old_images = identity(old_frame.spatial().images());
    let old_bytes = identity(old_frame.spatial().images()[0].bytes());
    let old_raster = old_frame
        .spatial()
        .rasterize_reference(ReferenceRasterLimitsV2::new(6_300))
        .expect("old frame should rasterize");

    reject(&mut scheduler, &old_offer, 12);
    commit_root_property(&mut scheduler, crate::support::headless::WIDTH, 102, 13);
    let latest_offer = take_offer(&mut scheduler, 13);
    let latest_frame = latest_offer
        .paint_frame()
        .expect("latest spatial offer should have a frame");

    assert_eq!(old_frame.generation().get(), 1);
    assert_eq!(latest_frame.generation().get(), 2);
    assert_eq!(old_frame.spatial().viewport(), VIEWPORT);
    assert_eq!(old_frame.spatial().images()[0].bytes(), &[11, 22, 33, 255]);
    assert_identity_value(old_frame.spatial().images(), old_images);
    assert_identity_value(old_frame.spatial().images()[0].bytes(), old_bytes);
    assert_ne!(
        old_frame.spatial().images().as_ptr(),
        latest_frame.spatial().images().as_ptr()
    );
    let old_raster_after = old_frame
        .spatial()
        .rasterize_reference(ReferenceRasterLimitsV2::new(6_300))
        .expect("retained old frame should still rasterize");
    assert_raster_eq(&old_raster, &old_raster_after);

    let latest_images = identity(latest_frame.spatial().images());
    let latest_bytes = identity(latest_frame.spatial().images()[0].bytes());
    reject(&mut scheduler, &latest_offer, 14);
    let before_noop = scheduler.committed();
    let transaction = scheduler.begin_transaction();
    let receipt = scheduler
        .commit(transaction, SchedulerTick::new(15))
        .expect("true no-op should commit");
    let after_noop = scheduler.committed();

    assert!(receipt.is_empty());
    assert_eq!(receipt.generation(), latest_frame.generation());
    assert!(before_noop.shares_state_with(&after_noop));
    assert_identity_value(latest_frame.spatial().images(), latest_images);
    assert_identity_value(latest_frame.spatial().images()[0].bytes(), latest_bytes);

    let retry = take_offer(&mut scheduler, 15);
    let retry_frame = retry
        .paint_frame()
        .expect("retried spatial work should retain its paint frame");
    assert_eq!(retry.accounted_bytes(), 40);
    assert_eq!(retry_frame.generation(), latest_frame.generation());
    assert_identity_value(retry_frame.spatial().images(), latest_images);
    assert_identity_value(retry_frame.spatial().images()[0].bytes(), latest_bytes);
}

#[test]
fn failed_rebuild_preserves_the_offered_paint_frame_and_retry_identity() {
    let mut scheduler = spatial_scheduler_with_source(SourcePlan::MalformedCanonicalOnCall(3));
    commit_root_property(&mut scheduler, crate::support::headless::WIDTH, 101, 10);
    let offer = request_and_offer(&mut scheduler, 10);
    let frame = offer
        .paint_frame()
        .expect("spatial offer should retain a paint frame");
    let images = identity(frame.spatial().images());
    let image_bytes = identity(frame.spatial().images()[0].bytes());
    let raster = frame
        .spatial()
        .rasterize_reference(ReferenceRasterLimitsV2::new(6_300))
        .expect("offered frame should rasterize");
    reject(&mut scheduler, &offer, 12);
    let before = scheduler.committed();

    let root = before.root();
    let mut transaction = scheduler.begin_transaction();
    transaction
        .set_property(
            root,
            crate::support::headless::WIDTH,
            fenestra_ui_ir::prototype::PropertyValue::ScalarI32(102),
        )
        .expect("candidate rebuild property should stage");
    scheduler
        .commit(transaction, SchedulerTick::new(13))
        .expect_err("malformed candidate should reject the rebuild");

    let after = scheduler.committed();
    assert!(before.shares_state_with(&after));
    assert_eq!(frame.generation(), after.generation());
    assert_identity_value(frame.spatial().images(), images);
    assert_identity_value(frame.spatial().images()[0].bytes(), image_bytes);
    let raster_after = frame
        .spatial()
        .rasterize_reference(ReferenceRasterLimitsV2::new(6_300))
        .expect("retained frame should still rasterize");
    assert_raster_eq(&raster, &raster_after);

    let retry = take_offer(&mut scheduler, 13);
    let retry_frame = retry
        .paint_frame()
        .expect("failed rebuild should preserve retriable paint work");
    assert_eq!(retry_frame.generation(), frame.generation());
    assert_identity_value(retry_frame.spatial().images(), images);
    assert_identity_value(retry_frame.spatial().images()[0].bytes(), image_bytes);
}

fn identity<T>(slice: &[T]) -> (*const T, usize) {
    (slice.as_ptr(), slice.len())
}

fn assert_identity<T>(left: &[T], right: &[T]) {
    assert_identity_value(left, identity(right));
}

fn assert_identity_value<T>(actual: &[T], expected: (*const T, usize)) {
    assert_eq!(identity(actual), expected);
}

fn assert_raster_eq(
    left: &fenestra_ui_spatial::prototype::ReferenceRasterV2,
    right: &fenestra_ui_spatial::prototype::ReferenceRasterV2,
) {
    assert_eq!(left.width(), right.width());
    assert_eq!(left.height(), right.height());
    assert_eq!(left.stride(), right.stride());
    assert_eq!(left.bytes(), right.bytes());
}
