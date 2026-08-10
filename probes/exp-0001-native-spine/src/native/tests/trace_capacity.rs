use std::cell::Cell;

use fenestra_ui_runtime::prototype::SchedulerTick;

use super::super::trace::{
    NativeObservationV1, NativeOutcomeV1, NativeTraceCapacityV1, NativeTraceErrorKindV1,
    NativeTraceEventV1, NativeTraceLimitKindV1, NativeTraceStageV1, NativeTraceStepV1,
    NativeTraceV1,
};
use super::trace_step;

const EVENT_BYTES: usize = 192;
const MAX_EVENTS: usize = 128;
const MAX_BYTES: usize = 24_576;

#[test]
fn schema_capacity_accounting_and_sequence_are_fixed() {
    let mut trace = NativeTraceV1::new();
    assert_eq!(
        trace.capacity(),
        NativeTraceCapacityV1::new(MAX_EVENTS, MAX_BYTES)
    );
    assert!(trace.is_empty());

    trace
        .record(SchedulerTick::new(3), observed_build())
        .expect("first event should fit");
    trace
        .record(SchedulerTick::new(3), observed_build())
        .expect("equal ticks are monotonic");

    assert_eq!(NativeTraceEventV1::ACCOUNTED_BYTES, EVENT_BYTES);
    assert_eq!(trace.len(), 2);
    assert_eq!(trace.accounted_bytes(), 2 * EVENT_BYTES);
    assert_eq!(trace.events()[0].schema_revision(), 1);
    assert_eq!(trace.events()[0].sequence(), 0);
    assert_eq!(trace.events()[1].sequence(), 1);
    assert_eq!(trace.events()[1].tick(), SchedulerTick::new(3));
}

#[test]
fn registered_capacity_is_inclusive_and_rejects_one_over_atomically() {
    let mut trace = NativeTraceV1::new();
    for tick in 0..MAX_EVENTS {
        trace
            .record(SchedulerTick::new(tick as u64), observed_build())
            .expect("the registered boundary should be inclusive");
    }
    let prefix = trace.events().to_vec();

    let error = trace
        .record(SchedulerTick::new(MAX_EVENTS as u64), observed_build())
        .expect_err("the first event over the fixed bound must fail");

    assert_eq!(
        error,
        NativeTraceErrorKindV1::LimitExceeded(NativeTraceLimitKindV1::Events)
    );
    assert_eq!(trace.events(), prefix);
    assert_eq!(trace.accounted_bytes(), MAX_BYTES);
}

#[test]
fn admission_preflights_events_then_bytes_then_storage() {
    let storage_called = Cell::new(false);
    let mut events = NativeTraceV1::with_capacity_for_test(NativeTraceCapacityV1::new(0, 0));
    let error = events
        .record_with_reserver_for_test(SchedulerTick::new(0), observed_build(), |_| {
            storage_called.set(true);
            Err(())
        })
        .expect_err("event and byte limits are both exceeded");
    assert_eq!(
        error,
        NativeTraceErrorKindV1::LimitExceeded(NativeTraceLimitKindV1::Events)
    );
    assert!(!storage_called.get());

    let mut bytes = NativeTraceV1::with_capacity_for_test(NativeTraceCapacityV1::new(1, 191));
    let error = bytes
        .record_with_reserver_for_test(SchedulerTick::new(0), observed_build(), |_| {
            storage_called.set(true);
            Err(())
        })
        .expect_err("the byte bound must precede storage");
    assert_eq!(
        error,
        NativeTraceErrorKindV1::LimitExceeded(NativeTraceLimitKindV1::AccountedBytes)
    );
    assert!(!storage_called.get());

    let mut storage = NativeTraceV1::new();
    let error = storage
        .record_with_reserver_for_test(SchedulerTick::new(0), observed_build(), |additional| {
            assert_eq!(additional, 1);
            Err(())
        })
        .expect_err("storage rejection must be typed");
    assert_eq!(error, NativeTraceErrorKindV1::Storage);
    assert!(storage.events().is_empty());
}

#[test]
fn storage_failure_preserves_dense_sequence_and_tick_prefix() {
    let mut trace = NativeTraceV1::new();
    trace
        .record(SchedulerTick::new(7), observed_build())
        .expect("prefix should fit");
    let prefix = trace.events().to_vec();

    assert_eq!(
        trace
            .record_with_reserver_for_test(SchedulerTick::new(8), observed_build(), |_| Err(()))
            .expect_err("storage should fail"),
        NativeTraceErrorKindV1::Storage
    );
    assert_eq!(trace.events(), prefix);
    assert_eq!(trace.accounted_bytes(), NativeTraceEventV1::ACCOUNTED_BYTES);

    trace
        .record(SchedulerTick::new(8), observed_build())
        .expect("retry should reuse the unconsumed sequence");
    assert_eq!(trace.events()[1].sequence(), 1);
    assert_eq!(trace.events()[1].tick(), SchedulerTick::new(8));
}

#[test]
fn ticks_are_nondecreasing_and_regression_is_atomic() {
    let mut trace = NativeTraceV1::new();
    trace
        .record(SchedulerTick::new(5), observed_build())
        .expect("first tick should fit");
    trace
        .record(SchedulerTick::new(5), observed_build())
        .expect("same-turn observations share a tick");
    let prefix = trace.events().to_vec();

    assert_eq!(
        trace
            .record(SchedulerTick::new(4), observed_build())
            .expect_err("tick regression must fail"),
        NativeTraceErrorKindV1::TickRegression
    );
    assert_eq!(trace.events(), prefix);

    trace
        .record(SchedulerTick::new(6), observed_build())
        .expect("a later tick should still append");
    assert_eq!(trace.events()[2].sequence(), 2);
}

fn observed_build() -> NativeTraceStepV1 {
    trace_step(
        NativeTraceStageV1::Manifest,
        NativeObservationV1::Build,
        NativeOutcomeV1::Observed,
    )
}
