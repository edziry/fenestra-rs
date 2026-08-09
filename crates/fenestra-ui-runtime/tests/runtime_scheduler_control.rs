mod support;

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{
    CallbackFinish, CompletionWatermark, ControlAdmission, ControlSequence, FrameId, FrameWork,
    QueueCapacity, RendererEpoch, SchedulerAction, SchedulerCapacity, SchedulerErrorKind,
    SchedulerInput, SchedulerInputResult, SchedulerLane, SchedulerState, SchedulerTick,
    SubmissionId, UiRuntime, UiScheduler,
};

use support::{WIDTH, capacity, construction};

fn scheduler(controls: QueueCapacity) -> UiScheduler {
    let runtime = UiRuntime::new(construction(), capacity()).expect("runtime should initialize");
    let capacity = SchedulerCapacity::new(
        QueueCapacity::new(1, 80, 8),
        controls,
        QueueCapacity::new(1, 40, 8),
        QueueCapacity::new(2, 80, 8),
    );
    UiScheduler::new(runtime, capacity).expect("scheduler capacity should be valid")
}

fn process(scheduler: &mut UiScheduler, input: SchedulerInput, tick: u64) -> SchedulerInputResult {
    scheduler
        .process_input(input, SchedulerTick::new(tick))
        .expect("scheduler input should be accepted")
}

fn accepted(result: SchedulerInputResult) -> ControlSequence {
    let SchedulerInputResult::Control(ControlAdmission::Accepted(sequence)) = result else {
        panic!("control should receive a fresh acceptance sequence");
    };
    sequence
}

fn already_accepted(result: SchedulerInputResult) -> ControlSequence {
    let SchedulerInputResult::Control(ControlAdmission::AlreadyAccepted(sequence)) = result else {
        panic!("duplicate control should retain its acceptance sequence");
    };
    sequence
}

fn commit_width(scheduler: &mut UiScheduler, width: i32, tick: u64) {
    let root = scheduler.committed().root();
    let mut transaction = scheduler.begin_transaction();
    transaction
        .set_property(root, WIDTH, PropertyValue::ScalarI32(width))
        .expect("property write should stage");
    scheduler
        .commit(transaction, SchedulerTick::new(tick))
        .expect("property write should commit");
}

fn take_offer(scheduler: &mut UiScheduler, tick: u64) -> FrameWork {
    let action = scheduler
        .next_action(SchedulerTick::new(tick))
        .expect("offer turn should advance");
    let Some(SchedulerAction::OfferFrame(work)) = action else {
        panic!("one frame offer should be ready");
    };
    work
}

fn accept_offer(scheduler: &mut UiScheduler, frame: FrameId, tick: u64) -> SubmissionId {
    let SchedulerInputResult::FrameAccepted(submission) =
        process(scheduler, SchedulerInput::AcceptFrame(frame), tick)
    else {
        panic!("offer acceptance should return a submission identity");
    };
    submission
}

fn submit_width(scheduler: &mut UiScheduler, width: i32, tick: u64) -> SubmissionId {
    commit_width(scheduler, width, tick);
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(tick))
            .expect("frame request turn should advance"),
        Some(SchedulerAction::RequestFrame)
    );
    assert_eq!(
        process(scheduler, SchedulerInput::FrameReady, tick + 1),
        SchedulerInputResult::FrameReady
    );
    let offer = take_offer(scheduler, tick + 1);
    accept_offer(scheduler, offer.id(), tick + 2)
}

