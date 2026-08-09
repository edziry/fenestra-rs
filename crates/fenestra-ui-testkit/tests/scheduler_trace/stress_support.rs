use fenestra_ui_ir::prototype::{PropertyId, PropertyValue};
use fenestra_ui_runtime::prototype::{
    CompletionWatermark, FrameWork, QueueCapacity, SchedulerAction, SchedulerErrorKind,
    SchedulerInput, SchedulerInputResult, SchedulerState, SubmissionId, UiScheduler,
};
use fenestra_ui_testkit::prototype::{
    FakeClockDomainV1, FakeClockV1, FakeControlDeliveryV1, FakeFrameReadyDeliveryV1,
    FakePlatformV1, FakeRendererCapacityV1, FakeRendererModeV1, FakeRendererOfferOutcomeV1,
    FakeRendererV1, SchedulerTraceActionV1, SchedulerTraceCapacityV1,
    SchedulerTraceCommitOutcomeV1, SchedulerTraceEventV1, SchedulerTraceInputOutcomeV1,
    SchedulerTraceLaneStatsV1, SchedulerTraceStepV1, SchedulerTraceV1, SyntheticResourceIdV1,
    SyntheticResourceUseV1,
};

pub(super) const WIDTH: PropertyId = PropertyId::new(0);
pub(super) const DEFERRED: QueueCapacity = QueueCapacity::new(1, 80, 8);
pub(super) const CONTROLS: QueueCapacity = QueueCapacity::new(4, 128, 8);
pub(super) const VISUAL: QueueCapacity = QueueCapacity::new(1, 40, 8);
pub(super) const IN_FLIGHT: QueueCapacity = QueueCapacity::new(2, 80, 8);
pub(super) const RENDERER: FakeRendererCapacityV1 = FakeRendererCapacityV1::new(2, 192, 8);
pub(super) const TRACE: SchedulerTraceCapacityV1 = SchedulerTraceCapacityV1::new(256, 24_576);
pub(super) const DOMAIN: FakeClockDomainV1 = FakeClockDomainV1::new(6);
pub(super) const EXPECTED_EVENTS: usize = 97;
pub(super) const EXPECTED_TRACE_BYTES: usize =
    EXPECTED_EVENTS * SchedulerTraceEventV1::ACCOUNTED_BYTES;

pub(super) fn requested_offer(
    scheduler: &mut UiScheduler,
    platform: &mut FakePlatformV1,
    trace: &mut SchedulerTraceV1,
    clock: &FakeClockV1,
    renderer: &FakeRendererV1,
) -> FrameWork {
    assert_eq!(
        next_action(scheduler, trace, clock, renderer),
        Some(SchedulerAction::RequestFrame)
    );
    assert_eq!(
        frame_ready(platform, scheduler, trace, clock, renderer, false),
        FakeFrameReadyDeliveryV1::Accepted
    );
    take_offer(next_action(scheduler, trace, clock, renderer))
}

pub(super) fn commit_width(
    scheduler: &mut UiScheduler,
    trace: &mut SchedulerTraceV1,
    clock: &FakeClockV1,
    renderer: &FakeRendererV1,
    width: i32,
) {
    let root = scheduler.committed().root();
    let mut transaction = scheduler.begin_transaction();
    transaction
        .set_property(root, WIDTH, PropertyValue::ScalarI32(width))
        .expect("width mutation should stage");
    let commit = scheduler
        .commit(transaction, clock.now())
        .expect("width mutation should publish");
    assert_eq!(commit.mutation_count(), 1);
    record(
        trace,
        clock,
        SchedulerTraceStepV1::Commit(SchedulerTraceCommitOutcomeV1::Published),
        scheduler,
        renderer,
    );
}

pub(super) fn next_action(
    scheduler: &mut UiScheduler,
    trace: &mut SchedulerTraceV1,
    clock: &FakeClockV1,
    renderer: &FakeRendererV1,
) -> Option<SchedulerAction> {
    let action = scheduler
        .next_action(clock.now())
        .expect("scheduler turn should advance");
    let projected = SchedulerTraceActionV1::from_action(action.as_ref());
    record(
        trace,
        clock,
        SchedulerTraceStepV1::Action(projected),
        scheduler,
        renderer,
    );
    action
}

