#[path = "headless/fixture_support.rs"]
mod fixture_support;
#[path = "headless/platform_support.rs"]
mod support;

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{
    CallbackFinish, CommittedRuntimeSnapshot, CompletionWatermark, ControlAdmission, HeadlessPoint,
    HeadlessSurface, SchedulerAction, SchedulerErrorKind, SchedulerInput, SchedulerInputResult,
    SchedulerTick, TransactionErrorKind, UiScheduler,
};
use fenestra_ui_testkit::prototype::{
    FakeCallbackDepthV1, FakeFrameReadyDeliveryV1, FakePlatformV1, HeadlessFixtureV1,
    HeadlessPlatformErrorKindV1, HeadlessPlatformErrorV1, HeadlessPointerMutationV1,
    HeadlessPointerScriptV1, HeadlessPointerTargetV1,
};

use fixture_support::WIDTH;

fn assert_pending_input_blocks_headless_callback(
    error: HeadlessPlatformErrorV1,
    platform: &FakePlatformV1,
    scheduler: &UiScheduler,
    before: &CommittedRuntimeSnapshot,
) {
    assert_eq!(
        error.kind(),
        HeadlessPlatformErrorKindV1::Scheduler(SchedulerErrorKind::InputOutOfOrder)
    );
    assert_eq!(error.operation_index(), None);
    assert!(platform.has_pending_frame_ready());
    assert!(scheduler.committed().shares_state_with(before));
    assert_eq!(scheduler.stats().deferred().items(), 0);
}

#[test]
fn resize_delivery_publishes_once_and_identical_resize_is_a_true_no_op() {
    let fixture = HeadlessFixtureV1::build().expect("registered fixture should validate");
    let mut scheduler = support::scheduler(&fixture);
    let mut platform = FakePlatformV1::new();
    let initial = scheduler.committed();
    let resized_surface = HeadlessSurface::new(90, 70);

    let report = platform
        .run_headless_resize_callback(&mut scheduler, resized_surface, SchedulerTick::new(7))
        .expect("valid resize callback should defer one operation");
    assert_eq!(report.captured_generation(), initial.generation());
    assert_eq!(report.target(), HeadlessPointerTargetV1::None);
    assert_eq!(report.deepest_depth(), 1);
    assert!(report.shares_entry_snapshot());
    assert_eq!(
        report.finish(),
        CallbackFinish::Deferred {
            operation_count: 1,
            accounted_bytes: 80,
        }
    );
    assert!(scheduler.committed().shares_state_with(&initial));
    assert_eq!(scheduler.stats().deferred().items(), 1);
    assert_eq!(scheduler.stats().visual().items(), 0);

    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(7))
            .expect("resize publication should emit the sole request"),
        Some(SchedulerAction::RequestFrame)
    );
    let resized = scheduler.committed();
    assert_eq!(resized.generation().get(), 1);
    assert_eq!(
        resized
            .headless_projection()
            .expect("headless projection should remain available")
            .surface(),
        resized_surface
    );
    let after_resize_stats = scheduler.stats();
    assert_eq!(after_resize_stats.deferred().items(), 0);
    assert_eq!(after_resize_stats.visual().items(), 1);
    assert_eq!(after_resize_stats.visual().accounted_bytes(), 40);
    assert_eq!(
        after_resize_stats.visual().earliest_tick(),
        Some(SchedulerTick::new(7))
    );
    assert_eq!(
        after_resize_stats.visual().latest_tick(),
        Some(SchedulerTick::new(7))
    );

    let repeated = platform
        .run_headless_resize_callback(&mut scheduler, resized_surface, SchedulerTick::new(8))
        .expect("identical resize should enter the ordinary deferred path");
    assert_eq!(repeated.captured_generation(), resized.generation());
    assert_eq!(repeated.target(), HeadlessPointerTargetV1::None);
    assert_eq!(
        repeated.finish(),
        CallbackFinish::Deferred {
            operation_count: 1,
            accounted_bytes: 80,
        }
    );
    assert!(scheduler.committed().shares_state_with(&resized));
    assert_eq!(scheduler.stats().visual(), after_resize_stats.visual());
    assert_eq!(scheduler.stats().deferred().items(), 1);

    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(8))
            .expect("identical resize should validate as one no-op"),
        None
    );
    assert!(scheduler.committed().shares_state_with(&resized));
    assert_eq!(scheduler.committed().generation().get(), 1);
    assert_eq!(scheduler.stats(), after_resize_stats);
}

#[test]
fn ordinary_resize_is_deferred_then_rejected_without_publication() {
    let fixture = HeadlessFixtureV1::build().expect("registered fixture should validate");
    let mut scheduler = support::ordinary_scheduler(&fixture);
    let mut platform = FakePlatformV1::new();
    let before = scheduler.committed();

    let report = platform
        .run_headless_resize_callback(
            &mut scheduler,
            HeadlessSurface::new(90, 70),
            SchedulerTick::new(1),
        )
        .expect("headless availability is validated only at deferred commit");
    assert_eq!(report.captured_generation(), before.generation());
    assert_eq!(
        report.finish(),
        CallbackFinish::Deferred {
            operation_count: 1,
            accounted_bytes: 80,
        }
    );
    assert!(scheduler.committed().shares_state_with(&before));

    let error = scheduler
        .next_action(SchedulerTick::new(2))
        .expect_err("ordinary runtime must reject the deferred headless resize");
    assert_eq!(
        error.kind(),
        SchedulerErrorKind::Transaction(TransactionErrorKind::HeadlessUnavailable)
    );
    assert_eq!(error.operation_index(), Some(0));
    assert!(scheduler.committed().shares_state_with(&before));
    assert_eq!(scheduler.stats().deferred().items(), 0);
    assert_eq!(scheduler.stats().visual().items(), 0);
}

