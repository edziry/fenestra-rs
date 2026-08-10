use super::super::types::{
    NativeFailureCauseV1, NativeObservationV1, NativeOutcomeV1, NativeTraceStageV1,
    NativeTraceStepV1,
};

pub(crate) fn validate_step_v1(step: NativeTraceStepV1, expected_scheduler_turn: u64) -> bool {
    if !step.pending.is_bounded()
        || step.scheduler_state.is_none()
        || step.current_generation.is_none()
        || !scheduler_turn_is_valid(step, expected_scheduler_turn)
        || !stage_tuple_is_valid(step)
        || !identity_shape_is_valid(step)
        || !surface_observation_is_valid(step)
        || !pointer_fields_are_valid(step)
        || !input_source_is_valid(step)
        || !capture_is_valid(step)
        || !publication_is_valid(step)
        || !generation_values_are_valid(step)
        || !post_state_is_valid(step)
        || !staging_digest_is_valid(step)
        || !surface_is_valid(step)
    {
        return false;
    }
    true
}

fn generation_values_are_valid(step: NativeTraceStepV1) -> bool {
    let Some(current) = step.current_generation else {
        return false;
    };
    step.captured_generation
        .is_none_or(|captured| captured == current)
        && step
            .published_generation
            .is_none_or(|published| published == current)
}

fn post_state_is_valid(step: NativeTraceStepV1) -> bool {
    if matches!(
        (step.stage, step.observation, step.outcome),
        (
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Present,
            NativeOutcomeV1::Failed(NativeFailureCauseV1::Presenter),
        )
    ) {
        return step.scheduler_state
            == Some(fenestra_ui_runtime::prototype::SchedulerState::Faulted);
    }
    if !matches!(
        (step.stage, step.observation, step.outcome),
        (
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Shutdown,
            NativeOutcomeV1::Stopped,
        )
    ) {
        return true;
    }
    step.scheduler_state == Some(fenestra_ui_runtime::prototype::SchedulerState::Stopped)
        && !step.redraw_armed
        && step.pending.surface() == 0
        && step.pending.pointer() == 0
        && step.pending.presenter() == 0
        && lane_is_empty(step.deferred)
        && lane_is_empty(step.controls)
        && lane_is_empty(step.visual)
        && lane_is_empty(step.in_flight)
}

const fn lane_is_empty(lane: super::super::types::NativeTraceLaneStatsV1) -> bool {
    lane.items() == 0 && lane.accounted_bytes() == 0
}

fn staging_digest_is_valid(step: NativeTraceStepV1) -> bool {
    let required = matches!(
        (step.stage, step.observation, step.outcome),
        (
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Frame,
            NativeOutcomeV1::Accepted,
        )
    );
    step.staging_digest.is_some() == required
}

