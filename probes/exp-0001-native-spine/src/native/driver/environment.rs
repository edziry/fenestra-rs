use fenestra_ui_runtime::prototype::SchedulerTick;

use super::super::trace::{
    NativeFailureCauseV1, NativeObservationV1, NativeOutcomeV1, NativeTraceStageV1,
    NativeTraceStepV1,
};
use super::state::NativeDriverV1;
use super::types::PresenterPortV1;

impl<P: PresenterPortV1> NativeDriverV1<P> {
    pub(crate) fn reject_environment_surface_before_redraw(
        &mut self,
        tick: SchedulerTick,
    ) -> Result<(), NativeFailureCauseV1> {
        self.reject_environment_surface_change(tick, true)
    }

    pub(crate) fn reject_environment_surface_between_directives(
        &mut self,
        tick: SchedulerTick,
    ) -> Result<(), NativeFailureCauseV1> {
        self.reject_environment_surface_change(tick, false)
    }

    fn reject_environment_surface_change(
        &mut self,
        tick: SchedulerTick,
        redraw_observed: bool,
    ) -> Result<(), NativeFailureCauseV1> {
        let observed = self
            .surface
            .pending_tuple()
            .or(self.surface.accepted_tuple())
            .ok_or(NativeFailureCauseV1::Invariant)?;
        self.reserve_trace(tick, if redraw_observed { 2 } else { 1 }, 0)?;
        if redraw_observed {
            let mut ignored = NativeTraceStepV1::new(
                NativeTraceStageV1::Platform,
                NativeObservationV1::Redraw,
                NativeOutcomeV1::Ignored,
            );
            ignored.surface = self.surface.accepted_tuple();
            self.record(tick, ignored)?;
        }
        let cause = NativeFailureCauseV1::EnvironmentSurfaceChanged;
        let mut failed = NativeTraceStepV1::new(
            NativeTraceStageV1::Platform,
            NativeObservationV1::Surface,
            NativeOutcomeV1::Failed(cause),
        );
        failed.surface = Some(observed);
        self.record(tick, failed)?;
        self.surface.discard_pending();
        Err(cause)
    }
}
