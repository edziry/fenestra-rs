#[path = "trace_expected/events.rs"]
mod events;
#[path = "trace_expected/scheduler.rs"]
mod scheduler_expected;
#[path = "trace_expected/state.rs"]
mod state;

use fenestra_ui_testkit::prototype::{
    FakeClockDomainV1, HeadlessInputKindV1, HeadlessOutcomeV1, HeadlessPointerTargetV1,
    HeadlessTraceEventV1, HeadlessTraceQueueStatsV1, HeadlessTraceStageV1, SchedulerTraceEventV1,
};

use self::events::expected_events;
use self::state::{State, assert_state};

#[derive(Clone, Copy)]
pub(super) struct ExpectedEvent {
    pub(super) tick: u64,
    pub(super) stage: HeadlessTraceStageV1,
    pub(super) input: HeadlessInputKindV1,
    pub(super) outcome: HeadlessOutcomeV1,
    pub(super) captured: Option<u64>,
    pub(super) published: Option<u64>,
    pub(super) target: HeadlessPointerTargetV1,
    pub(super) frame: Option<u64>,
    pub(super) control: Option<u64>,
    state: State,
}

pub fn assert_headless_events(events: &[HeadlessTraceEventV1], domain: FakeClockDomainV1) {
    let expected = expected_events();
    assert_eq!(events.len(), expected.len());
    for (index, (event, expected)) in events.iter().copied().zip(expected).enumerate() {
        assert_eq!(event.schema_revision(), 1);
        assert_eq!(event.sequence(), index as u64);
        assert_eq!(event.clock_domain(), domain);
        assert_eq!(event.tick().get(), expected.tick);
        assert_eq!(event.stage(), expected.stage);
        assert_eq!(event.input(), expected.input);
        assert_eq!(event.outcome(), expected.outcome);
        assert_eq!(
            event
                .captured_generation()
                .map(|generation| generation.get()),
            expected.captured
        );
        assert_eq!(
            event
                .published_generation()
                .map(|generation| generation.get()),
            expected.published
        );
        assert_eq!(event.target(), expected.target);
        assert_eq!(event.frame().map(|frame| frame.get()), expected.frame);
        assert_eq!(
            event.control().map(|control| control.get()),
            expected.control
        );
        assert_state(event, expected.state);
    }
}

pub fn assert_scheduler_correlation(
    headless: &[HeadlessTraceEventV1],
    scheduler: &[SchedulerTraceEventV1],
) {
    const HEADLESS_INDICES: [usize; 41] = [
        4, 5, 7, 9, 10, 12, 13, 15, 16, 18, 19, 20, 21, 22, 24, 25, 26, 27, 28, 29, 31, 33, 35, 36,
        37, 38, 39, 40, 41, 42, 43, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54,
    ];
    assert_eq!(scheduler.len(), HEADLESS_INDICES.len());

    let mut generation = None;
    let mut scheduler_index = 0;
    for (headless_index, event) in headless.iter().copied().enumerate() {
        if let Some(published) = event.published_generation() {
            generation = Some(published);
        }
        if HEADLESS_INDICES.get(scheduler_index) != Some(&headless_index) {
            continue;
        }
        let scheduler_event = scheduler[scheduler_index];
        assert_eq!(event.clock_domain(), scheduler_event.clock_domain());
        assert_eq!(event.tick(), scheduler_event.tick());
        assert_eq!(generation, Some(scheduler_event.generation()));
        if let Some(frame) = scheduler_event.frame() {
            assert_eq!(event.frame(), Some(frame));
        }
        if let Some(control) = scheduler_event.control() {
            assert_eq!(event.control(), Some(control));
        }
        assert_queue_pair(event.deferred(), scheduler_event.deferred());
        assert_queue_pair(event.controls(), scheduler_event.controls());
        assert_queue_pair(event.visual(), scheduler_event.visual());
        assert_queue_pair(event.in_flight(), scheduler_event.in_flight());
        assert_eq!(event.renderer().items(), scheduler_event.renderer().items());
        assert_eq!(
            event.renderer().accounted_bytes(),
            scheduler_event.renderer().accounted_bytes()
        );
        scheduler_index += 1;
    }
    assert_eq!(scheduler_index, scheduler.len());
    scheduler_expected::assert_scheduler_steps(scheduler);
    assert_residence(scheduler);
}

fn assert_queue_pair(
    headless: HeadlessTraceQueueStatsV1,
    scheduler: fenestra_ui_testkit::prototype::SchedulerTraceLaneStatsV1,
) {
    assert_eq!(headless.items(), scheduler.items());
    assert_eq!(headless.accounted_bytes(), scheduler.accounted_bytes());
}

fn assert_residence(events: &[SchedulerTraceEventV1]) {
    assert_eq!(events[12].tick().get(), 7);
    assert_eq!(events[12].visual().oldest_residence_ticks(), Some(5));
    assert_eq!(events[12].deferred().oldest_residence_ticks(), Some(0));

    assert_eq!(events[23].tick().get(), 11);
    assert_eq!(events[23].in_flight().oldest_residence_ticks(), Some(3));
    assert_eq!(events[23].renderer().oldest_residence_ticks(), Some(3));

    assert_eq!(events[29].tick().get(), 13);
    assert_eq!(events[29].in_flight().oldest_residence_ticks(), Some(2));
    assert_eq!(events[29].renderer().oldest_residence_ticks(), Some(2));

    assert_eq!(events[39].tick().get(), 18);
    assert_eq!(events[39].in_flight().oldest_residence_ticks(), Some(7));
    assert_eq!(events[39].renderer().oldest_residence_ticks(), None);

    let final_event = events[40];
    assert_eq!(final_event.tick().get(), 19);
    assert_eq!(final_event.deferred().oldest_residence_ticks(), None);
    assert_eq!(final_event.controls().oldest_residence_ticks(), None);
    assert_eq!(final_event.visual().oldest_residence_ticks(), None);
    assert_eq!(final_event.in_flight().oldest_residence_ticks(), None);
    assert_eq!(final_event.renderer().oldest_residence_ticks(), None);
}