fn stage_tuple_is_valid(step: NativeTraceStepV1) -> bool {
    matches!(
        (step.stage, step.observation, step.outcome),
        (
            NativeTraceStageV1::Manifest,
            NativeObservationV1::Build,
            NativeOutcomeV1::Observed
        ) | (
            NativeTraceStageV1::Shell,
            NativeObservationV1::Resumed,
            NativeOutcomeV1::Observed
                | NativeOutcomeV1::Failed(
                    NativeFailureCauseV1::Runtime | NativeFailureCauseV1::Presenter,
                ),
        ) | (
            NativeTraceStageV1::Shell,
            NativeObservationV1::Close,
            NativeOutcomeV1::Completed | NativeOutcomeV1::Failed(NativeFailureCauseV1::Runtime),
        ) | (
            NativeTraceStageV1::Shell,
            NativeObservationV1::Timeout,
            NativeOutcomeV1::Failed(NativeFailureCauseV1::Timeout),
        ) | (
            NativeTraceStageV1::Platform,
            NativeObservationV1::Surface,
            NativeOutcomeV1::Observed
                | NativeOutcomeV1::Coalesced
                | NativeOutcomeV1::Failed(
                    NativeFailureCauseV1::Arithmetic
                        | NativeFailureCauseV1::WidthLimit
                        | NativeFailureCauseV1::HeightLimit
                        | NativeFailureCauseV1::SurfaceRepaintUnavailable,
                ),
        ) | (
            NativeTraceStageV1::Platform,
            NativeObservationV1::Scale,
            NativeOutcomeV1::Failed(
                NativeFailureCauseV1::InvalidScale | NativeFailureCauseV1::EnvironmentScaleChanged,
            ),
        ) | (
            NativeTraceStageV1::Platform,
            NativeObservationV1::Pointer,
            NativeOutcomeV1::Observed | NativeOutcomeV1::Coalesced,
        ) | (
            NativeTraceStageV1::Platform,
            NativeObservationV1::Redraw,
            NativeOutcomeV1::Ignored
        ) | (
            NativeTraceStageV1::Platform,
            NativeObservationV1::Close,
            NativeOutcomeV1::Observed | NativeOutcomeV1::Coalesced,
        ) | (
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Surface,
            NativeOutcomeV1::Deferred
                | NativeOutcomeV1::Published
                | NativeOutcomeV1::Coalesced
                | NativeOutcomeV1::Ignored,
        ) | (
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Frame,
            NativeOutcomeV1::Armed
                | NativeOutcomeV1::Offered
                | NativeOutcomeV1::Accepted
                | NativeOutcomeV1::Rejected,
        ) | (
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Present,
            NativeOutcomeV1::Accepted | NativeOutcomeV1::Failed(NativeFailureCauseV1::Presenter),
        ) | (
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Completion,
            NativeOutcomeV1::Accepted | NativeOutcomeV1::Completed,
        ) | (
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Shutdown,
            NativeOutcomeV1::Accepted | NativeOutcomeV1::Stopped,
        ) | (
            NativeTraceStageV1::Renderer,
            NativeObservationV1::Frame,
            NativeOutcomeV1::Rejected
        ) | (
            NativeTraceStageV1::Renderer,
            NativeObservationV1::Present,
            NativeOutcomeV1::Completed,
        ) | (
            NativeTraceStageV1::Oracle,
            NativeObservationV1::Surface,
            NativeOutcomeV1::Matched | NativeOutcomeV1::Failed(NativeFailureCauseV1::Oracle),
        )
    )
}

fn scheduler_turn_is_valid(step: NativeTraceStepV1, expected: u64) -> bool {
    if step.stage == NativeTraceStageV1::Scheduler {
        step.scheduler_turn == Some(expected)
    } else {
        step.scheduler_turn.is_none()
    }
}

fn surface_observation_is_valid(step: NativeTraceStepV1) -> bool {
    let required = matches!(
        (step.stage, step.observation, step.outcome),
        (
            NativeTraceStageV1::Platform,
            NativeObservationV1::Scale,
            NativeOutcomeV1::Failed(NativeFailureCauseV1::EnvironmentScaleChanged),
        )
    );
    step.surface_observation.is_some() == required
}

fn pointer_fields_are_valid(step: NativeTraceStepV1) -> bool {
    let press = matches!(
        (step.stage, step.observation, step.outcome),
        (
            NativeTraceStageV1::Platform,
            NativeObservationV1::Pointer,
            NativeOutcomeV1::Observed,
        )
    );
    step.target.is_some() == press
}

fn input_source_is_valid(step: NativeTraceStepV1) -> bool {
    let required = step.stage == NativeTraceStageV1::Platform
        && matches!(
            step.observation,
            NativeObservationV1::Pointer | NativeObservationV1::Close
        );
    step.input_source.is_some() == required
}

fn capture_is_valid(step: NativeTraceStepV1) -> bool {
    let required = matches!(
        (step.stage, step.observation, step.outcome),
        (
            NativeTraceStageV1::Platform,
            NativeObservationV1::Pointer,
            NativeOutcomeV1::Observed,
        ) | (
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Surface,
            NativeOutcomeV1::Deferred,
        )
    );
    step.captured_generation.is_some() == required
}

fn publication_is_valid(step: NativeTraceStepV1) -> bool {
    let required = matches!(
        (step.stage, step.observation, step.outcome),
        (
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Surface,
            NativeOutcomeV1::Published,
        )
    );
    let optional = matches!(
        (step.stage, step.observation, step.outcome),
        (
            NativeTraceStageV1::Oracle,
            NativeObservationV1::Surface,
            NativeOutcomeV1::Matched,
        ) | (
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Frame,
            NativeOutcomeV1::Offered,
        )
    );
    (required && step.published_generation.is_some())
        || (optional)
        || (!required && !optional && step.published_generation.is_none())
}

