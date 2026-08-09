mod support;

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{
    CompletionWatermark, ControlAdmission, ControlSequence, QueueCapacity, SchedulerAction,
    SchedulerCapacity, SchedulerErrorKind, SchedulerInput, SchedulerInputResult, SchedulerLane,
    SchedulerState, SchedulerTick, SubmissionId, UiRuntime, UiScheduler, VisualCancelResult,
};

use support::{WIDTH, capacity, construction};

fn scheduler() -> UiScheduler {
    scheduler_with(QueueCapacity::new(1, 40, 8), QueueCapacity::new(2, 80, 8))
}

fn scheduler_with(visual: QueueCapacity, in_flight: QueueCapacity) -> UiScheduler {
    let runtime = UiRuntime::new(construction(), capacity().with_retained_generations(3))
        .expect("runtime should initialize");
    let scheduler_capacity = SchedulerCapacity::new(
        QueueCapacity::new(1, 80, 8),
        QueueCapacity::new(4, 128, 8),
        visual,
        in_flight,
    );
    UiScheduler::new(runtime, scheduler_capacity).expect("scheduler capacity should be valid")
}

fn stage_width(
    scheduler: &UiScheduler,
    width: i32,
) -> fenestra_ui_runtime::prototype::UiTransaction {
    let root = scheduler.committed().root();
    let mut transaction = scheduler.begin_transaction();
    transaction
        .set_property(root, WIDTH, PropertyValue::ScalarI32(width))
        .expect("property write should stage");
    transaction
}

fn submit_width(scheduler: &mut UiScheduler, width: i32, base_tick: u64) -> SubmissionId {
    let transaction = stage_width(scheduler, width);
    scheduler
        .commit(transaction, SchedulerTick::new(base_tick))
        .expect("property write should commit");
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(base_tick))
            .expect("request tick should be accepted"),
        Some(SchedulerAction::RequestFrame)
    );
    scheduler
        .process_input(
            SchedulerInput::FrameReady,
            SchedulerTick::new(base_tick + 1),
        )
        .expect("frame-ready should be accepted");
    let Some(SchedulerAction::OfferFrame(work)) = scheduler
        .next_action(SchedulerTick::new(base_tick + 1))
        .expect("offer tick should be accepted")
    else {
        panic!("one frame should be offered");
    };
    let SchedulerInputResult::FrameAccepted(submission) = scheduler
        .process_input(
            SchedulerInput::AcceptFrame(work.id()),
            SchedulerTick::new(base_tick + 2),
        )
        .expect("offer should be accepted")
    else {
        panic!("acceptance should return one submission");
    };
    submission
}

fn admit_completion(
    scheduler: &mut UiScheduler,
    submission: SubmissionId,
    tick: u64,
) -> ControlSequence {
    let result = scheduler
        .process_input(
            SchedulerInput::Complete(CompletionWatermark::from_submission(submission)),
            SchedulerTick::new(tick),
        )
        .expect("completion control should remain admissible");
    let SchedulerInputResult::Control(ControlAdmission::Accepted(sequence)) = result else {
        panic!("new completion should receive one control sequence");
    };
    sequence
}

#[test]
fn item_and_byte_ceilings_independently_defer_the_next_offer() {
    for in_flight in [QueueCapacity::new(1, 80, 8), QueueCapacity::new(2, 40, 8)] {
        let mut scheduler = scheduler_with(QueueCapacity::new(1, 40, 8), in_flight);
        let first = submit_width(&mut scheduler, 130, 10);
        let second = stage_width(&scheduler, 140);
        scheduler
            .commit(second, SchedulerTick::new(12))
            .expect("second generation should commit");
        assert_eq!(
            scheduler
                .next_action(SchedulerTick::new(12))
                .expect("frame request should be emitted"),
            Some(SchedulerAction::RequestFrame)
        );
        scheduler
            .process_input(SchedulerInput::FrameReady, SchedulerTick::new(13))
            .expect("frame-ready should be accepted");
        assert_eq!(
            scheduler
                .next_action(SchedulerTick::new(13))
                .expect("full submission lane should defer the offer"),
            None
        );
        assert_eq!(scheduler.stats().visual().items(), 1);
        assert_eq!(scheduler.stats().visual().accounted_bytes(), 40);
        assert_eq!(scheduler.stats().in_flight().items(), 1);
        assert_eq!(scheduler.stats().in_flight().accounted_bytes(), 40);

        let completion = admit_completion(&mut scheduler, first, 14);
        assert_eq!(completion.get(), 0);
        assert_eq!(scheduler.stats().in_flight().items(), 1);
        assert_eq!(
            scheduler
                .next_action(SchedulerTick::new(14))
                .expect("completion should process before visual work"),
            None
        );
        assert!(matches!(
            scheduler
                .next_action(SchedulerTick::new(14))
                .expect("pending work should become offerable"),
            Some(SchedulerAction::OfferFrame(_))
        ));
    }

    let runtime = UiRuntime::new(construction(), capacity().with_retained_generations(3))
        .expect("runtime should initialize");
    let impossible = SchedulerCapacity::new(
        QueueCapacity::new(1, 80, 8),
        QueueCapacity::new(4, 128, 8),
        QueueCapacity::new(1, 40, 8),
        QueueCapacity::new(usize::MAX, 80, 8),
    );
    let error = UiScheduler::new(runtime, impossible)
        .err()
        .expect("retention arithmetic must be checked");
    assert_eq!(error.kind(), SchedulerErrorKind::ArithmeticExhausted);
}