#[test]
fn shutdown_reserve_stays_inside_control_capacity_and_survives_ordinary_pressure() {
    for controls in [QueueCapacity::new(2, 128, 8), QueueCapacity::new(4, 64, 8)] {
        let mut constrained = scheduler(controls);
        let submission = submit_width(&mut constrained, 130, 1);
        let completion = accepted(process(
            &mut constrained,
            SchedulerInput::Complete(CompletionWatermark::from_submission(submission)),
            4,
        ));
        assert_eq!(completion.get(), 0);

        let error = constrained
            .process_input(
                SchedulerInput::RendererLost(RendererEpoch::new(0)),
                SchedulerTick::new(4),
            )
            .expect_err("ordinary controls must not consume the shutdown reserve");
        assert_eq!(
            error.kind(),
            SchedulerErrorKind::CapacityExceeded(SchedulerLane::Controls)
        );
        let shutdown = accepted(process(
            &mut constrained,
            SchedulerInput::RequestShutdown,
            4,
        ));
        assert_eq!(shutdown.get(), 1);
        assert_eq!(constrained.stats().controls().items(), 2);
        assert_eq!(constrained.stats().controls().accounted_bytes(), 64);
    }

    let mut exact = scheduler(QueueCapacity::new(4, 128, 8));
    let first = submit_width(&mut exact, 130, 2);
    let second = submit_width(&mut exact, 140, 4);
    let first_completion = accepted(process(
        &mut exact,
        SchedulerInput::Complete(CompletionWatermark::from_submission(first)),
        7,
    ));
    assert_eq!(first_completion.get(), 0);
    assert_eq!(exact.stats().in_flight().items(), 2);
    assert_eq!(
        already_accepted(process(
            &mut exact,
            SchedulerInput::Complete(CompletionWatermark::from_submission(first)),
            7,
        )),
        first_completion
    );
    assert_eq!(exact.stats().controls().items(), 1);

    let second_completion = accepted(process(
        &mut exact,
        SchedulerInput::Complete(CompletionWatermark::from_submission(second)),
        7,
    ));
    assert_eq!(second_completion.get(), 1);
    let loss = accepted(process(
        &mut exact,
        SchedulerInput::RendererLost(RendererEpoch::new(0)),
        7,
    ));
    assert_eq!(loss.get(), 2);
    assert_eq!(
        already_accepted(process(
            &mut exact,
            SchedulerInput::RendererLost(RendererEpoch::new(0)),
            7,
        )),
        loss
    );

    let shutdown = accepted(process(&mut exact, SchedulerInput::RequestShutdown, 7));
    assert_eq!(shutdown.get(), 3);
    assert_eq!(
        already_accepted(process(&mut exact, SchedulerInput::RequestShutdown, 7)),
        shutdown
    );
    assert_eq!(exact.stats().controls().items(), 4);
    assert_eq!(exact.stats().controls().accounted_bytes(), 128);
    assert_eq!(exact.stats().in_flight().items(), 2);
    assert!(exact.stats().controls().items() <= exact.capacity().controls().max_items());
    assert!(exact.stats().controls().accounted_bytes() <= exact.capacity().controls().max_bytes());

    assert_eq!(
        exact
            .next_action(SchedulerTick::new(7))
            .expect("first completion should process first"),
        None
    );
    assert_eq!(exact.stats().in_flight().items(), 1);
    assert_eq!(exact.stats().controls().items(), 3);
    assert_eq!(
        exact
            .next_action(SchedulerTick::new(7))
            .expect("second completion should process second"),
        None
    );
    assert_eq!(exact.stats().in_flight().items(), 0);
    assert_eq!(exact.stats().controls().items(), 2);
    assert_eq!(
        exact
            .next_action(SchedulerTick::new(7))
            .expect("loss should process before shutdown"),
        None
    );
    assert_eq!(exact.state(), SchedulerState::Faulted);
    assert_eq!(exact.stats().controls().items(), 1);
    assert_eq!(
        exact
            .next_action(SchedulerTick::new(7))
            .expect("shutdown should be the final ordered control"),
        Some(SchedulerAction::StopRenderer(shutdown))
    );
    assert_eq!(exact.state(), SchedulerState::Stopped);
    assert_eq!(exact.stats().controls().items(), 0);
}

#[test]
fn accepted_controls_precede_ordinary_and_deferred_publication() {
    let mut scheduler = scheduler(QueueCapacity::new(2, 64, 8));
    let root = scheduler.committed().root();
    let mut callback = scheduler
        .begin_callback(SchedulerTick::new(10))
        .expect("callback should begin");
    callback
        .transaction()
        .set_property(root, WIDTH, PropertyValue::ScalarI32(130))
        .expect("callback property write should stage");
    assert_eq!(
        callback.finish().expect("callback should be deferred"),
        CallbackFinish::Deferred {
            operation_count: 1,
            accounted_bytes: 80,
        }
    );

    let loss = accepted(process(
        &mut scheduler,
        SchedulerInput::RendererLost(RendererEpoch::new(0)),
        11,
    ));
    let shutdown = accepted(process(&mut scheduler, SchedulerInput::RequestShutdown, 11));
    assert_eq!((loss.get(), shutdown.get()), (0, 1));
    assert_eq!(scheduler.stats().deferred().items(), 1);

    let mut ordinary = scheduler.begin_transaction();
    ordinary
        .set_property(root, WIDTH, PropertyValue::ScalarI32(140))
        .expect("ordinary property write should stage");
    let error = scheduler
        .commit(ordinary, SchedulerTick::new(11))
        .expect_err("accepted control must block ordinary publication");
    assert_eq!(error.kind(), SchedulerErrorKind::ControlPending);
    assert_eq!(scheduler.committed().generation().get(), 0);

    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(11))
            .expect("loss should be processed first"),
        None
    );
    assert_eq!(scheduler.state(), SchedulerState::Faulted);
    assert_eq!(scheduler.stats().controls().items(), 1);
    assert_eq!(scheduler.stats().deferred().items(), 0);
    assert_eq!(scheduler.committed().generation().get(), 0);

    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(11))
            .expect("shutdown should remain deliverable after loss"),
        Some(SchedulerAction::StopRenderer(shutdown))
    );
    assert_eq!(scheduler.state(), SchedulerState::Stopped);
    assert_eq!(scheduler.stats().controls().items(), 0);
    assert_eq!(scheduler.committed().generation().get(), 0);
}