#[test]
fn pointer_and_resize_do_not_expand_the_closed_scheduler_input_protocol() {
    assert_eq!(
        support::scheduler_input_discriminant(SchedulerInput::FrameReady),
        0
    );
}

#[test]
fn retained_frame_ready_blocks_headless_delivery_on_the_same_platform_owner() {
    let fixture = HeadlessFixtureV1::build().expect("registered fixture should validate");
    let mut scheduler = support::scheduler(&fixture);
    let mut platform = FakePlatformV1::new();
    let root = scheduler.committed().root();
    let capture = platform
        .capture_headless_pointer(&scheduler, &fixture, HeadlessPoint::new(5, 5))
        .expect("initial control target should be capturable");

    let mut first = scheduler.begin_transaction();
    first
        .set_property(root, WIDTH, PropertyValue::ScalarI32(99))
        .expect("first width should stage");
    scheduler
        .commit(first, SchedulerTick::new(1))
        .expect("first width should publish");
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(1))
            .expect("first request should advance"),
        Some(SchedulerAction::RequestFrame)
    );
    assert_eq!(
        platform
            .frame_ready(&mut scheduler, SchedulerTick::new(1))
            .expect("first frame-ready should be accepted"),
        FakeFrameReadyDeliveryV1::Accepted
    );
    let Some(SchedulerAction::OfferFrame(frame)) = scheduler
        .next_action(SchedulerTick::new(1))
        .expect("first offer should advance")
    else {
        panic!("first frame-ready should produce one offer");
    };
    let SchedulerInputResult::FrameAccepted(submission) = scheduler
        .process_input(
            SchedulerInput::AcceptFrame(frame.id()),
            SchedulerTick::new(1),
        )
        .expect("first offer should enter the in-flight lane")
    else {
        panic!("offer acceptance should return one submission");
    };

    let mut second = scheduler.begin_transaction();
    second
        .set_property(root, WIDTH, PropertyValue::ScalarI32(98))
        .expect("second width should stage");
    scheduler
        .commit(second, SchedulerTick::new(2))
        .expect("second width should publish");
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(2))
            .expect("successor request should advance"),
        Some(SchedulerAction::RequestFrame)
    );
    let SchedulerInputResult::Control(ControlAdmission::Accepted(_)) = scheduler
        .process_input(
            SchedulerInput::Complete(CompletionWatermark::from_submission(submission)),
            SchedulerTick::new(2),
        )
        .expect("completion should enter the control lane")
    else {
        panic!("completion should receive one control identity");
    };
    assert_eq!(
        platform
            .frame_ready(&mut scheduler, SchedulerTick::new(2))
            .expect("control pressure should retain frame-ready"),
        FakeFrameReadyDeliveryV1::Retained(SchedulerErrorKind::ControlPending)
    );
    assert!(platform.has_pending_frame_ready());
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(2))
            .expect("completion control should drain first"),
        None
    );
    assert_eq!(scheduler.stats().controls().items(), 0);
    assert_eq!(scheduler.stats().in_flight().items(), 0);
    assert!(platform.has_pending_frame_ready());
    let before_blocked_resize = scheduler.committed();

    let pointer_error = platform
        .run_headless_pointer_callback(
            &mut scheduler,
            &fixture,
            HeadlessPointerScriptV1::new(
                HeadlessPoint::new(5, 5),
                FakeCallbackDepthV1::Outer,
                None,
            ),
            SchedulerTick::new(2),
        )
        .expect_err("pointer callback must not overtake retained platform input");
    assert_pending_input_blocks_headless_callback(
        pointer_error,
        &platform,
        &scheduler,
        &before_blocked_resize,
    );

    let captured_error = platform
        .run_headless_captured_callback(
            &mut scheduler,
            &capture,
            FakeCallbackDepthV1::Outer,
            HeadlessPointerMutationV1::new(WIDTH, PropertyValue::ScalarI32(97)),
            SchedulerTick::new(2),
        )
        .expect_err("captured callback must not overtake retained platform input");
    assert_pending_input_blocks_headless_callback(
        captured_error,
        &platform,
        &scheduler,
        &before_blocked_resize,
    );

    let resize_error = platform
        .run_headless_resize_callback(
            &mut scheduler,
            HeadlessSurface::new(90, 70),
            SchedulerTick::new(2),
        )
        .expect_err("headless delivery must not overtake retained platform input");
    assert_pending_input_blocks_headless_callback(
        resize_error,
        &platform,
        &scheduler,
        &before_blocked_resize,
    );

    assert_eq!(
        platform
            .retry_frame_ready(&mut scheduler, SchedulerTick::new(2))
            .expect("the same platform should deliver its retained observation"),
        FakeFrameReadyDeliveryV1::Accepted
    );
    assert!(!platform.has_pending_frame_ready());
    assert!(matches!(
        scheduler
            .next_action(SchedulerTick::new(2))
            .expect("retried frame-ready should make the successor offer eligible"),
        Some(SchedulerAction::OfferFrame(_))
    ));
}
