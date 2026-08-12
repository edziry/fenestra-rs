use fenestra_ui_runtime::prototype::SchedulerTick;
use fenestra_ui_spatial::prototype::{ReferenceRasterLimitsV2, SpatialViewportV2};

use super::super::super::spatial_v2::{SpatialPresentErrorKindV2, present_spatial_offer_v2};
use super::fixture::{
    LOGICAL_VIEWPORT, offer_at, reject, replace_rejected_offer, spatial_scheduler,
};
use super::support::{PortPlan, RecordingPort, surface};

#[test]
fn old_offered_frame_remains_readable_after_a_later_runtime_commit() {
    let mut scheduler = spatial_scheduler();
    let old_work = offer_at(&mut scheduler, LOGICAL_VIEWPORT, 10);
    let old_frame = old_work
        .paint_frame()
        .expect("old offer should carry spatial paint data");
    let old_generation = old_frame.generation();
    let old_viewport = old_frame.spatial().viewport();
    let old_shapes = slice_identity(old_frame.spatial().shapes());
    let old_paints = slice_identity(old_frame.spatial().resolved_paints());
    let old_raster = old_frame
        .spatial()
        .rasterize_reference(ReferenceRasterLimitsV2::new(4))
        .expect("old offer should rasterize");

    reject(&mut scheduler, &old_work, 12);
    let latest_viewport = SpatialViewportV2::new(5, 1);
    let latest_work = replace_rejected_offer(&mut scheduler, latest_viewport, 13);
    let latest_frame = latest_work
        .paint_frame()
        .expect("latest offer should carry spatial paint data");

    assert_eq!(old_frame.generation(), old_generation);
    assert_eq!(old_frame.spatial().viewport(), old_viewport);
    assert_eq!(slice_identity(old_frame.spatial().shapes()), old_shapes);
    assert_eq!(
        slice_identity(old_frame.spatial().resolved_paints()),
        old_paints
    );
    assert_eq!(old_raster.bytes(), super::fixture::REFERENCE_RGBA);
    assert_ne!(latest_frame.generation(), old_frame.generation());
    assert_eq!(latest_frame.spatial().viewport(), latest_viewport);
}

#[test]
fn viewport_mismatch_rejects_before_calling_the_private_port() {
    let mut scheduler = spatial_scheduler();
    let work = offer_at(&mut scheduler, LOGICAL_VIEWPORT, 10);
    let mut presenter = RecordingPort::new(PortPlan::Success);

    let error = present_spatial_offer_v2(
        &mut scheduler,
        &work,
        surface(4, 1, SpatialViewportV2::new(3, 1)),
        &mut presenter,
        SchedulerTick::new(12),
    )
    .expect_err("logical surface mismatch should reject the offer");
    assert_eq!(error.kind(), SpatialPresentErrorKindV2::ViewportMismatch);
    assert!(error.accepted_submission().is_none());
    assert_eq!(presenter.calls(), 0);
    assert_eq!(scheduler.stats().visual().items(), 1);
    assert_eq!(scheduler.stats().in_flight().items(), 0);
}

fn slice_identity<T>(value: &[T]) -> (*const T, usize) {
    (value.as_ptr(), value.len())
}
