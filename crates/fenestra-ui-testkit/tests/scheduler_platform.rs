#[path = "scheduler_renderer/support.rs"]
mod support;

use fenestra_ui_ir::prototype::{PropertyId, PropertyValue};
use fenestra_ui_runtime::prototype::{
    CallbackFinish, CompletionWatermark, ControlAdmission, RendererEpoch, SchedulerAction,
    SchedulerErrorKind, SchedulerInput, SchedulerInputResult, SchedulerState, SchedulerTick,
};
use fenestra_ui_testkit::prototype::{
    FakeCallbackDepthV1, FakeCallbackMutationV1, FakeCallbackScriptV1, FakeFrameReadyDeliveryV1,
    FakePlatformErrorKindV1, FakePlatformV1,
};

use support::scheduler;

const WIDTH: PropertyId = PropertyId::new(0);

#[test]
fn outer_query_is_bounded_without_publication() {
    let mut scheduler = scheduler();
    let mut platform = FakePlatformV1::new();
    let before = scheduler.committed();

    let early = platform
        .frame_ready(&mut scheduler, SchedulerTick::new(0))
        .expect_err("frame-ready before a request should remain an input-order error");
    assert_eq!(
        early.kind(),
        FakePlatformErrorKindV1::Scheduler(SchedulerErrorKind::InputOutOfOrder)
    );
    assert!(!platform.has_pending_frame_ready());

    let report = platform
        .run_callback(
            &mut scheduler,
            FakeCallbackScriptV1::new(FakeCallbackDepthV1::Outer, None, false),
            SchedulerTick::new(1),
        )
        .expect("outer query-only callback should finish");
    assert_eq!(report.captured_generation(), before.generation());
    assert_eq!(report.deepest_depth(), 1);
    assert!(report.shares_entry_snapshot());
    assert_eq!(report.finish(), CallbackFinish::NoChanges);
    assert_eq!(scheduler.stats().deferred().items(), 0);
    assert!(scheduler.committed().shares_state_with(&before));
}

#[test]
fn nested_callback_uses_one_captured_snapshot_and_publishes_on_a_later_turn() {
    let mut scheduler = scheduler();
    let mut platform = FakePlatformV1::new();
    let before = scheduler.committed();
    let root = before.root();

    let report = platform
        .run_callback(
            &mut scheduler,
            FakeCallbackScriptV1::new(
                FakeCallbackDepthV1::Grandchild,
                Some(FakeCallbackMutationV1::new(
                    root,
                    WIDTH,
                    PropertyValue::ScalarI32(130),
                )),
                false,
            ),
            SchedulerTick::new(10),
        )
        .expect("nested callback script should be staged");

    assert_eq!(report.captured_generation(), before.generation());
    assert_eq!(report.deepest_depth(), 3);
    assert!(report.shares_entry_snapshot());
    assert_eq!(
        report.finish(),
        CallbackFinish::Deferred {
            operation_count: 1,
            accounted_bytes: 80,
        }
    );
    assert!(scheduler.committed().shares_state_with(&before));
    assert_eq!(
        scheduler.committed().property(root, WIDTH),
        Some(&PropertyValue::ScalarI32(120))
    );
    assert_eq!(scheduler.stats().deferred().items(), 1);

    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(11))
            .expect("a later scheduler turn should publish the mutation"),
        Some(SchedulerAction::RequestFrame)
    );
    assert_eq!(scheduler.committed().generation().get(), 1);
    assert_eq!(
        scheduler.committed().property(root, WIDTH),
        Some(&PropertyValue::ScalarI32(130))
    );

    assert_eq!(
        platform
            .frame_ready(&mut scheduler, SchedulerTick::new(11))
            .expect("fake platform should acknowledge the frame request"),
        FakeFrameReadyDeliveryV1::Accepted
    );
    assert!(matches!(
        scheduler
            .next_action(SchedulerTick::new(11))
            .expect("frame-ready should make one offer eligible"),
        Some(SchedulerAction::OfferFrame(_))
    ));
}