#[test]
fn shutdown_cancels_visual_work_only_when_its_stop_action_is_delivered() {
    let mut scheduler = scheduler(QueueCapacity::new(2, 64, 8));
    commit_width(&mut scheduler, 130, 10);
    assert_eq!(scheduler.stats().visual().items(), 1);

    let shutdown = accepted(process(&mut scheduler, SchedulerInput::RequestShutdown, 11));
    assert_eq!(shutdown.get(), 0);
    assert_eq!(scheduler.state(), SchedulerState::ShutdownQueued);
    assert_eq!(scheduler.stats().controls().items(), 1);
    assert_eq!(scheduler.stats().visual().items(), 1);

    let error = scheduler
        .process_input(SchedulerInput::FrameReady, SchedulerTick::new(11))
        .expect_err("visual input after accepted shutdown must be rejected");
    assert_eq!(error.kind(), SchedulerErrorKind::ControlPending);
    assert_eq!(scheduler.stats().visual().items(), 1);

    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(11))
            .expect("shutdown should be delivered before the frame request"),
        Some(SchedulerAction::StopRenderer(shutdown))
    );
    assert_eq!(scheduler.state(), SchedulerState::Stopped);
    assert_eq!(scheduler.stats().controls().items(), 0);
    assert_eq!(scheduler.stats().visual().items(), 0);

    assert_eq!(
        already_accepted(process(&mut scheduler, SchedulerInput::RequestShutdown, 12,)),
        shutdown
    );
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(12))
            .expect("duplicate shutdown must not emit another stop"),
        None
    );
}

#[test]
fn loss_and_shutdown_never_retire_an_accepted_submission() {
    let mut scheduler = scheduler(QueueCapacity::new(2, 64, 8));
    let submitted = submit_width(&mut scheduler, 130, 1);
    assert_eq!(scheduler.stats().in_flight().items(), 1);

    commit_width(&mut scheduler, 140, 4);
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(4))
            .expect("second request should be emitted"),
        Some(SchedulerAction::RequestFrame)
    );
    assert_eq!(
        process(&mut scheduler, SchedulerInput::FrameReady, 5),
        SchedulerInputResult::FrameReady
    );
    let _unaccepted = take_offer(&mut scheduler, 5);
    assert_eq!(scheduler.stats().visual().items(), 1);

    let loss = accepted(process(
        &mut scheduler,
        SchedulerInput::RendererLost(submitted.epoch()),
        6,
    ));
    let shutdown = accepted(process(&mut scheduler, SchedulerInput::RequestShutdown, 6));
    assert_eq!((loss.get(), shutdown.get()), (0, 1));

    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(6))
            .expect("loss should process before shutdown"),
        None
    );
    assert_eq!(scheduler.state(), SchedulerState::Faulted);
    assert_eq!(scheduler.stats().visual().items(), 0);
    assert_eq!(scheduler.stats().in_flight().items(), 1);

    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(6))
            .expect("shutdown should stop renderer delivery"),
        Some(SchedulerAction::StopRenderer(shutdown))
    );
    assert_eq!(scheduler.state(), SchedulerState::Draining);
    assert_eq!(scheduler.stats().in_flight().items(), 1);

    let completion = accepted(process(
        &mut scheduler,
        SchedulerInput::Complete(CompletionWatermark::from_submission(submitted)),
        7,
    ));
    assert_eq!(completion.get(), 2);
    assert_eq!(scheduler.stats().in_flight().items(), 1);
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(7))
            .expect("late completion control should remain processable"),
        None
    );
    assert_eq!(scheduler.stats().in_flight().items(), 0);
    assert_eq!(scheduler.state(), SchedulerState::Stopped);
}