#[test]
fn visual_residence_is_inclusive_then_blocks_publication_without_dropping_work() {
    let mut scheduler = scheduler_with(QueueCapacity::new(1, 40, 8), QueueCapacity::new(0, 0, 8));
    let first = stage_width(&scheduler, 130);
    scheduler
        .commit(first, SchedulerTick::new(10))
        .expect("property write should commit");

    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(10))
            .expect("one frame request should be emitted"),
        Some(SchedulerAction::RequestFrame)
    );
    scheduler
        .process_input(SchedulerInput::FrameReady, SchedulerTick::new(11))
        .expect("frame-ready should make visual work pending");

    let replacement = stage_width(&scheduler, 140);
    scheduler
        .commit(replacement, SchedulerTick::new(18))
        .expect("the exact residence deadline is inclusive");
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(18))
            .expect("zero submission capacity should defer the pending frame"),
        None
    );
    assert_eq!(scheduler.state(), SchedulerState::Running);

    let error = scheduler
        .next_action(SchedulerTick::new(19))
        .expect_err("the first observed deadline crossing should enter pressure");
    assert_eq!(
        error.kind(),
        SchedulerErrorKind::ResidenceExceeded(SchedulerLane::Visual)
    );
    assert_eq!(scheduler.state(), SchedulerState::Faulted);
    assert_eq!(scheduler.committed().generation().get(), 2);
    assert_eq!(scheduler.stats().visual().items(), 1);
    assert_eq!(scheduler.stats().visual().accounted_bytes(), 40);
    assert_eq!(
        scheduler.stats().visual().earliest_tick(),
        Some(SchedulerTick::new(10))
    );
    assert_eq!(
        scheduler.stats().visual().latest_tick(),
        Some(SchedulerTick::new(18))
    );

    let regression = scheduler
        .cancel_visual(SchedulerTick::new(18))
        .expect_err("tick regression should precede the residence latch");
    assert_eq!(regression.kind(), SchedulerErrorKind::TickRegression);
    assert_eq!(
        scheduler
            .cancel_visual(SchedulerTick::new(19))
            .expect("replaceable visual work may be canceled explicitly"),
        VisualCancelResult::Canceled
    );
    assert_eq!(scheduler.stats().visual().items(), 0);
    assert_eq!(
        scheduler
            .cancel_visual(SchedulerTick::new(19))
            .expect("canceling an empty visual lane is idempotent"),
        VisualCancelResult::AlreadyEmpty
    );

    let blocked = stage_width(&scheduler, 150);
    let error = scheduler
        .commit(blocked, SchedulerTick::new(19))
        .expect_err("canceling work must not clear terminal pressure");
    assert_eq!(
        error.kind(),
        SchedulerErrorKind::ResidenceExceeded(SchedulerLane::Visual)
    );
    assert_eq!(scheduler.committed().generation().get(), 2);
}

#[test]
fn stalled_submission_stays_retained_but_late_completion_remains_admissible() {
    let mut scheduler = scheduler();
    let first = submit_width(&mut scheduler, 130, 10);
    let second = submit_width(&mut scheduler, 140, 12);

    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(20))
            .expect("the exact residence deadline is inclusive"),
        None
    );
    let error = scheduler
        .next_action(SchedulerTick::new(21))
        .expect_err("the first observed crossing should report a stalled submission");
    assert_eq!(
        error.kind(),
        SchedulerErrorKind::ResidenceExceeded(SchedulerLane::InFlight)
    );
    assert_eq!(scheduler.state(), SchedulerState::Faulted);
    assert_eq!(scheduler.stats().in_flight().items(), 2);
    assert_eq!(scheduler.stats().in_flight().accounted_bytes(), 80);

    let first_completion = admit_completion(&mut scheduler, first, 21);
    assert_eq!(first_completion.get(), 0);
    assert_eq!(scheduler.stats().in_flight().items(), 2);
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(21))
            .expect("completion must process after residence pressure"),
        None
    );
    assert_eq!(scheduler.stats().in_flight().items(), 1);
    assert_eq!(scheduler.stats().in_flight().accounted_bytes(), 40);
    let second_completion = admit_completion(&mut scheduler, second, 21);
    assert_eq!(second_completion.get(), 1);
    assert_eq!(scheduler.stats().in_flight().items(), 1);
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(21))
            .expect("the final accepted submission should still complete"),
        None
    );
    assert_eq!(scheduler.stats().in_flight().items(), 0);
    assert_eq!(scheduler.state(), SchedulerState::Faulted);

    let blocked = stage_width(&scheduler, 150);
    let error = scheduler
        .commit(blocked, SchedulerTick::new(21))
        .expect_err("completion must not clear terminal residence pressure");
    assert_eq!(
        error.kind(),
        SchedulerErrorKind::ResidenceExceeded(SchedulerLane::InFlight)
    );
}

#[test]
fn simultaneous_residence_crossings_follow_scheduler_lane_order() {
    let mut scheduler = scheduler();
    let _submission = submit_width(&mut scheduler, 130, 10);
    let transaction = stage_width(&scheduler, 140);
    scheduler
        .commit(transaction, SchedulerTick::new(13))
        .expect("a second generation should commit");

    let error = scheduler
        .next_action(SchedulerTick::new(22))
        .expect_err("both live lanes are beyond their residence deadlines");
    assert_eq!(
        error.kind(),
        SchedulerErrorKind::ResidenceExceeded(SchedulerLane::Visual)
    );
    assert_eq!(scheduler.stats().visual().items(), 1);
    assert_eq!(scheduler.stats().in_flight().items(), 1);
}
