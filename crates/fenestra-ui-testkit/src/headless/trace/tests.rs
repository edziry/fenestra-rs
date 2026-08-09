use fenestra_ui_runtime::prototype::{
    QueueCapacity, RendererEpoch, SchedulerCapacity, SchedulerTick, UiRuntime, UiScheduler,
};

use super::*;
use crate::headless::fixture::HeadlessFixtureV1;
use crate::scheduler::FakeRendererCapacityV1;

const DOMAIN: FakeClockDomainV1 = FakeClockDomainV1::new(91);

#[test]
fn inclusive_event_and_byte_limits_accept_the_exact_boundary() {
    let (scheduler, renderer, clock) = environment();
    let mut trace = HeadlessTraceV1::new(
        DOMAIN,
        HeadlessTraceCapacityV1::new(1, HeadlessTraceEventV1::ACCOUNTED_BYTES),
    );

    trace
        .record(&clock, observed_step(), &scheduler, &renderer)
        .expect("the exact event and byte boundary should be inclusive");

    assert_eq!(trace.len(), 1);
    assert_eq!(
        trace.accounted_bytes(),
        HeadlessTraceEventV1::ACCOUNTED_BYTES
    );
}

#[test]
fn event_limit_has_priority_and_preserves_the_accepted_prefix() {
    let (scheduler, renderer, clock) = environment();
    let mut trace = HeadlessTraceV1::new(
        DOMAIN,
        HeadlessTraceCapacityV1::new(1, HeadlessTraceEventV1::ACCOUNTED_BYTES),
    );
    trace
        .record(&clock, observed_step(), &scheduler, &renderer)
        .expect("the first event should fit");
    let prefix = trace.events().to_vec();

    let error = trace
        .record(&clock, observed_step(), &scheduler, &renderer)
        .expect_err("the second event exceeds both dimensions");

    assert_eq!(error.kind(), HeadlessTraceErrorKind::EventLimitExceeded);
    assert_eq!(trace.events(), prefix);
    assert_eq!(trace.next_sequence, Some(1));
    assert_eq!(trace.last_tick, Some(SchedulerTick::new(0)));
}

#[test]
fn byte_limit_preserves_sequence_after_one_accepted_event() {
    let (scheduler, renderer, clock) = environment();
    let mut trace = HeadlessTraceV1::new(
        DOMAIN,
        HeadlessTraceCapacityV1::new(2, HeadlessTraceEventV1::ACCOUNTED_BYTES),
    );
    trace
        .record(&clock, observed_step(), &scheduler, &renderer)
        .expect("the first event should fit");

    let error = trace
        .record(&clock, observed_step(), &scheduler, &renderer)
        .expect_err("only the byte dimension should reject the second event");

    assert_eq!(error.kind(), HeadlessTraceErrorKind::ByteLimitExceeded);
    assert_eq!(trace.len(), 1);
    assert_eq!(trace.next_sequence, Some(1));
}

#[test]
fn checked_byte_arithmetic_preserves_an_empty_prefix() {
    let (scheduler, renderer, clock) = environment();
    let mut trace =
        HeadlessTraceV1::new(DOMAIN, HeadlessTraceCapacityV1::new(usize::MAX, usize::MAX));
    trace.accounted_bytes = usize::MAX;

    let error = trace
        .record(&clock, observed_step(), &scheduler, &renderer)
        .expect_err("fixed accounting must use checked addition");

    assert_eq!(error.kind(), HeadlessTraceErrorKind::ArithmeticExhausted);
    assert!(trace.events().is_empty());
    assert_eq!(trace.next_sequence, Some(0));
    assert_eq!(trace.last_tick, None);
}

fn environment() -> (UiScheduler, FakeRendererV1, FakeClockV1) {
    let fixture = HeadlessFixtureV1::build().expect("registered fixture should validate");
    let runtime = UiRuntime::new_headless(
        fixture.style().clone(),
        fixture.spec(),
        fixture.surface(),
        fixture.runtime_capacity(),
    )
    .expect("registered runtime should initialize");
    let capacity = SchedulerCapacity::new(
        QueueCapacity::new(1, 80, 8),
        QueueCapacity::new(4, 128, 8),
        QueueCapacity::new(1, 40, 8),
        QueueCapacity::new(2, 80, 8),
    );
    let scheduler = UiScheduler::new(runtime, capacity).expect("scheduler bounds should validate");
    let renderer = FakeRendererV1::new(
        RendererEpoch::new(0),
        FakeRendererCapacityV1::new(2, 192, 8),
    );
    let clock = FakeClockV1::new(DOMAIN, SchedulerTick::new(0));
    (scheduler, renderer, clock)
}

const fn observed_step() -> HeadlessTraceStep {
    HeadlessTraceStep::new(
        HeadlessTraceStageV1::Build,
        HeadlessInputKindV1::None,
        HeadlessOutcomeV1::Observed,
    )
}