pub(super) fn frame_ready(
    platform: &mut FakePlatformV1,
    scheduler: &mut UiScheduler,
    trace: &mut SchedulerTraceV1,
    clock: &FakeClockV1,
    renderer: &FakeRendererV1,
    retry: bool,
) -> FakeFrameReadyDeliveryV1 {
    let delivery = if retry {
        platform.retry_frame_ready(scheduler, clock.now())
    } else {
        platform.frame_ready(scheduler, clock.now())
    }
    .expect("frame-ready delivery should have a typed outcome");
    let outcome = match delivery {
        FakeFrameReadyDeliveryV1::Accepted => {
            SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::FrameReady)
        }
        FakeFrameReadyDeliveryV1::Retained(kind) => SchedulerTraceInputOutcomeV1::Retained(kind),
        FakeFrameReadyDeliveryV1::Canceled => SchedulerTraceInputOutcomeV1::Canceled,
    };
    record_input(
        trace,
        clock,
        SchedulerInput::FrameReady,
        outcome,
        scheduler,
        renderer,
    );
    delivery
}

pub(super) fn accept_late(
    scheduler: &mut UiScheduler,
    renderer: &mut FakeRendererV1,
    trace: &mut SchedulerTraceV1,
    clock: &FakeClockV1,
    work: FrameWork,
    resource_id: u64,
) -> SubmissionId {
    let frame = work.id();
    let resource = resource(resource_id);
    let FakeRendererOfferOutcomeV1::Accepted(submission) = renderer
        .offer(
            scheduler,
            work,
            &[resource],
            FakeRendererModeV1::Late,
            clock.now(),
        )
        .expect("late renderer should accept bounded work")
    else {
        panic!("late renderer should return a submission");
    };
    record_input(
        trace,
        clock,
        SchedulerInput::AcceptFrame(frame),
        SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::FrameAccepted(submission)),
        scheduler,
        renderer,
    );
    submission
}

pub(super) fn complete(
    scheduler: &mut UiScheduler,
    renderer: &mut FakeRendererV1,
    trace: &mut SchedulerTraceV1,
    clock: &FakeClockV1,
    submission: SubmissionId,
) {
    let watermark = CompletionWatermark::from_submission(submission);
    let FakeControlDeliveryV1::Accepted(admission) = renderer
        .complete(scheduler, watermark, clock.now())
        .expect("completion should remain admissible")
    else {
        panic!("completion should enter the control lane");
    };
    record_input(
        trace,
        clock,
        SchedulerInput::Complete(watermark),
        SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::Control(admission)),
        scheduler,
        renderer,
    );
}

pub(super) fn record_input(
    trace: &mut SchedulerTraceV1,
    clock: &FakeClockV1,
    input: SchedulerInput,
    outcome: SchedulerTraceInputOutcomeV1,
    scheduler: &UiScheduler,
    renderer: &FakeRendererV1,
) {
    record(
        trace,
        clock,
        SchedulerTraceStepV1::Input { input, outcome },
        scheduler,
        renderer,
    );
}

pub(super) fn record(
    trace: &mut SchedulerTraceV1,
    clock: &FakeClockV1,
    step: SchedulerTraceStepV1,
    scheduler: &UiScheduler,
    renderer: &FakeRendererV1,
) {
    trace
        .record(clock, step, scheduler, renderer)
        .expect("registered trace capacity should hold");
}

pub(super) fn take_offer(action: Option<SchedulerAction>) -> FrameWork {
    let Some(SchedulerAction::OfferFrame(work)) = action else {
        panic!("one frame offer should be ready");
    };
    work
}

pub(super) const fn resource(id: u64) -> SyntheticResourceUseV1 {
    SyntheticResourceUseV1::new(SyntheticResourceIdV1::new(id), 64)
}

pub(super) fn advance(clock: &mut FakeClockV1) {
    advance_by(clock, 1);
}

pub(super) fn advance_by(clock: &mut FakeClockV1, delta: u64) {
    clock
        .advance(delta)
        .expect("registered script tick should advance");
}

