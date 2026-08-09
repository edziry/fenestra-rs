mod support;

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{
    CapacityKind, CompletionWatermark, ControlAdmission, ControlSequence, FrameId, FrameWork,
    QueueCapacity, RendererEpoch, SchedulerAction, SchedulerCapacity, SchedulerErrorKind,
    SchedulerInput, SchedulerInputResult, SchedulerTick, SubmissionId, TransactionErrorKind,
    UiRuntime, UiScheduler,
};

use support::{WIDTH, capacity, construction, layout};

fn scheduler() -> UiScheduler {
    scheduler_with_retained(4)
}

fn scheduler_with_retained(retained_generations: usize) -> UiScheduler {
    let runtime = UiRuntime::new(
        construction(),
        capacity().with_retained_generations(retained_generations),
    )
    .expect("runtime should initialize");
    let scheduler_capacity = SchedulerCapacity::new(
        QueueCapacity::new(1, 80, 8),
        QueueCapacity::new(4, 128, 8),
        QueueCapacity::new(1, 40, 8),
        QueueCapacity::new(2, 80, 8),
    );
    UiScheduler::new(runtime, scheduler_capacity).expect("scheduler capacity should be valid")
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

fn next_action(scheduler: &mut UiScheduler, tick: u64) -> Option<SchedulerAction> {
    scheduler
        .next_action(SchedulerTick::new(tick))
        .expect("action tick should be monotonic")
}

fn frame_ready(scheduler: &mut UiScheduler, tick: u64) {
    assert_eq!(
        scheduler
            .process_input(SchedulerInput::FrameReady, SchedulerTick::new(tick))
            .expect("frame-ready input should be accepted"),
        SchedulerInputResult::FrameReady
    );
}

fn take_offer(scheduler: &mut UiScheduler, tick: u64) -> FrameWork {
    let Some(SchedulerAction::OfferFrame(work)) = next_action(scheduler, tick) else {
        panic!("one frame offer should be ready");
    };
    work
}

fn accept_offer(scheduler: &mut UiScheduler, frame: FrameId, tick: u64) -> SubmissionId {
    let result = scheduler
        .process_input(SchedulerInput::AcceptFrame(frame), SchedulerTick::new(tick))
        .expect("frame offer should be accepted");
    let SchedulerInputResult::FrameAccepted(submission) = result else {
        panic!("acceptance should return the submission identity");
    };
    submission
}

fn admit_completion(
    scheduler: &mut UiScheduler,
    watermark: CompletionWatermark,
    tick: u64,
) -> ControlSequence {
    let result = scheduler
        .process_input(
            SchedulerInput::Complete(watermark),
            SchedulerTick::new(tick),
        )
        .expect("completion control should be admitted");
    let SchedulerInputResult::Control(ControlAdmission::Accepted(sequence)) = result else {
        panic!("new completion should receive one control sequence");
    };
    sequence
}

fn submit_width(scheduler: &mut UiScheduler, width: i32, base_tick: u64) -> SubmissionId {
    commit_width(scheduler, width, base_tick);
    assert_eq!(
        next_action(scheduler, base_tick),
        Some(SchedulerAction::RequestFrame)
    );
    frame_ready(scheduler, base_tick + 1);
    let offer = take_offer(scheduler, base_tick + 1);
    accept_offer(scheduler, offer.id(), base_tick + 2)
}

#[test]
fn rejected_offer_returns_to_the_visual_lane_without_phantom_submission() {
    let mut scheduler = scheduler();
    commit_width(&mut scheduler, 130, 10);
    assert_eq!(
        next_action(&mut scheduler, 10),
        Some(SchedulerAction::RequestFrame)
    );
    frame_ready(&mut scheduler, 11);

    let offer = take_offer(&mut scheduler, 11);
    assert_eq!(offer.generation().get(), 1);
    assert!(offer.snapshot().shares_state_with(&scheduler.committed()));
    assert_eq!(offer.invalidation(), layout());
    assert_eq!(offer.accounted_bytes(), 40);
    assert_eq!(offer.earliest_tick(), SchedulerTick::new(10));
    assert_eq!(offer.latest_tick(), SchedulerTick::new(10));
    assert_eq!(scheduler.stats().visual().items(), 1);
    assert_eq!(scheduler.stats().in_flight().items(), 0);

    assert_eq!(
        scheduler
            .process_input(
                SchedulerInput::RejectFrame(offer.id()),
                SchedulerTick::new(12),
            )
            .expect("renderer pressure should reject the offer"),
        SchedulerInputResult::FrameRejected(offer.id())
    );
    assert_eq!(scheduler.stats().visual().items(), 1);
    assert_eq!(scheduler.stats().visual().accounted_bytes(), 40);
    assert_eq!(scheduler.stats().in_flight().items(), 0);

    let retry = take_offer(&mut scheduler, 12);
    assert_ne!(retry.id(), offer.id());
    assert_eq!(retry.generation(), offer.generation());
    assert!(retry.snapshot().shares_state_with(offer.snapshot()));
    assert_eq!(retry.invalidation(), offer.invalidation());
    assert_eq!(retry.earliest_tick(), offer.earliest_tick());
    assert_eq!(retry.latest_tick(), offer.latest_tick());
    let stale_error = scheduler
        .process_input(
            SchedulerInput::AcceptFrame(offer.id()),
            SchedulerTick::new(12),
        )
        .expect_err("late feedback must not accept a newer offer");
    assert_eq!(stale_error.kind(), SchedulerErrorKind::FrameIdMismatch);
    assert_eq!(scheduler.stats().visual().items(), 1);
    assert_eq!(scheduler.stats().in_flight().items(), 0);
    let submission = accept_offer(&mut scheduler, retry.id(), 13);
    assert_eq!(submission.epoch(), RendererEpoch::new(0));
    assert_eq!(submission.token(), 0);
    assert_eq!(scheduler.stats().visual().items(), 0);
    assert_eq!(scheduler.stats().in_flight().items(), 1);
    assert_eq!(scheduler.stats().in_flight().accounted_bytes(), 40);
}

#[test]
fn slow_renderer_keeps_one_latest_visual_and_completion_releases_a_prefix() {
    let mut scheduler = scheduler();
    let first = submit_width(&mut scheduler, 130, 10);
    let second = submit_width(&mut scheduler, 140, 12);
    assert_eq!(first.token(), 0);
    assert_eq!(second.token(), 1);
    assert_eq!(scheduler.stats().in_flight().items(), 2);
    assert_eq!(scheduler.stats().in_flight().accounted_bytes(), 80);

    commit_width(&mut scheduler, 150, 14);
    assert_eq!(
        next_action(&mut scheduler, 14),
        Some(SchedulerAction::RequestFrame)
    );
    frame_ready(&mut scheduler, 15);
    assert_eq!(next_action(&mut scheduler, 15), None);

    commit_width(&mut scheduler, 160, 15);
    assert_eq!(next_action(&mut scheduler, 15), None);
    assert_eq!(scheduler.committed().generation().get(), 4);
    assert_eq!(scheduler.stats().visual().items(), 1);
    assert_eq!(
        scheduler.stats().visual().earliest_tick(),
        Some(SchedulerTick::new(14))
    );
    assert_eq!(
        scheduler.stats().visual().latest_tick(),
        Some(SchedulerTick::new(15))
    );

    let first_completion = admit_completion(
        &mut scheduler,
        CompletionWatermark::from_submission(first),
        16,
    );
    assert_eq!(first_completion.get(), 0);
    assert_eq!(scheduler.stats().controls().items(), 1);
    assert_eq!(scheduler.stats().in_flight().items(), 2);
    assert_eq!(next_action(&mut scheduler, 16), None);
    assert_eq!(scheduler.stats().controls().items(), 0);
    assert_eq!(scheduler.stats().in_flight().items(), 1);

    let latest = take_offer(&mut scheduler, 16);
    assert_eq!(latest.generation().get(), 4);
    let third = accept_offer(&mut scheduler, latest.id(), 17);
    assert_eq!(third.token(), 2);
    assert_eq!(scheduler.stats().in_flight().items(), 2);

    let final_watermark = CompletionWatermark::from_submission(third);
    let final_completion = admit_completion(&mut scheduler, final_watermark, 18);
    assert_eq!(final_completion.get(), 1);
    assert_eq!(scheduler.stats().in_flight().items(), 2);
    assert_eq!(next_action(&mut scheduler, 18), None);
    assert_eq!(scheduler.stats().in_flight().items(), 0);
    assert_eq!(
        scheduler
            .process_input(
                SchedulerInput::Complete(final_watermark),
                SchedulerTick::new(18),
            )
            .expect("equal completion should be idempotent"),
        SchedulerInputResult::Control(ControlAdmission::AlreadyAccepted(final_completion))
    );
    assert_eq!(scheduler.stats().controls().items(), 0);

    for (watermark, expected) in [
        (
            CompletionWatermark::from_submission(second),
            SchedulerErrorKind::CompletionRegression,
        ),
        (
            CompletionWatermark::new(RendererEpoch::new(1), third.token()),
            SchedulerErrorKind::ForeignRendererEpoch,
        ),
        (
            CompletionWatermark::new(third.epoch(), third.token() + 1),
            SchedulerErrorKind::CompletionBeyondAccepted,
        ),
    ] {
        let error = scheduler
            .process_input(SchedulerInput::Complete(watermark), SchedulerTick::new(18))
            .expect_err("invalid completion watermark should fail");
        assert_eq!(error.kind(), expected);
        assert_eq!(scheduler.stats().in_flight().items(), 0);
    }
}

#[test]
fn completion_releases_the_snapshot_that_blocks_a_later_publication() {
    let mut scheduler = scheduler_with_retained(3);
    let first = submit_width(&mut scheduler, 130, 10);
    let _second = submit_width(&mut scheduler, 140, 12);

    commit_width(&mut scheduler, 150, 14);
    assert_eq!(
        next_action(&mut scheduler, 14),
        Some(SchedulerAction::RequestFrame)
    );
    frame_ready(&mut scheduler, 15);
    let retained_third_generation = scheduler.committed();
    commit_width(&mut scheduler, 160, 15);

    let root = scheduler.committed().root();
    let mut blocked = scheduler.begin_transaction();
    blocked
        .set_property(root, WIDTH, PropertyValue::ScalarI32(170))
        .expect("property write should stage");
    let error = scheduler
        .commit(blocked, SchedulerTick::new(16))
        .expect_err("three retained old generations should block publication");
    assert_eq!(
        error.kind(),
        SchedulerErrorKind::Transaction(TransactionErrorKind::CapacityExceeded(
            CapacityKind::RetainedGenerations,
        ))
    );

    let completion = admit_completion(
        &mut scheduler,
        CompletionWatermark::from_submission(first),
        17,
    );
    assert_eq!(completion.get(), 0);
    assert_eq!(scheduler.stats().in_flight().items(), 2);
    assert_eq!(next_action(&mut scheduler, 17), None);
    assert_eq!(scheduler.stats().in_flight().items(), 1);

    let mut unblocked = scheduler.begin_transaction();
    unblocked
        .set_property(root, WIDTH, PropertyValue::ScalarI32(170))
        .expect("property write should stage");
    let summary = scheduler
        .commit(unblocked, SchedulerTick::new(17))
        .expect("completion should make the next publication representable");
    assert_eq!(summary.generation().get(), 5);
    assert_eq!(scheduler.stats().in_flight().items(), 1);
    assert_eq!(retained_third_generation.generation().get(), 3);
}
