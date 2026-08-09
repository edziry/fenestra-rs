mod support;

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{
    CompletionWatermark, ControlAdmission, ControlSequence, FrameId, QueueCapacity, RendererEpoch,
    SchedulerAction, SchedulerCapacity, SchedulerErrorKind, SchedulerInput, SchedulerInputResult,
    SchedulerState, SchedulerTick, SubmissionId, UiRuntime, UiScheduler,
};

use support::{WIDTH, capacity, construction};

fn scheduler() -> UiScheduler {
    let runtime = UiRuntime::new(construction(), capacity()).expect("runtime should initialize");
    UiScheduler::new(
        runtime,
        SchedulerCapacity::new(
            QueueCapacity::new(1, 80, 8),
            QueueCapacity::new(4, 128, 8),
            QueueCapacity::new(1, 40, 8),
            QueueCapacity::new(2, 80, 8),
        ),
    )
    .expect("scheduler capacity should be valid")
}

fn accepted(result: SchedulerInputResult) -> ControlSequence {
    let SchedulerInputResult::Control(ControlAdmission::Accepted(sequence)) = result else {
        panic!("control should receive a fresh sequence");
    };
    sequence
}

fn process(scheduler: &mut UiScheduler, input: SchedulerInput, tick: u64) -> SchedulerInputResult {
    scheduler
        .process_input(input, SchedulerTick::new(tick))
        .expect("scheduler input should be accepted")
}

fn accept_offer(scheduler: &mut UiScheduler, frame: FrameId, tick: u64) -> SubmissionId {
    let SchedulerInputResult::FrameAccepted(submission) =
        process(scheduler, SchedulerInput::AcceptFrame(frame), tick)
    else {
        panic!("offer should become a submission");
    };
    submission
}

fn submit_width(scheduler: &mut UiScheduler, width: i32, tick: u64) -> SubmissionId {
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
    assert_eq!(
        process(scheduler, SchedulerInput::FrameReady, tick + 1),
        SchedulerInputResult::FrameReady
    );
    let Some(SchedulerAction::OfferFrame(work)) = scheduler
        .next_action(SchedulerTick::new(tick + 1))
        .expect("offer turn should advance")
    else {
        panic!("frame should be offered");
    };
    accept_offer(scheduler, work.id(), tick + 2)
}

#[test]
fn projected_completion_blocks_publication_and_orders_validation_before_duplicates() {
    let mut scheduler = scheduler();
    let _first = submit_width(&mut scheduler, 130, 1);
    let second = submit_width(&mut scheduler, 140, 3);
    let watermark = CompletionWatermark::from_submission(second);
    let completion = accepted(process(
        &mut scheduler,
        SchedulerInput::Complete(watermark),
        5,
    ));
    assert_eq!(completion.get(), 0);
    assert_eq!(scheduler.stats().controls().items(), 1);
    assert_eq!(scheduler.stats().in_flight().items(), 2);

    let lower = CompletionWatermark::new(second.epoch(), second.token() - 1);
    let error = scheduler
        .process_input(SchedulerInput::Complete(lower), SchedulerTick::new(5))
        .expect_err("queued completion is part of the projected watermark");
    assert_eq!(error.kind(), SchedulerErrorKind::CompletionRegression);
    assert_eq!(
        process(&mut scheduler, SchedulerInput::Complete(watermark), 5),
        SchedulerInputResult::Control(ControlAdmission::AlreadyAccepted(completion))
    );
    let error = scheduler
        .process_input(SchedulerInput::Complete(watermark), SchedulerTick::new(4))
        .expect_err("tick regression precedes duplicate admission");
    assert_eq!(error.kind(), SchedulerErrorKind::TickRegression);

    let root = scheduler.committed().root();
    let mut transaction = scheduler.begin_transaction();
    transaction
        .set_property(root, WIDTH, PropertyValue::ScalarI32(150))
        .expect("property write should stage");
    let error = scheduler
        .commit(transaction, SchedulerTick::new(5))
        .expect_err("queued completion must precede ordinary publication");
    assert_eq!(error.kind(), SchedulerErrorKind::ControlPending);
    let error = scheduler
        .begin_callback(SchedulerTick::new(5))
        .err()
        .expect("queued completion must precede another callback");
    assert_eq!(error.kind(), SchedulerErrorKind::ControlPending);

    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(5))
            .expect("completion should process on the next turn"),
        None
    );
    assert_eq!(scheduler.stats().controls().items(), 0);
    assert_eq!(scheduler.stats().in_flight().items(), 0);
    assert_eq!(scheduler.state(), SchedulerState::Running);
}

#[test]
fn loss_without_shutdown_remains_faulted_after_late_completion() {
    let mut scheduler = scheduler();
    let submission = submit_width(&mut scheduler, 130, 1);
    let loss = accepted(process(
        &mut scheduler,
        SchedulerInput::RendererLost(RendererEpoch::new(0)),
        4,
    ));
    assert_eq!(loss.get(), 0);
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(4))
            .expect("loss should process without an adapter action"),
        None
    );
    assert_eq!(scheduler.state(), SchedulerState::Faulted);
    assert_eq!(scheduler.stats().in_flight().items(), 1);

    let completion = accepted(process(
        &mut scheduler,
        SchedulerInput::Complete(CompletionWatermark::from_submission(submission)),
        5,
    ));
    assert_eq!(completion.get(), 1);
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(5))
            .expect("late completion should remain processable"),
        None
    );
    assert_eq!(scheduler.stats().in_flight().items(), 0);
    assert_eq!(scheduler.state(), SchedulerState::Faulted);
}

#[test]
fn residence_pressure_precedes_admitting_a_new_loss_control() {
    let mut scheduler = scheduler();
    let _submission = submit_width(&mut scheduler, 130, 1);

    let error = scheduler
        .process_input(
            SchedulerInput::RendererLost(RendererEpoch::new(0)),
            SchedulerTick::new(12),
        )
        .expect_err("new loss must not hide the first residence crossing");
    assert_eq!(
        error.kind(),
        SchedulerErrorKind::ResidenceExceeded(
            fenestra_ui_runtime::prototype::SchedulerLane::InFlight,
        )
    );
    assert_eq!(scheduler.state(), SchedulerState::Faulted);
    assert_eq!(scheduler.stats().controls().items(), 0);
    assert_eq!(scheduler.stats().in_flight().items(), 1);
}
