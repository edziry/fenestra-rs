use fenestra_ui_runtime::prototype::{
    ControlAdmission, ControlSequence, SchedulerInput, SchedulerInputResult, SchedulerTick,
};
use fenestra_ui_testkit::prototype::{
    HeadlessPointerTargetV1, NodePathV1, PathSegmentV1, observe_headless_projection_v1,
};

use super::super::trace::{
    NativeFailureCauseV1, NativeInputSourceV1, NativeObservationV1, NativeOutcomeV1,
    NativeTraceStageV1, NativeTraceStepV1,
};
use super::super::types::NativePhysicalPointV1;
use super::state::{NativeDriverV1, map_contract_error};
use super::types::PresenterPortV1;

impl<P: PresenterPortV1> NativeDriverV1<P> {
    pub(crate) fn cursor_moved(
        &mut self,
        physical: NativePhysicalPointV1,
        source: NativeInputSourceV1,
        tick: SchedulerTick,
    ) -> Result<(), NativeFailureCauseV1> {
        if !physical.is_finite() {
            return Err(NativeFailureCauseV1::InvalidPoint);
        }
        self.reserve_trace(tick, 1, 0)?;
        let mut step = NativeTraceStepV1::new(
            NativeTraceStageV1::Platform,
            NativeObservationV1::Pointer,
            NativeOutcomeV1::Coalesced,
        );
        step.input_source = Some(source);
        self.record_pointer_draft(tick, step, true)?;
        self.pending_pointer = Some(physical);
        Ok(())
    }

    pub(crate) fn pointer_pressed(
        &mut self,
        source: NativeInputSourceV1,
        tick: SchedulerTick,
    ) -> Result<HeadlessPointerTargetV1, NativeFailureCauseV1> {
        let physical = self
            .pending_pointer
            .ok_or(NativeFailureCauseV1::Invariant)?;
        let surface = self
            .surface
            .input_tuple()
            .ok_or(NativeFailureCauseV1::Invariant)?;
        let point = surface
            .scale()
            .logical_point(physical)
            .map_err(map_contract_error)?;
        let snapshot = self.scheduler.committed();
        let observed = observe_headless_projection_v1(&self.fixture, &snapshot)
            .map_err(|_| NativeFailureCauseV1::Oracle)?;
        if observed.generation() != snapshot.generation()
            || observed.projection().surface() != surface.logical_surface()
        {
            return Err(NativeFailureCauseV1::Invariant);
        }
        let target = observed
            .projection()
            .hit_regions()
            .iter()
            .rev()
            .find(|record| contains(record.clip(), point.x(), point.y()))
            .map_or(HeadlessPointerTargetV1::None, |record| {
                pointer_target(record.path())
            });
        self.reserve_trace(tick, 1, 0)?;
        let mut step = NativeTraceStepV1::new(
            NativeTraceStageV1::Platform,
            NativeObservationV1::Pointer,
            NativeOutcomeV1::Observed,
        );
        step.captured_generation = Some(snapshot.generation());
        step.input_source = Some(source);
        step.surface = Some(surface);
        step.target = Some(target);
        self.record_pointer_draft(tick, step, false)?;
        self.pending_pointer = None;
        Ok(target)
    }

    pub(crate) fn close_requested(
        &mut self,
        source: NativeInputSourceV1,
        tick: SchedulerTick,
    ) -> Result<ControlAdmission, NativeFailureCauseV1> {
        if let Some(control) = self.shutdown_control {
            self.reserve_trace(tick, 1, 0)?;
            let mut duplicate = NativeTraceStepV1::new(
                NativeTraceStageV1::Platform,
                NativeObservationV1::Close,
                NativeOutcomeV1::Coalesced,
            );
            duplicate.input_source = Some(source);
            self.record(tick, duplicate)?;
            return Ok(ControlAdmission::AlreadyAccepted(control));
        }
        self.reserve_trace(tick, 2, 1)?;
        self.surface.discard_pending();
        self.redraw_armed = false;
        let mut observed = NativeTraceStepV1::new(
            NativeTraceStageV1::Platform,
            NativeObservationV1::Close,
            NativeOutcomeV1::Observed,
        );
        observed.input_source = Some(source);
        self.record_pointer_draft(tick, observed, false)?;
        self.pending_pointer = None;
        let SchedulerInputResult::Control(admission) = self
            .scheduler
            .process_input(SchedulerInput::RequestShutdown, tick)
            .map_err(|_| NativeFailureCauseV1::Scheduler)?
        else {
            return Err(NativeFailureCauseV1::Invariant);
        };
        let control = admission_sequence(admission);
        self.shutdown_control = Some(control);
        let mut step = NativeTraceStepV1::new(
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Shutdown,
            NativeOutcomeV1::Accepted,
        );
        step.control = Some(control.get());
        self.record_scheduler(tick, step)?;
        Ok(admission)
    }
}

const fn admission_sequence(admission: ControlAdmission) -> ControlSequence {
    match admission {
        ControlAdmission::Accepted(control) | ControlAdmission::AlreadyAccepted(control) => control,
    }
}

fn pointer_target(path: &NodePathV1) -> HeadlessPointerTargetV1 {
    if path == &NodePathV1::root().static_child(0).static_child(0) {
        return HeadlessPointerTargetV1::StaticControl;
    }
    match path.segments().last() {
        Some(PathSegmentV1::Member { key, .. }) => HeadlessPointerTargetV1::Key(*key),
        _ => HeadlessPointerTargetV1::None,
    }
}

fn contains(rect: fenestra_ui_runtime::prototype::HeadlessRect, x: i32, y: i32) -> bool {
    if rect.width() <= 0 || rect.height() <= 0 {
        return false;
    }
    rect.x()
        .checked_add(rect.width())
        .is_some_and(|right| rect.x() <= x && x < right)
        && rect
            .y()
            .checked_add(rect.height())
            .is_some_and(|bottom| rect.y() <= y && y < bottom)
}