fn identity_shape_is_valid(step: NativeTraceStepV1) -> bool {
    let (frame, submission, control) = match (step.stage, step.observation, step.outcome) {
        (NativeTraceStageV1::Scheduler, NativeObservationV1::Frame, NativeOutcomeV1::Accepted) => {
            (true, true, false)
        }
        (
            NativeTraceStageV1::Renderer | NativeTraceStageV1::Scheduler,
            NativeObservationV1::Frame,
            NativeOutcomeV1::Rejected,
        )
        | (NativeTraceStageV1::Scheduler, NativeObservationV1::Frame, NativeOutcomeV1::Offered) => {
            (true, false, false)
        }
        (
            NativeTraceStageV1::Renderer,
            NativeObservationV1::Present,
            NativeOutcomeV1::Completed,
        ) => (true, true, false),
        (
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Present,
            NativeOutcomeV1::Accepted | NativeOutcomeV1::Failed(NativeFailureCauseV1::Presenter),
        ) => (true, true, true),
        (
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Completion,
            NativeOutcomeV1::Accepted | NativeOutcomeV1::Completed,
        ) => (false, true, true),
        (
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Shutdown,
            NativeOutcomeV1::Accepted | NativeOutcomeV1::Stopped,
        ) => (false, false, true),
        _ => (false, false, false),
    };
    step.frame.is_some() == frame
        && step.submission.is_some() == submission
        && step.control.is_some() == control
}

fn surface_is_valid(step: NativeTraceStepV1) -> bool {
    let required = surface_is_required(step);
    let forbidden = surface_is_forbidden(step);
    (!required || step.surface.is_some()) && (!forbidden || step.surface.is_none())
}

fn surface_is_required(step: NativeTraceStepV1) -> bool {
    matches!(
        (step.stage, step.observation, step.outcome),
        (
            NativeTraceStageV1::Platform,
            NativeObservationV1::Surface,
            NativeOutcomeV1::Observed
                | NativeOutcomeV1::Coalesced
                | NativeOutcomeV1::Failed(NativeFailureCauseV1::SurfaceRepaintUnavailable),
        ) | (
            NativeTraceStageV1::Platform,
            NativeObservationV1::Pointer,
            NativeOutcomeV1::Observed,
        ) | (
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Surface,
            NativeOutcomeV1::Deferred | NativeOutcomeV1::Published | NativeOutcomeV1::Coalesced,
        ) | (
            NativeTraceStageV1::Scheduler | NativeTraceStageV1::Renderer,
            NativeObservationV1::Frame | NativeObservationV1::Present,
            _,
        ) | (
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Completion,
            _,
        ) | (
            NativeTraceStageV1::Oracle,
            NativeObservationV1::Surface,
            NativeOutcomeV1::Matched | NativeOutcomeV1::Failed(NativeFailureCauseV1::Oracle),
        )
    )
}

fn surface_is_forbidden(step: NativeTraceStepV1) -> bool {
    matches!(
        step.observation,
        NativeObservationV1::Build
            | NativeObservationV1::Resumed
            | NativeObservationV1::Close
            | NativeObservationV1::Shutdown
            | NativeObservationV1::Timeout
    ) || matches!(
        (step.stage, step.observation, step.outcome),
        (
            NativeTraceStageV1::Platform,
            NativeObservationV1::Pointer,
            NativeOutcomeV1::Coalesced,
        ) | (
            NativeTraceStageV1::Platform,
            NativeObservationV1::Scale,
            NativeOutcomeV1::Failed(
                NativeFailureCauseV1::InvalidScale | NativeFailureCauseV1::EnvironmentScaleChanged,
            ),
        ) | (
            NativeTraceStageV1::Platform,
            NativeObservationV1::Surface,
            NativeOutcomeV1::Failed(
                NativeFailureCauseV1::Arithmetic
                    | NativeFailureCauseV1::WidthLimit
                    | NativeFailureCauseV1::HeightLimit,
            ),
        ) | (
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Surface,
            NativeOutcomeV1::Ignored,
        )
    )
}