pub(super) fn assert_events_are_bounded(events: &[SchedulerTraceEventV1]) {
    assert!(!events.is_empty());
    let mut prior_tick = None;
    for (index, event) in events.iter().copied().enumerate() {
        assert_eq!(event.sequence(), index as u64);
        assert_eq!(event.clock_domain(), DOMAIN);
        assert!(prior_tick.is_none_or(|prior| prior <= event.tick()));
        prior_tick = Some(event.tick());
        assert_lane(event.deferred(), DEFERRED);
        assert_lane(event.controls(), CONTROLS);
        assert_lane(event.visual(), VISUAL);
        assert_lane(event.in_flight(), IN_FLIGHT);
        assert_eq!(
            event.deferred().accounted_bytes(),
            event.deferred().items() * 80
        );
        assert_eq!(
            event.controls().accounted_bytes(),
            event.controls().items() * 32
        );
        assert_eq!(
            event.visual().accounted_bytes(),
            event.visual().items() * 40
        );
        assert_eq!(
            event.in_flight().accounted_bytes(),
            event.in_flight().items() * 40
        );
        assert!(event.renderer().items() <= RENDERER.max_items());
        assert!(event.renderer().accounted_bytes() <= RENDERER.max_bytes());
        assert_eq!(
            event.renderer().accounted_bytes(),
            event.renderer().items() * 96
        );
        if let Some(age) = event.renderer().oldest_residence_ticks() {
            assert!(age <= RENDERER.residence_ticks());
        }
        assert_eq!(
            event.renderer().oldest_residence_ticks().is_some(),
            event.renderer().items() > 0
        );
    }
}

fn assert_lane(stats: SchedulerTraceLaneStatsV1, capacity: QueueCapacity) {
    assert!(stats.items() <= capacity.max_items());
    assert!(stats.accounted_bytes() <= capacity.max_bytes());
    if let Some(age) = stats.oldest_residence_ticks() {
        assert!(age <= capacity.residence_ticks());
    }
    assert_eq!(stats.oldest_residence_ticks().is_some(), stats.items() > 0);
}

pub(super) fn assert_script_coverage(events: &[SchedulerTraceEventV1]) {
    assert!(events.iter().any(|event| {
        event.callback_depth() == Some(3)
            && event.deferred().items() == 1
            && event.deferred().accounted_bytes() == 80
    }));
    assert!(events.iter().any(|event| {
        event.step() == SchedulerTraceStepV1::Action(SchedulerTraceActionV1::Idle)
            && event.visual().items() == 1
            && event.in_flight().items() == 2
    }));
    assert!(events.iter().any(|event| {
        event.renderer().items() == 2 && event.renderer().accounted_bytes() == 192
    }));
    assert!(events.iter().any(|event| {
        event.step()
            == SchedulerTraceStepV1::Input {
                input: SchedulerInput::FrameReady,
                outcome: SchedulerTraceInputOutcomeV1::Retained(SchedulerErrorKind::ControlPending),
            }
    }));
    assert!(events.iter().any(|event| matches!(
        event.step(),
        SchedulerTraceStepV1::Input {
            input: SchedulerInput::RejectFrame(_),
            outcome: SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::FrameRejected(_)),
        }
    )));
    assert!(events.iter().any(|event| matches!(
        event.step(),
        SchedulerTraceStepV1::Input {
            input: SchedulerInput::RendererLost(_),
            ..
        }
    )));
    assert!(events.iter().any(|event| {
        event.step() == SchedulerTraceStepV1::Action(SchedulerTraceActionV1::Idle)
            && event.lifecycle() == SchedulerState::Faulted
            && event.in_flight().items() == 1
            && event.renderer().items() == 1
    }));
    assert!(events.iter().any(|event| matches!(
        event.step(),
        SchedulerTraceStepV1::Action(SchedulerTraceActionV1::StopRenderer(_))
    )));
    assert_eq!(
        events
            .iter()
            .filter_map(|event| event.control())
            .map(|sequence| sequence.get())
            .collect::<Vec<_>>(),
        [0, 1, 2, 3, 3, 3, 4]
    );
    assert!(events.iter().any(|event| {
        event.in_flight().oldest_residence_ticks() == Some(8)
            && event.renderer().oldest_residence_ticks() == Some(8)
    }));
    let final_event = events.last().copied().expect("script should emit events");
    assert_eq!(final_event.lifecycle(), SchedulerState::Stopped);
    assert_eq!(final_event.deferred().items(), 0);
    assert_eq!(final_event.controls().items(), 0);
    assert_eq!(final_event.visual().items(), 0);
    assert_eq!(final_event.in_flight().items(), 0);
    assert_eq!(final_event.renderer().items(), 0);
}
