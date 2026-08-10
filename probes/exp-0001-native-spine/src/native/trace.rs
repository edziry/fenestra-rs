mod event;
mod types;

use std::fmt;

use fenestra_ui_runtime::prototype::SchedulerTick;

pub(super) use event::NativeTraceEventV1;
pub(super) use types::{
    NativeFailureCauseV1, NativeObservationV1, NativeOutcomeV1, NativeTraceLaneStatsV1,
    NativeTracePendingV1, NativeTraceStageV1, NativeTraceStepV1, NativeTraceSubmissionV1,
};

use event::validate_step_v1;

const MAX_EVENTS: usize = 128;
const MAX_ACCOUNTED_BYTES: usize = 24_576;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NativeTraceLimitKindV1 {
    Events,
    AccountedBytes,
}

impl NativeTraceLimitKindV1 {
    pub(super) const ALL: [Self; 2] = [Self::Events, Self::AccountedBytes];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NativeTraceErrorKindV1 {
    LimitExceeded(NativeTraceLimitKindV1),
    InvalidApplicability,
    TickRegression,
    Storage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct NativeTraceCapacityV1 {
    max_events: usize,
    max_accounted_bytes: usize,
}

impl NativeTraceCapacityV1 {
    pub(super) const fn new(max_events: usize, max_accounted_bytes: usize) -> Self {
        Self {
            max_events,
            max_accounted_bytes,
        }
    }
}

pub(super) struct NativeTraceV1 {
    capacity: NativeTraceCapacityV1,
    events: Vec<NativeTraceEventV1>,
    accounted_bytes: usize,
    last_tick: Option<SchedulerTick>,
    next_sequence: u64,
    next_scheduler_turn: u64,
}

impl NativeTraceV1 {
    pub(super) const fn new() -> Self {
        Self {
            capacity: NativeTraceCapacityV1::new(MAX_EVENTS, MAX_ACCOUNTED_BYTES),
            events: Vec::new(),
            accounted_bytes: 0,
            last_tick: None,
            next_sequence: 0,
            next_scheduler_turn: 0,
        }
    }

    #[cfg(test)]
    pub(super) const fn with_capacity_for_test(capacity: NativeTraceCapacityV1) -> Self {
        Self {
            capacity,
            events: Vec::new(),
            accounted_bytes: 0,
            last_tick: None,
            next_sequence: 0,
            next_scheduler_turn: 0,
        }
    }

    pub(super) const fn capacity(&self) -> NativeTraceCapacityV1 {
        self.capacity
    }

    pub(super) const fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub(super) const fn len(&self) -> usize {
        self.events.len()
    }

    pub(super) const fn accounted_bytes(&self) -> usize {
        self.accounted_bytes
    }

    pub(super) fn events(&self) -> &[NativeTraceEventV1] {
        &self.events
    }

    pub(super) fn record(
        &mut self,
        tick: SchedulerTick,
        step: NativeTraceStepV1,
    ) -> Result<(), NativeTraceErrorKindV1> {
        self.preflight(tick, step)?;
        self.events
            .try_reserve(1)
            .map_err(|_| NativeTraceErrorKindV1::Storage)?;
        self.append(tick, step)
    }

    #[cfg(test)]
    pub(super) fn record_with_reserver_for_test<F>(
        &mut self,
        tick: SchedulerTick,
        step: NativeTraceStepV1,
        reserver: F,
    ) -> Result<(), NativeTraceErrorKindV1>
    where
        F: FnOnce(usize) -> Result<(), ()>,
    {
        self.preflight(tick, step)?;
        reserver(1).map_err(|()| NativeTraceErrorKindV1::Storage)?;
        self.append(tick, step)
    }

    fn preflight(
        &self,
        tick: SchedulerTick,
        step: NativeTraceStepV1,
    ) -> Result<(), NativeTraceErrorKindV1> {
        if !validate_step_v1(step, self.next_scheduler_turn) {
            return Err(NativeTraceErrorKindV1::InvalidApplicability);
        }
        if self.last_tick.is_some_and(|last| tick < last) {
            return Err(NativeTraceErrorKindV1::TickRegression);
        }
        let next_events =
            self.events
                .len()
                .checked_add(1)
                .ok_or(NativeTraceErrorKindV1::LimitExceeded(
                    NativeTraceLimitKindV1::Events,
                ))?;
        if next_events > self.capacity.max_events {
            return Err(NativeTraceErrorKindV1::LimitExceeded(
                NativeTraceLimitKindV1::Events,
            ));
        }
        let next_bytes = self
            .accounted_bytes
            .checked_add(NativeTraceEventV1::ACCOUNTED_BYTES)
            .ok_or(NativeTraceErrorKindV1::LimitExceeded(
                NativeTraceLimitKindV1::AccountedBytes,
            ))?;
        if next_bytes > self.capacity.max_accounted_bytes {
            return Err(NativeTraceErrorKindV1::LimitExceeded(
                NativeTraceLimitKindV1::AccountedBytes,
            ));
        }
        Ok(())
    }

    fn append(
        &mut self,
        tick: SchedulerTick,
        step: NativeTraceStepV1,
    ) -> Result<(), NativeTraceErrorKindV1> {
        let next_sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or(NativeTraceErrorKindV1::InvalidApplicability)?;
        let next_scheduler_turn = if step.stage == NativeTraceStageV1::Scheduler {
            self.next_scheduler_turn
                .checked_add(1)
                .ok_or(NativeTraceErrorKindV1::InvalidApplicability)?
        } else {
            self.next_scheduler_turn
        };
        let next_bytes = self
            .accounted_bytes
            .checked_add(NativeTraceEventV1::ACCOUNTED_BYTES)
            .ok_or(NativeTraceErrorKindV1::LimitExceeded(
                NativeTraceLimitKindV1::AccountedBytes,
            ))?;
        self.events
            .push(NativeTraceEventV1::new(self.next_sequence, tick, step));
        self.accounted_bytes = next_bytes;
        self.last_tick = Some(tick);
        self.next_sequence = next_sequence;
        self.next_scheduler_turn = next_scheduler_turn;
        Ok(())
    }
}

impl fmt::Debug for NativeTraceV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NativeTraceV1")
            .field("event_count", &self.events.len())
            .field("accounted_bytes", &self.accounted_bytes)
            .finish()
    }
}
