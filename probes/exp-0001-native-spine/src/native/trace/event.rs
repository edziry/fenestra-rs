use fenestra_ui_runtime::prototype::{RuntimeGeneration, SchedulerTick};
use fenestra_ui_testkit::prototype::HeadlessPointerTargetV1;

use super::super::surface::NativeSurfaceTupleV1;
use super::types::{
    NativeFailureCauseV1, NativeObservationV1, NativeOutcomeV1, NativeTraceLaneStatsV1,
    NativeTracePendingV1, NativeTraceStageV1, NativeTraceStepV1, NativeTraceSubmissionV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeTraceEventV1 {
    schema_revision: u16,
    sequence: u64,
    tick: SchedulerTick,
    step: NativeTraceStepV1,
}

impl NativeTraceEventV1 {
    pub(crate) const ACCOUNTED_BYTES: usize = 192;

    pub(crate) const fn new(sequence: u64, tick: SchedulerTick, step: NativeTraceStepV1) -> Self {
        Self {
            schema_revision: 1,
            sequence,
            tick,
            step,
        }
    }

    pub(crate) const fn schema_revision(self) -> u16 {
        self.schema_revision
    }

    pub(crate) const fn sequence(self) -> u64 {
        self.sequence
    }

    pub(crate) const fn tick(self) -> SchedulerTick {
        self.tick
    }

    pub(crate) const fn stage(self) -> NativeTraceStageV1 {
        self.step.stage
    }

    pub(crate) const fn observation(self) -> NativeObservationV1 {
        self.step.observation
    }

    pub(crate) const fn outcome(self) -> NativeOutcomeV1 {
        self.step.outcome
    }

    pub(crate) const fn scheduler_turn(self) -> Option<u64> {
        self.step.scheduler_turn
    }

    pub(crate) const fn captured_generation(self) -> Option<RuntimeGeneration> {
        self.step.captured_generation
    }

    pub(crate) const fn published_generation(self) -> Option<RuntimeGeneration> {
        self.step.published_generation
    }

    pub(crate) const fn surface(self) -> Option<NativeSurfaceTupleV1> {
        self.step.surface
    }

    pub(crate) const fn target(self) -> Option<HeadlessPointerTargetV1> {
        self.step.target
    }

    pub(crate) const fn frame(self) -> Option<u64> {
        self.step.frame
    }

    pub(crate) const fn submission(self) -> Option<NativeTraceSubmissionV1> {
        self.step.submission
    }

    pub(crate) const fn control(self) -> Option<u64> {
        self.step.control
    }

    pub(crate) const fn redraw_armed(self) -> bool {
        self.step.redraw_armed
    }

    pub(crate) const fn pending(self) -> NativeTracePendingV1 {
        self.step.pending
    }

    pub(crate) const fn deferred(self) -> NativeTraceLaneStatsV1 {
        self.step.deferred
    }

    pub(crate) const fn controls(self) -> NativeTraceLaneStatsV1 {
        self.step.controls
    }

    pub(crate) const fn visual(self) -> NativeTraceLaneStatsV1 {
        self.step.visual
    }

    pub(crate) const fn in_flight(self) -> NativeTraceLaneStatsV1 {
        self.step.in_flight
    }
}

pub(crate) fn validate_step_v1(step: NativeTraceStepV1, expected_scheduler_turn: u64) -> bool {
    if !step.pending.is_bounded()
        || !scheduler_turn_is_valid(step, expected_scheduler_turn)
        || !stage_tuple_is_valid(step)
        || !identity_shape_is_valid(step)
    {
        return false;
    }
    if step.target.is_some() != (step.observation == NativeObservationV1::Pointer) {
        return false;
    }
    let capture_required = step.observation == NativeObservationV1::Pointer
        || (step.observation == NativeObservationV1::Surface
            && step.outcome == NativeOutcomeV1::Deferred);
    if step.captured_generation.is_some() != capture_required {
        return false;
    }
    let publication_required = matches!(
        step.outcome,
        NativeOutcomeV1::Published | NativeOutcomeV1::Matched
    );
    if step.published_generation.is_some() != publication_required {
        return false;
    }
    if surface_is_required(step) != step.surface.is_some() {
        return false;
    }
    true
}

fn stage_tuple_is_valid(step: NativeTraceStepV1) -> bool {
    match step.stage {
        NativeTraceStageV1::Manifest => {
            step.observation == NativeObservationV1::Build
                && step.outcome == NativeOutcomeV1::Observed
        }
        NativeTraceStageV1::Shell => matches!(
            step.observation,
            NativeObservationV1::Resumed
                | NativeObservationV1::Close
                | NativeObservationV1::Timeout
        ),
        NativeTraceStageV1::Platform => matches!(
            step.observation,
            NativeObservationV1::Surface
                | NativeObservationV1::Scale
                | NativeObservationV1::Pointer
                | NativeObservationV1::Redraw
                | NativeObservationV1::Close
        ),
        NativeTraceStageV1::Scheduler => matches!(
            step.observation,
            NativeObservationV1::Surface
                | NativeObservationV1::Frame
                | NativeObservationV1::Present
                | NativeObservationV1::Completion
                | NativeObservationV1::Shutdown
        ),
        NativeTraceStageV1::Renderer => matches!(
            step.observation,
            NativeObservationV1::Frame
                | NativeObservationV1::Present
                | NativeObservationV1::Completion
        ),
        NativeTraceStageV1::Oracle => {
            step.observation == NativeObservationV1::Surface
                && matches!(
                    step.outcome,
                    NativeOutcomeV1::Matched | NativeOutcomeV1::Failed(_)
                )
        }
    }
}

fn scheduler_turn_is_valid(step: NativeTraceStepV1, expected: u64) -> bool {
    if step.stage == NativeTraceStageV1::Scheduler {
        step.scheduler_turn == Some(expected)
    } else {
        step.scheduler_turn.is_none()
    }
}

fn identity_shape_is_valid(step: NativeTraceStepV1) -> bool {
    match (step.stage, step.observation, step.outcome) {
        (NativeTraceStageV1::Renderer, NativeObservationV1::Frame, NativeOutcomeV1::Accepted) => {
            step.frame.is_some() && step.submission.is_some() && step.control.is_none()
        }
        (NativeTraceStageV1::Renderer, NativeObservationV1::Frame, NativeOutcomeV1::Rejected)
        | (NativeTraceStageV1::Scheduler, NativeObservationV1::Frame, NativeOutcomeV1::Offered) => {
            step.frame.is_some() && step.submission.is_none() && step.control.is_none()
        }
        (
            NativeTraceStageV1::Renderer,
            NativeObservationV1::Present,
            NativeOutcomeV1::Completed,
        ) => step.frame.is_some() && step.submission.is_some() && step.control.is_none(),
        (
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Present,
            NativeOutcomeV1::Accepted,
        ) => step.frame.is_some() && step.submission.is_some() && step.control.is_some(),
        (
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Completion,
            NativeOutcomeV1::Accepted,
        ) => step.frame.is_none() && step.submission.is_some() && step.control.is_some(),
        (
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Shutdown,
            NativeOutcomeV1::Accepted | NativeOutcomeV1::Stopped,
        ) => step.frame.is_none() && step.submission.is_none() && step.control.is_some(),
        _ => step.frame.is_none() && step.submission.is_none() && step.control.is_none(),
    }
}

fn surface_is_required(step: NativeTraceStepV1) -> bool {
    matches!(
        step.observation,
        NativeObservationV1::Pointer
            | NativeObservationV1::Redraw
            | NativeObservationV1::Frame
            | NativeObservationV1::Present
            | NativeObservationV1::Completion
    ) || matches!(
        step.outcome,
        NativeOutcomeV1::Deferred | NativeOutcomeV1::Published | NativeOutcomeV1::Matched
    ) || is_environment_failure(step.outcome)
}

fn is_environment_failure(outcome: NativeOutcomeV1) -> bool {
    matches!(
        outcome,
        NativeOutcomeV1::Failed(
            NativeFailureCauseV1::EnvironmentScaleChanged
                | NativeFailureCauseV1::SurfaceRepaintUnavailable
        )
    )
}