#[test]
fn nested_shutdown_wins_over_staged_mutation_and_latches_once() {
    let mut scheduler = scheduler();
    let mut platform = FakePlatformV1::new();
    let before = scheduler.committed();
    let root = before.root();

    let report = platform
        .run_callback(
            &mut scheduler,
            FakeCallbackScriptV1::new(
                FakeCallbackDepthV1::Nested,
                Some(FakeCallbackMutationV1::new(
                    root,
                    WIDTH,
                    PropertyValue::ScalarI32(140),
                )),
                true,
            ),
            SchedulerTick::new(20),
        )
        .expect("shutdown callback script should finish");

    assert_eq!(report.captured_generation(), before.generation());
    assert_eq!(report.deepest_depth(), 2);
    assert!(report.shares_entry_snapshot());
    assert_eq!(report.finish(), CallbackFinish::ShutdownRequested);
    assert_eq!(scheduler.state(), SchedulerState::ShutdownQueued);
    assert_eq!(scheduler.stats().controls().items(), 1);
    assert_eq!(scheduler.stats().controls().accounted_bytes(), 32);
    assert_eq!(scheduler.stats().deferred().items(), 0);
    assert!(scheduler.committed().shares_state_with(&before));
    assert_eq!(
        scheduler.committed().property(root, WIDTH),
        Some(&PropertyValue::ScalarI32(120))
    );

    let duplicate = platform
        .run_callback(
            &mut scheduler,
            FakeCallbackScriptV1::new(FakeCallbackDepthV1::Outer, None, true),
            SchedulerTick::new(20),
        )
        .expect_err("the accepted shutdown latch should block later callbacks");
    assert_eq!(
        duplicate.kind(),
        FakePlatformErrorKindV1::Scheduler(SchedulerErrorKind::ControlPending)
    );
    assert_eq!(scheduler.stats().controls().items(), 1);
}

#[test]
fn frame_ready_is_retained_until_an_accepted_control_is_processed() {
    let mut scheduler = scheduler();
    let mut platform = FakePlatformV1::new();

    request_width(&mut scheduler, 130, 1);
    assert_eq!(
        platform
            .frame_ready(&mut scheduler, SchedulerTick::new(1))
            .expect("the first frame-ready input should be accepted"),
        FakeFrameReadyDeliveryV1::Accepted
    );
    let Some(SchedulerAction::OfferFrame(first)) = scheduler
        .next_action(SchedulerTick::new(1))
        .expect("the first offer turn should advance")
    else {
        panic!("the first frame-ready input should make an offer eligible");
    };
    let SchedulerInputResult::FrameAccepted(submission) = scheduler
        .process_input(
            SchedulerInput::AcceptFrame(first.id()),
            SchedulerTick::new(1),
        )
        .expect("the first offer should be accepted")
    else {
        panic!("offer acceptance should return a submission identity");
    };

    request_width(&mut scheduler, 140, 2);
    let SchedulerInputResult::Control(ControlAdmission::Accepted(_)) = scheduler
        .process_input(
            SchedulerInput::Complete(CompletionWatermark::from_submission(submission)),
            SchedulerTick::new(2),
        )
        .expect("completion should enter the ordered control lane")
    else {
        panic!("completion should receive one control identity");
    };

    assert_eq!(
        platform
            .frame_ready(&mut scheduler, SchedulerTick::new(2))
            .expect("control backpressure should retain frame-ready"),
        FakeFrameReadyDeliveryV1::Retained(SchedulerErrorKind::ControlPending)
    );
    assert!(platform.has_pending_frame_ready());
    assert_eq!(scheduler.stats().visual().items(), 1);

    let callback = platform
        .run_callback(
            &mut scheduler,
            FakeCallbackScriptV1::new(FakeCallbackDepthV1::Outer, None, false),
            SchedulerTick::new(2),
        )
        .expect_err("a callback must not overtake retained platform input");
    assert_eq!(
        callback.kind(),
        FakePlatformErrorKindV1::Scheduler(SchedulerErrorKind::InputOutOfOrder)
    );
    assert!(platform.has_pending_frame_ready());

    let second = platform
        .frame_ready(&mut scheduler, SchedulerTick::new(2))
        .expect_err("a second input must not overwrite retained frame-ready");
    assert_eq!(
        second.kind(),
        FakePlatformErrorKindV1::Scheduler(SchedulerErrorKind::InputOutOfOrder)
    );
    assert!(platform.has_pending_frame_ready());

    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(2))
            .expect("the earlier completion should be processed first"),
        None
    );
    assert_eq!(scheduler.stats().in_flight().items(), 0);
    assert_eq!(
        platform
            .retry_frame_ready(&mut scheduler, SchedulerTick::new(2))
            .expect("the retained input should enter the opened scheduler turn"),
        FakeFrameReadyDeliveryV1::Accepted
    );
    assert!(!platform.has_pending_frame_ready());
    assert!(matches!(
        scheduler
            .next_action(SchedulerTick::new(2))
            .expect("retried frame-ready should make one offer eligible"),
        Some(SchedulerAction::OfferFrame(_))
    ));
}

