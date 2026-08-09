mod event;
mod types;

use fenestra_ui_runtime::prototype::{SchedulerTick, UiScheduler};

use crate::scheduler::{FakeClockDomainV1, FakeClockV1, FakeRendererV1};

pub use event::HeadlessTraceEventV1;
pub use types::{
    HeadlessFailureCauseV1, HeadlessInputKindV1, HeadlessOutcomeV1, HeadlessTraceCapacityV1,
    HeadlessTraceProjectionCountsV1, HeadlessTraceQueueStatsV1, HeadlessTraceRendererStatsV1,
    HeadlessTraceStageV1,
};
pub(super) use types::{HeadlessTraceError, HeadlessTraceErrorKind, HeadlessTraceStep};

/// Bounded deterministic observations from the fixed headless experiment.
#[derive(Debug, Eq, PartialEq)]
pub struct HeadlessTraceV1 {
    domain: FakeClockDomainV1,
    capacity: HeadlessTraceCapacityV1,
    events: Vec<HeadlessTraceEventV1>,
    accounted_bytes: usize,
    next_sequence: Option<u64>,
    last_tick: Option<SchedulerTick>,
}

impl HeadlessTraceV1 {
    pub(super) const fn new(domain: FakeClockDomainV1, capacity: HeadlessTraceCapacityV1) -> Self {
        Self {
            domain,
            capacity,
            events: Vec::new(),
            accounted_bytes: 0,
            next_sequence: Some(0),
            last_tick: None,
        }
    }

    pub(super) fn record(
        &mut self,
        clock: &FakeClockV1,
        step: HeadlessTraceStep,
        scheduler: &UiScheduler,
        renderer: &FakeRendererV1,
    ) -> Result<(), HeadlessTraceError> {
        let tick = self.validate_clock(*clock)?;
        let (next_len, next_bytes) = self.preflight_capacity()?;
        let sequence = self
            .next_sequence
            .ok_or_else(HeadlessTraceError::arithmetic_exhausted)?;
        let event = event::project_event(sequence, self.domain, tick, step, scheduler, renderer)?;
        let next_sequence = sequence.checked_add(1);

        self.events.push(event);
        debug_assert_eq!(self.events.len(), next_len);
        self.accounted_bytes = next_bytes;
        self.next_sequence = next_sequence;
        self.last_tick = Some(tick);
        Ok(())
    }

    /// Returns the fixed fake-clock domain shared with the scheduler trace.
    #[must_use]
    pub const fn domain(&self) -> FakeClockDomainV1 {
        self.domain
    }

    /// Returns the inclusive event and accounted-byte ceilings.
    #[must_use]
    pub const fn capacity(&self) -> HeadlessTraceCapacityV1 {
        self.capacity
    }

    /// Returns the accepted deterministic event prefix.
    #[must_use]
    pub fn events(&self) -> &[HeadlessTraceEventV1] {
        &self.events
    }

    /// Returns the number of accepted events.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.events.len()
    }

    /// Reports whether no event has been accepted.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Returns protocol-accounted bytes for the accepted prefix.
    #[must_use]
    pub const fn accounted_bytes(&self) -> usize {
        self.accounted_bytes
    }

    fn validate_clock(&self, clock: FakeClockV1) -> Result<SchedulerTick, HeadlessTraceError> {
        if clock.domain() != self.domain {
            return Err(HeadlessTraceError::new(
                HeadlessTraceErrorKind::ClockDomainMismatch,
            ));
        }
        let tick = clock.now();
        if self.last_tick.is_some_and(|last| tick < last) {
            return Err(HeadlessTraceError::new(
                HeadlessTraceErrorKind::TickRegression,
            ));
        }
        Ok(tick)
    }

    fn preflight_capacity(&self) -> Result<(usize, usize), HeadlessTraceError> {
        let next_len = self
            .events
            .len()
            .checked_add(1)
            .ok_or_else(HeadlessTraceError::arithmetic_exhausted)?;
        let next_bytes = self
            .accounted_bytes
            .checked_add(HeadlessTraceEventV1::ACCOUNTED_BYTES)
            .ok_or_else(HeadlessTraceError::arithmetic_exhausted)?;
        if next_len > self.capacity.max_events() {
            return Err(HeadlessTraceError::new(
                HeadlessTraceErrorKind::EventLimitExceeded,
            ));
        }
        if next_bytes > self.capacity.max_bytes() {
            return Err(HeadlessTraceError::new(
                HeadlessTraceErrorKind::ByteLimitExceeded,
            ));
        }
        Ok((next_len, next_bytes))
    }
}

#[cfg(test)]
mod tests;
