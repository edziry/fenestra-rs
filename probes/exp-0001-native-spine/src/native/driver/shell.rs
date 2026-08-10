use fenestra_ui_runtime::prototype::{SchedulerState, SchedulerTick};

use super::super::trace::{
    NativeFailureCauseV1, NativeObservationV1, NativeOutcomeV1, NativeTraceStageV1,
    NativeTraceStepV1,
};
use super::state::NativeDriverV1;
use super::types::PresenterPortV1;

impl<P: PresenterPortV1> NativeDriverV1<P> {
    pub(crate) fn record_shell_resumed(
        &mut self,
        tick: SchedulerTick,
    ) -> Result<(), NativeFailureCauseV1> {
        self.record_shell(
            tick,
            NativeObservationV1::Resumed,
            NativeOutcomeV1::Observed,
        )
    }

    pub(crate) fn record_shell_timeout(
        &mut self,
        tick: SchedulerTick,
    ) -> Result<(), NativeFailureCauseV1> {
        self.record_shell(
            tick,
            NativeObservationV1::Timeout,
            NativeOutcomeV1::Failed(NativeFailureCauseV1::Timeout),
        )
    }

    pub(crate) fn record_shell_close_completed(
        &mut self,
        tick: SchedulerTick,
    ) -> Result<(), NativeFailureCauseV1> {
        if self.scheduler_state() != SchedulerState::Stopped {
            return Err(NativeFailureCauseV1::Invariant);
        }
        self.record_shell(tick, NativeObservationV1::Close, NativeOutcomeV1::Completed)
    }

    fn record_shell(
        &mut self,
        tick: SchedulerTick,
        observation: NativeObservationV1,
        outcome: NativeOutcomeV1,
    ) -> Result<(), NativeFailureCauseV1> {
        self.reserve_trace(tick, 1, 0)?;
        self.record(
            tick,
            NativeTraceStepV1::new(NativeTraceStageV1::Shell, observation, outcome),
        )
    }
}
