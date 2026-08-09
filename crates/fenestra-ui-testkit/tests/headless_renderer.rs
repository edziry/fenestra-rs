#[path = "headless/fixture_support.rs"]
mod fixture_support;
#[path = "headless/platform_support.rs"]
mod platform_support;

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{
    ControlAdmission, FrameWork, RendererEpoch, SchedulerAction, SchedulerTick, UiScheduler,
};
use fenestra_ui_testkit::prototype::{
    FakeControlDeliveryV1, FakeFrameReadyDeliveryV1, FakePlatformV1, FakeRendererCapacityV1,
    FakeRendererErrorKindV1, FakeRendererModeV1, FakeRendererOfferOutcomeV1, FakeRendererV1,
    HeadlessFixtureV1, HeadlessRendererErrorKindV1, HeadlessRendererErrorV1,
    SyntheticResourceUseV1, headless_frame_resource_v1,
};

use fixture_support::WIDTH;

fn tick(value: u64) -> SchedulerTick {
    SchedulerTick::new(value)
}

fn assert_error_type(error: &HeadlessRendererErrorV1) {
    assert_eq!(
        error.kind(),
        HeadlessRendererErrorKindV1::HeadlessUnavailable
    );
}

fn publish_width(scheduler: &mut UiScheduler) {
    let root = scheduler.committed().root();
    let mut transaction = scheduler.begin_transaction();
    transaction
        .set_property(root, WIDTH, PropertyValue::ScalarI32(99))
        .expect("fixture width should stage");
    scheduler
        .commit(transaction, tick(1))
        .expect("fixture width should publish");
}

fn take_offer(scheduler: &mut UiScheduler, platform: &mut FakePlatformV1) -> FrameWork {
    assert_eq!(
        scheduler
            .next_action(tick(1))
            .expect("publication should request a frame"),
        Some(SchedulerAction::RequestFrame)
    );
    assert_eq!(
        platform
            .frame_ready(scheduler, tick(1))
            .expect("frame-ready should be accepted"),
        FakeFrameReadyDeliveryV1::Accepted
    );
    let Some(SchedulerAction::OfferFrame(frame)) = scheduler
        .next_action(tick(1))
        .expect("frame-ready should produce an offer")
    else {
        panic!("headless renderer test should receive one frame");
    };
    frame
}

#[test]
fn headless_resource_is_generation_identified_and_composes_with_immediate_mode() {
    let fixture = HeadlessFixtureV1::build().expect("registered fixture should validate");
    let mut scheduler = platform_support::scheduler(&fixture);
    let mut platform = FakePlatformV1::new();
    let mut renderer = FakeRendererV1::new(
        RendererEpoch::new(0),
        FakeRendererCapacityV1::new(2, 192, 8),
    );
    publish_width(&mut scheduler);
    let frame = take_offer(&mut scheduler, &mut platform);
    assert_eq!(frame.id().get(), 0);
    assert_eq!(frame.generation().get(), 1);

    let resource = headless_frame_resource_v1(&frame)
        .expect("headless frame should expose one generation resource");
    assert_eq!(resource, SyntheticResourceUseV1::new(resource.id(), 64));
    assert_eq!(resource.id().get(), frame.generation().get());
    assert_eq!(resource.synthetic_bytes(), 64);

    let FakeRendererOfferOutcomeV1::Immediate {
        submission,
        completion: FakeControlDeliveryV1::Accepted(ControlAdmission::Accepted(sequence)),
    } = renderer
        .offer(
            &mut scheduler,
            frame,
            std::slice::from_ref(&resource),
            FakeRendererModeV1::Immediate,
            tick(1),
        )
        .expect("derived resource should compose with the existing renderer")
    else {
        panic!("immediate mode should accept and complete token zero");
    };
    assert_eq!(submission.epoch().get(), 0);
    assert_eq!(submission.token(), 0);
    assert_eq!(sequence.get(), 0);
    assert_eq!(renderer.stats().items(), 0);
    assert_eq!(renderer.stats().accounted_bytes(), 0);
    assert_eq!(scheduler.stats().controls().items(), 1);
    assert_eq!(scheduler.stats().in_flight().items(), 1);
    assert_eq!(
        scheduler
            .next_action(tick(1))
            .expect("immediate completion should process in scheduler order"),
        None
    );
    assert_eq!(scheduler.stats().controls().items(), 0);
    assert_eq!(scheduler.stats().in_flight().items(), 0);
}

