use fenestra_ui_runtime::prototype::{QueueStats, SchedulerTick, UiScheduler};

use super::super::surface::NativeSurfaceStateV1;
use super::super::trace::{
    NativeFailureCauseV1, NativeTraceErrorKindV1, NativeTraceLaneStatsV1, NativeTracePendingV1,
    NativeTraceStepV1, NativeTraceV1,
};
use super::state::NativeDriverV1;

impl<P> NativeDriverV1<P> {
    pub(super) fn reserve_trace(
        &mut self,
        tick: SchedulerTick,
        event_count: usize,
        scheduler_event_count: u64,
    ) -> Result<(), NativeFailureCauseV1> {
        self.trace
            .reserve_batch(tick, event_count, scheduler_event_count)
            .map_err(map_trace_error)
    }
    pub(super) fn record(
        &mut self,
        tick: SchedulerTick,
        mut step: NativeTraceStepV1,
    ) -> Result<(), NativeFailureCauseV1> {
        populate_step(
            &self.scheduler,
            &self.surface,
            self.pending_pointer.is_some(),
            self.redraw_armed,
            self.presenter_pending,
            &mut step,
        );
        record(&mut self.trace, tick, step)
    }

    pub(super) fn record_surface_draft(
        &mut self,
        tick: SchedulerTick,
        mut step: NativeTraceStepV1,
        surface: &NativeSurfaceStateV1,
    ) -> Result<(), NativeFailureCauseV1> {
        populate_step(
            &self.scheduler,
            surface,
            self.pending_pointer.is_some(),
            self.redraw_armed,
            self.presenter_pending,
            &mut step,
        );
        record(&mut self.trace, tick, step)
    }

    pub(super) fn record_pointer_draft(
        &mut self,
        tick: SchedulerTick,
        mut step: NativeTraceStepV1,
        pointer_pending: bool,
    ) -> Result<(), NativeFailureCauseV1> {
        populate_step(
            &self.scheduler,
            &self.surface,
            pointer_pending,
            self.redraw_armed,
            self.presenter_pending,
            &mut step,
        );
        record(&mut self.trace, tick, step)
    }

    pub(super) fn record_scheduler(
        &mut self,
        tick: SchedulerTick,
        mut step: NativeTraceStepV1,
    ) -> Result<(), NativeFailureCauseV1> {
        let next_scheduler_turn = self
            .scheduler_turn
            .checked_add(1)
            .ok_or(NativeFailureCauseV1::Trace)?;
        step.scheduler_turn = Some(self.scheduler_turn);
        self.record(tick, step)?;
        self.scheduler_turn = next_scheduler_turn;
        Ok(())
    }
}

pub(super) struct SchedulerSnapshotRecorderV1<'a> {
    trace: &'a mut NativeTraceV1,
    scheduler: &'a UiScheduler,
    scheduler_turn: &'a mut u64,
    pending: NativeTracePendingV1,
    redraw_armed: bool,
}

impl<'a> SchedulerSnapshotRecorderV1<'a> {
    pub(super) const fn new(
        trace: &'a mut NativeTraceV1,
        scheduler: &'a UiScheduler,
        scheduler_turn: &'a mut u64,
        pending: NativeTracePendingV1,
        redraw_armed: bool,
    ) -> Self {
        Self {
            trace,
            scheduler,
            scheduler_turn,
            pending,
            redraw_armed,
        }
    }

    pub(super) fn record(
        self,
        tick: SchedulerTick,
        mut step: NativeTraceStepV1,
    ) -> Result<(), NativeFailureCauseV1> {
        let next_scheduler_turn = self
            .scheduler_turn
            .checked_add(1)
            .ok_or(NativeFailureCauseV1::Trace)?;
        step.scheduler_turn = Some(*self.scheduler_turn);
        populate_snapshot(self.scheduler, self.pending, self.redraw_armed, &mut step);
        record(self.trace, tick, step)?;
        *self.scheduler_turn = next_scheduler_turn;
        Ok(())
    }
}

fn populate_step(
    scheduler: &UiScheduler,
    surface: &NativeSurfaceStateV1,
    pointer_pending: bool,
    redraw_armed: bool,
    presenter_pending: bool,
    step: &mut NativeTraceStepV1,
) {
    let pending = NativeTracePendingV1::new(
        surface.pending_count(),
        usize::from(pointer_pending),
        usize::from(presenter_pending),
    );
    populate_snapshot(scheduler, pending, redraw_armed, step);
}

fn populate_snapshot(
    scheduler: &UiScheduler,
    pending: NativeTracePendingV1,
    redraw_armed: bool,
    step: &mut NativeTraceStepV1,
) {
    let stats = scheduler.stats();
    step.scheduler_state = Some(scheduler.state());
    step.current_generation = Some(scheduler.committed().generation());
    step.redraw_armed = redraw_armed;
    step.pending = pending;
    step.deferred = lane(stats.deferred());
    step.controls = lane(stats.controls());
    step.visual = lane(stats.visual());
    step.in_flight = lane(stats.in_flight());
}

fn record(
    trace: &mut NativeTraceV1,
    tick: SchedulerTick,
    step: NativeTraceStepV1,
) -> Result<(), NativeFailureCauseV1> {
    trace.record(tick, step).map_err(map_trace_error)
}

pub(super) const fn map_trace_error(error: NativeTraceErrorKindV1) -> NativeFailureCauseV1 {
    match error {
        NativeTraceErrorKindV1::Storage => NativeFailureCauseV1::Storage,
        NativeTraceErrorKindV1::LimitExceeded(_)
        | NativeTraceErrorKindV1::InvalidApplicability
        | NativeTraceErrorKindV1::TickRegression => NativeFailureCauseV1::Trace,
    }
}

const fn lane(stats: QueueStats) -> NativeTraceLaneStatsV1 {
    NativeTraceLaneStatsV1::new(stats.items(), stats.accounted_bytes())
}

#[cfg(test)]
mod tests {
    use super::super::super::trace::{NativeTraceErrorKindV1, NativeTraceLimitKindV1};
    use super::{NativeFailureCauseV1, map_trace_error};

    #[test]
    fn storage_is_the_only_trace_error_with_a_storage_cause() {
        assert_eq!(
            map_trace_error(NativeTraceErrorKindV1::Storage),
            NativeFailureCauseV1::Storage
        );
        for error in [
            NativeTraceErrorKindV1::LimitExceeded(NativeTraceLimitKindV1::Events),
            NativeTraceErrorKindV1::LimitExceeded(NativeTraceLimitKindV1::AccountedBytes),
            NativeTraceErrorKindV1::InvalidApplicability,
            NativeTraceErrorKindV1::TickRegression,
        ] {
            assert_eq!(map_trace_error(error), NativeFailureCauseV1::Trace);
        }
    }
}
