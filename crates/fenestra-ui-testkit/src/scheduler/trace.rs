mod event;
mod types;

use fenestra_ui_runtime::prototype::{QueueStats, SchedulerTick, UiScheduler};

use self::event::project_event;
use super::{FakeClockDomainV1, FakeClockV1, FakeRendererV1};

pub use event::{SchedulerTraceEventV1, SchedulerTraceLaneStatsV1, SchedulerTraceRendererStatsV1};
pub use types::{
    SchedulerTraceActionV1, SchedulerTraceCallbackOutcomeV1, SchedulerTraceCapacityV1,
    SchedulerTraceCommitOutcomeV1, SchedulerTraceErrorKindV1, SchedulerTraceErrorV1,
    SchedulerTraceInputOutcomeV1, SchedulerTraceLimitV1, SchedulerTraceStageV1,
    SchedulerTraceStepV1,
};

/// Bounded deterministic observations from one scheduler experiment.
pub struct SchedulerTraceV1 {
    domain: FakeClockDomainV1,
    capacity: SchedulerTraceCapacityV1,
    events: Vec<SchedulerTraceEventV1>,
    accounted_bytes: usize,
    next_sequence: Option<u64>,
    last_tick: Option<SchedulerTick>,
}

impl SchedulerTraceV1 {
    /// Creates an empty trace bound to one explicit fake-clock domain.
    #[must_use]
    pub const fn new(domain: FakeClockDomainV1, capacity: SchedulerTraceCapacityV1) -> Self {
        Self {
            domain,
            capacity,
            events: Vec::new(),
            accounted_bytes: 0,
            next_sequence: Some(0),
            last_tick: None,
        }
    }

    /// Records one complete post-transition observation atomically.
    pub fn record(
        &mut self,
        clock: &FakeClockV1,
        step: SchedulerTraceStepV1,
        scheduler: &UiScheduler,
        renderer: &FakeRendererV1,
    ) -> Result<(), SchedulerTraceErrorV1> {
        let tick = self.validate_clock(*clock)?;
        let (next_len, next_bytes) = self.preflight_capacity()?;
        let sequence = self
            .next_sequence
            .ok_or_else(SchedulerTraceErrorV1::arithmetic_exhausted)?;
        let event = project_event(sequence, self.domain, tick, step, scheduler, renderer)?;
        let next_sequence = sequence.checked_add(1);

        self.events.push(event);
        debug_assert_eq!(self.events.len(), next_len);
        self.accounted_bytes = next_bytes;
        self.next_sequence = next_sequence;
        self.last_tick = Some(tick);
        Ok(())
    }

    /// Returns the fixed clock domain required by this trace.
    #[must_use]
    pub const fn domain(&self) -> FakeClockDomainV1 {
        self.domain
    }

    /// Returns the declared event and byte bounds.
    #[must_use]
    pub const fn capacity(&self) -> SchedulerTraceCapacityV1 {
        self.capacity
    }

    /// Returns the accepted deterministic event prefix.
    #[must_use]
    pub fn events(&self) -> &[SchedulerTraceEventV1] {
        &self.events
    }

    /// Returns the accepted event count.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.events.len()
    }

    /// Reports whether the trace contains no accepted events.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Returns the protocol-accounted bytes for the accepted prefix.
    #[must_use]
    pub const fn accounted_bytes(&self) -> usize {
        self.accounted_bytes
    }

    fn validate_clock(&self, clock: FakeClockV1) -> Result<SchedulerTick, SchedulerTraceErrorV1> {
        if clock.domain() != self.domain {
            return Err(SchedulerTraceErrorV1::new(
                SchedulerTraceErrorKindV1::ClockDomainMismatch,
            ));
        }
        let tick = clock.now();
        if self.last_tick.is_some_and(|last| tick < last) {
            return Err(SchedulerTraceErrorV1::new(
                SchedulerTraceErrorKindV1::TickRegression,
            ));
        }
        Ok(tick)
    }

    fn preflight_capacity(&self) -> Result<(usize, usize), SchedulerTraceErrorV1> {
        let next_len = self
            .events
            .len()
            .checked_add(1)
            .ok_or_else(SchedulerTraceErrorV1::arithmetic_exhausted)?;
        let next_bytes = self
            .accounted_bytes
            .checked_add(SchedulerTraceEventV1::ACCOUNTED_BYTES)
            .ok_or_else(SchedulerTraceErrorV1::arithmetic_exhausted)?;

        if next_len > self.capacity.max_events() {
            return Err(SchedulerTraceErrorV1::new(
                SchedulerTraceErrorKindV1::LimitExceeded(SchedulerTraceLimitV1::Events),
            ));
        }
        if next_bytes > self.capacity.max_bytes() {
            return Err(SchedulerTraceErrorV1::new(
                SchedulerTraceErrorKindV1::LimitExceeded(SchedulerTraceLimitV1::AccountedBytes),
            ));
        }
        Ok((next_len, next_bytes))
    }
}

pub(super) fn oldest_residence_ticks(
    stats: QueueStats,
    tick: SchedulerTick,
) -> Result<Option<u64>, SchedulerTraceErrorV1> {
    stats
        .earliest_tick()
        .map(|earliest| {
            tick.get().checked_sub(earliest.get()).ok_or_else(|| {
                SchedulerTraceErrorV1::new(SchedulerTraceErrorKindV1::TickRegression)
            })
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accounted_byte_overflow_preserves_the_empty_prefix() {
        let mut trace = SchedulerTraceV1::new(
            FakeClockDomainV1::new(0),
            SchedulerTraceCapacityV1::new(usize::MAX, usize::MAX),
        );
        trace.accounted_bytes = usize::MAX;

        let error = trace
            .preflight_capacity()
            .expect_err("fixed event accounting should use checked addition");

        assert_eq!(error.kind(), SchedulerTraceErrorKindV1::ArithmeticExhausted);
        assert!(trace.events().is_empty());
        assert_eq!(trace.accounted_bytes(), usize::MAX);
        assert_eq!(trace.next_sequence, Some(0));
        assert_eq!(trace.last_tick, None);
    }
}