#[test]
fn ordinary_frame_resource_failure_is_pure_and_leaves_the_offer_disposable() {
    let fixture = HeadlessFixtureV1::build().expect("registered fixture should validate");
    let mut scheduler = platform_support::ordinary_scheduler(&fixture);
    let mut platform = FakePlatformV1::new();
    let mut renderer = FakeRendererV1::new(
        RendererEpoch::new(0),
        FakeRendererCapacityV1::new(2, 192, 8),
    );
    publish_width(&mut scheduler);
    let frame = take_offer(&mut scheduler, &mut platform);
    let frame_id = frame.id();
    let scheduler_before = scheduler.stats();
    let renderer_before = renderer.stats();

    let error = headless_frame_resource_v1(&frame)
        .expect_err("ordinary frame should have no headless generation resource");
    assert_error_type(&error);
    assert_eq!(
        format!("{error:?}"),
        "HeadlessRendererErrorV1 { kind: HeadlessUnavailable }"
    );
    assert_eq!(scheduler.stats(), scheduler_before);
    assert_eq!(renderer.stats(), renderer_before);
    assert_eq!(
        renderer
            .offer(
                &mut scheduler,
                frame,
                &[],
                FakeRendererModeV1::Fail,
                tick(1),
            )
            .expect("borrowed preflight must leave the frame disposable"),
        FakeRendererOfferOutcomeV1::Rejected(frame_id)
    );
    let Some(SchedulerAction::OfferFrame(retry)) = scheduler
        .next_action(tick(1))
        .expect("rejected ordinary frame should remain recoverable")
    else {
        panic!("ordinary offer should retry with a fresh identity");
    };
    assert_ne!(retry.id(), frame_id);
}

#[test]
fn renderer_capacity_failure_rejects_the_derived_resource_without_ledger_growth() {
    let fixture = HeadlessFixtureV1::build().expect("registered fixture should validate");
    let mut scheduler = platform_support::scheduler(&fixture);
    let mut platform = FakePlatformV1::new();
    let mut renderer =
        FakeRendererV1::new(RendererEpoch::new(0), FakeRendererCapacityV1::new(2, 95, 8));
    publish_width(&mut scheduler);
    let frame = take_offer(&mut scheduler, &mut platform);
    let frame_id = frame.id();
    let generation = frame.generation();
    let snapshot = frame.snapshot().clone();
    let invalidation = frame.invalidation();
    let resource = headless_frame_resource_v1(&frame)
        .expect("headless frame should derive a 96-byte accounted resource");
    let scheduler_before = scheduler.stats();
    let renderer_before = renderer.stats();

    let error = renderer
        .offer(
            &mut scheduler,
            frame,
            std::slice::from_ref(&resource),
            FakeRendererModeV1::Late,
            tick(1),
        )
        .expect_err("95 bytes cannot hold one 64-byte resource plus its envelope");
    assert_eq!(error.kind(), FakeRendererErrorKindV1::CapacityExceeded);
    assert_eq!(scheduler.stats(), scheduler_before);
    assert_eq!(renderer.stats(), renderer_before);
    let Some(SchedulerAction::OfferFrame(retry)) = scheduler
        .next_action(tick(1))
        .expect("capacity rejection should restore the pending publication")
    else {
        panic!("capacity rejection should produce a fresh retry");
    };
    assert_ne!(retry.id(), frame_id);
    assert_eq!(retry.generation(), generation);
    assert!(retry.snapshot().shares_state_with(&snapshot));
    assert_eq!(retry.invalidation(), invalidation);
    assert_eq!(retry.earliest_tick(), tick(1));
    assert_eq!(retry.latest_tick(), tick(1));
}

#[test]
fn headless_renderer_error_vocabulary_is_closed() {
    fn discriminant(kind: HeadlessRendererErrorKindV1) -> u8 {
        match kind {
            HeadlessRendererErrorKindV1::HeadlessUnavailable => 0,
            HeadlessRendererErrorKindV1::IdentityMismatch => 1,
        }
    }

    assert_eq!(
        discriminant(HeadlessRendererErrorKindV1::HeadlessUnavailable),
        0
    );
    assert_eq!(
        discriminant(HeadlessRendererErrorKindV1::IdentityMismatch),
        1
    );
}