#[test]
fn retained_frame_ready_is_canceled_when_renderer_loss_removes_visual_work() {
    let mut scheduler = scheduler();
    let mut platform = FakePlatformV1::new();
    request_width(&mut scheduler, 130, 1);
    let SchedulerInputResult::Control(ControlAdmission::Accepted(_)) = scheduler
        .process_input(
            SchedulerInput::RendererLost(RendererEpoch::new(0)),
            SchedulerTick::new(1),
        )
        .expect("renderer loss should enter the control lane")
    else {
        panic!("renderer loss should receive one control identity");
    };

    assert_eq!(
        platform
            .frame_ready(&mut scheduler, SchedulerTick::new(1))
            .expect("loss control should temporarily block frame-ready"),
        FakeFrameReadyDeliveryV1::Retained(SchedulerErrorKind::ControlPending)
    );
    assert!(platform.has_pending_frame_ready());

    let regression = platform
        .retry_frame_ready(&mut scheduler, SchedulerTick::new(0))
        .expect_err("tick regression should precede terminal cancellation");
    assert_eq!(
        regression.kind(),
        FakePlatformErrorKindV1::Scheduler(SchedulerErrorKind::TickRegression)
    );
    assert!(platform.has_pending_frame_ready());
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(1))
            .expect("loss control should be processed"),
        None
    );
    assert_eq!(scheduler.state(), SchedulerState::Faulted);
    assert_eq!(scheduler.stats().visual().items(), 0);

    assert_eq!(
        platform
            .retry_frame_ready(&mut scheduler, SchedulerTick::new(1))
            .expect("terminal loss should cancel the retained platform input"),
        FakeFrameReadyDeliveryV1::Canceled
    );
    assert!(!platform.has_pending_frame_ready());
    let error = platform
        .retry_frame_ready(&mut scheduler, SchedulerTick::new(1))
        .expect_err("a canceled frame-ready input cannot be retried twice");
    assert_eq!(
        error.kind(),
        FakePlatformErrorKindV1::Scheduler(SchedulerErrorKind::InputOutOfOrder)
    );
}

fn request_width(
    scheduler: &mut fenestra_ui_runtime::prototype::UiScheduler,
    width: i32,
    tick: u64,
) {
    let root = scheduler.committed().root();
    let mut transaction = scheduler.begin_transaction();
    transaction
        .set_property(root, WIDTH, PropertyValue::ScalarI32(width))
        .expect("property write should stage");
    scheduler
        .commit(transaction, SchedulerTick::new(tick))
        .expect("property write should commit");
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(tick))
            .expect("frame request turn should advance"),
        Some(SchedulerAction::RequestFrame)
    );
}
