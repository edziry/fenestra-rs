use fenestra_ui_testkit::prototype::{
    SchedulerTraceActionV1, SchedulerTraceErrorKindV1, SchedulerTraceEventV1,
    SchedulerTraceLimitV1, SchedulerTraceStepV1,
};

use super::{clock, record, renderer, scheduler, trace};

#[test]
fn event_and_byte_limits_are_inclusive_independent_and_atomic() {
    for (max_events, max_bytes, expected) in [
        (1, 192, SchedulerTraceLimitV1::Events),
        (2, 96, SchedulerTraceLimitV1::AccountedBytes),
        (1, 96, SchedulerTraceLimitV1::Events),
        (usize::MAX, 96, SchedulerTraceLimitV1::AccountedBytes),
    ] {
        let scheduler = scheduler();
        let renderer = renderer();
        let clock = clock(7, 0);
        let mut trace = trace(7, max_events, max_bytes);
        let step = SchedulerTraceStepV1::Action(SchedulerTraceActionV1::Idle);

        record(&mut trace, &clock, step, &scheduler, &renderer);
        assert_eq!(trace.len(), 1);
        assert_eq!(
            trace.accounted_bytes(),
            SchedulerTraceEventV1::ACCOUNTED_BYTES
        );
        let first = trace.events()[0];

        let error = trace
            .record(&clock, step, &scheduler, &renderer)
            .expect_err("the second fixed-size event should cross one bound");
        assert_eq!(
            error.kind(),
            SchedulerTraceErrorKindV1::LimitExceeded(expected)
        );
        assert_eq!(trace.events(), &[first]);
        assert_eq!(trace.len(), 1);
        assert_eq!(
            trace.accounted_bytes(),
            SchedulerTraceEventV1::ACCOUNTED_BYTES
        );
    }
}

#[test]
fn clock_domain_and_tick_errors_preserve_the_exact_prefix() {
    let scheduler = scheduler();
    let renderer = renderer();
    let mut trace = trace(7, 3, 3 * SchedulerTraceEventV1::ACCOUNTED_BYTES);
    let step = SchedulerTraceStepV1::Action(SchedulerTraceActionV1::Idle);
    record(&mut trace, &clock(7, 2), step, &scheduler, &renderer);
    record(&mut trace, &clock(7, 2), step, &scheduler, &renderer);
    let prefix = trace.events().to_vec();

    let domain = trace
        .record(&clock(8, 2), step, &scheduler, &renderer)
        .expect_err("another fake clock domain must be rejected");
    assert_eq!(
        domain.kind(),
        SchedulerTraceErrorKindV1::ClockDomainMismatch
    );
    assert_eq!(trace.events(), prefix);

    let regression = trace
        .record(&clock(7, 1), step, &scheduler, &renderer)
        .expect_err("trace ticks must not move backward");
    assert_eq!(regression.kind(), SchedulerTraceErrorKindV1::TickRegression);
    assert_eq!(trace.events(), prefix);
}
