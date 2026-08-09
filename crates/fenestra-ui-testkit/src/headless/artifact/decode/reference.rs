mod observation;
mod projection;
mod scheduler;

use super::super::error::{HeadlessArtifactDecodeErrorKindV1, HeadlessArtifactDecodeErrorV1};
use super::scan::ScannedArtifactV1;
use super::state::LayoutV1;
use crate::headless::artifact::model::HeadlessArtifactV1;
use crate::headless::artifact::record::scheduler::{
    CallbackRecordV1, CommitRecordV1, InputRecordV1, LaneRecordV1, SchedulerEventRecordV1,
    StepRecordV1,
};
use crate::headless::artifact::record::{QueueRecordV1, TraceEventRecordV1};
use crate::headless::platform::HeadlessPointerTargetV1;
use crate::headless::trace::{
    HeadlessFailureCauseV1, HeadlessInputKindV1, HeadlessOutcomeV1, HeadlessTraceStageV1,
};

pub(super) fn validate_references_v1(
    artifact: &HeadlessArtifactV1,
    scanned: &ScannedArtifactV1<'_>,
    layout: LayoutV1,
) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    validate_headless_identity(artifact, scanned, layout)?;
    observation::validate_observations_v1(artifact, scanned, layout)?;
    scheduler::validate_scheduler_references_v1(artifact, scanned, layout)?;
    validate_trace_correlation(artifact, scanned, layout)?;
    projection::validate_projection_v1(artifact, scanned, layout)
}

fn validate_headless_identity(
    artifact: &HeadlessArtifactV1,
    scanned: &ScannedArtifactV1<'_>,
    layout: LayoutV1,
) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    let lines = scanned.lines();
    for (index, event) in artifact.headless_events.iter().enumerate() {
        let line = &lines[layout.headless.records_start + index];
        if event.sequence != index as u64 || event.domain != 8_001 || !valid_target(event.target) {
            return Err(invalid(line.number));
        }
    }
    for (index, event) in artifact.scheduler_events.iter().enumerate() {
        let line = &lines[layout.scheduler.records_start + index];
        if event.sequence != index as u64 || event.domain != 8_001 {
            return Err(invalid(line.number));
        }
    }
    Ok(())
}

fn validate_trace_correlation(
    artifact: &HeadlessArtifactV1,
    scanned: &ScannedArtifactV1<'_>,
    layout: LayoutV1,
) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    let lines = scanned.lines();
    let mut scheduler_index = 0_usize;
    let mut generation = artifact
        .headless_events
        .first()
        .and_then(|event| event.published)
        .unwrap_or_default();
    let mut last_offered_frame = None;
    for (headless_index, headless) in artifact.headless_events.iter().enumerate() {
        if let Some(published) = headless.published {
            generation = published;
        }
        if !maps_to_scheduler(headless) {
            continue;
        }
        let Some(scheduler) = artifact.scheduler_events.get(scheduler_index) else {
            return Err(invalid(lines[layout.headless.terminal].number));
        };
        let scheduler_line = &lines[layout.scheduler.records_start + scheduler_index];
        if scheduler.tick != headless.tick
            || scheduler.generation != generation
            || !step_matches_headless(scheduler, headless)
            || !lanes_match(scheduler, headless)
        {
            return Err(invalid(scheduler_line.number));
        }
        if let Some(scheduler_owned) =
            frame_or_control_error(scheduler, headless, last_offered_frame)
        {
            let line = if scheduler_owned {
                scheduler_line.number
            } else {
                lines[layout.headless.records_start + headless_index].number
            };
            return Err(invalid(line));
        }
        if let StepRecordV1::Action(
            crate::headless::artifact::record::scheduler::ActionRecordV1::OfferFrame(frame),
        ) = scheduler.step
        {
            last_offered_frame = Some(frame);
        }
        scheduler_index += 1;
    }
    if scheduler_index != artifact.scheduler_events.len() {
        let line = lines
            .get(layout.scheduler.records_start + scheduler_index)
            .unwrap_or(&lines[layout.scheduler.terminal]);
        return Err(invalid(line.number));
    }
    Ok(())
}

fn maps_to_scheduler(event: &TraceEventRecordV1) -> bool {
    !matches!(
        (event.stage, event.outcome),
        (
            HeadlessTraceStageV1::Build | HeadlessTraceStageV1::Projection,
            _
        ) | (HeadlessTraceStageV1::Input, HeadlessOutcomeV1::Observed)
    )
}

fn step_matches_headless(
    scheduler: &SchedulerEventRecordV1,
    headless: &TraceEventRecordV1,
) -> bool {
    match scheduler.step {
        StepRecordV1::Callback { outcome, .. } => {
            headless.stage == HeadlessTraceStageV1::Callback
                && match outcome {
                    CallbackRecordV1::NoChanges => headless.outcome == HeadlessOutcomeV1::NoChange,
                    CallbackRecordV1::Deferred { .. } => {
                        headless.outcome == HeadlessOutcomeV1::Deferred
                    }
                    CallbackRecordV1::ShutdownRequested => {
                        headless.outcome == HeadlessOutcomeV1::Accepted
                    }
                    CallbackRecordV1::Rejected(error) => failure_matches(headless.outcome, error),
                }
        }
        StepRecordV1::Commit(outcome) => {
            headless.stage == HeadlessTraceStageV1::Transaction
                && match outcome {
                    CommitRecordV1::Published => headless.outcome == HeadlessOutcomeV1::Published,
                    CommitRecordV1::Noop => headless.outcome == HeadlessOutcomeV1::NoChange,
                    CommitRecordV1::Rejected(error) => failure_matches(headless.outcome, error),
                }
        }
        StepRecordV1::Action(action) => action_matches_headless(action, headless),
        StepRecordV1::Input { input, .. } => input_matches_headless(input, headless),
    }
}

fn action_matches_headless(
    action: crate::headless::artifact::record::scheduler::ActionRecordV1,
    headless: &TraceEventRecordV1,
) -> bool {
    use crate::headless::artifact::record::scheduler::ActionRecordV1 as Action;
    headless.stage == HeadlessTraceStageV1::Scheduler
        && match action {
            Action::Idle => matches!(
                headless.outcome,
                HeadlessOutcomeV1::Action
                    | HeadlessOutcomeV1::Published
                    | HeadlessOutcomeV1::NoChange
                    | HeadlessOutcomeV1::Completed
                    | HeadlessOutcomeV1::Lost
                    | HeadlessOutcomeV1::Stopped
            ),
            Action::RequestFrame | Action::OfferFrame(_) | Action::StopRenderer(_) => {
                headless.outcome == HeadlessOutcomeV1::Action
            }
            Action::Rejected(error) => failure_matches(headless.outcome, error),
        }
}

fn input_matches_headless(input: InputRecordV1, headless: &TraceEventRecordV1) -> bool {
    match input {
        InputRecordV1::FrameReady => {
            headless.stage == HeadlessTraceStageV1::Input
                && headless.outcome == HeadlessOutcomeV1::Accepted
        }
        InputRecordV1::AcceptFrame(_) => {
            headless.stage == HeadlessTraceStageV1::Renderer
                && headless.outcome == HeadlessOutcomeV1::Accepted
        }
        InputRecordV1::RejectFrame(_) => {
            headless.stage == HeadlessTraceStageV1::Renderer
                && headless.outcome == HeadlessOutcomeV1::Rejected
        }
        InputRecordV1::Complete { .. } => {
            headless.stage == HeadlessTraceStageV1::Renderer
                && headless.outcome == HeadlessOutcomeV1::Completed
        }
        InputRecordV1::RendererLost(_) => {
            headless.stage == HeadlessTraceStageV1::Renderer
                && headless.outcome == HeadlessOutcomeV1::Lost
        }
        InputRecordV1::RequestShutdown => {
            headless.stage == HeadlessTraceStageV1::Input
                && matches!(
                    headless.outcome,
                    HeadlessOutcomeV1::Accepted | HeadlessOutcomeV1::NoChange
                )
        }
    }
}

fn failure_matches(
    outcome: HeadlessOutcomeV1,
    error: fenestra_ui_runtime::prototype::SchedulerErrorKind,
) -> bool {
    let expected = if matches!(
        error,
        fenestra_ui_runtime::prototype::SchedulerErrorKind::Transaction(_)
    ) {
        HeadlessFailureCauseV1::Runtime
    } else {
        HeadlessFailureCauseV1::Scheduler
    };
    outcome == HeadlessOutcomeV1::Failed(expected)
}

fn lanes_match(scheduler: &SchedulerEventRecordV1, headless: &TraceEventRecordV1) -> bool {
    lane_matches(scheduler.deferred, headless.deferred)
        && lane_matches(scheduler.controls, headless.controls)
        && lane_matches(scheduler.visual, headless.visual)
        && lane_matches(scheduler.in_flight, headless.in_flight)
        && scheduler.renderer.items == headless.renderer.items
        && scheduler.renderer.bytes == headless.renderer.bytes
}

const fn lane_matches(scheduler: LaneRecordV1, headless: QueueRecordV1) -> bool {
    scheduler.items == headless.items && scheduler.bytes == headless.bytes
}

fn frame_or_control_error(
    scheduler: &SchedulerEventRecordV1,
    headless: &TraceEventRecordV1,
    last_offered_frame: Option<u64>,
) -> Option<bool> {
    let loss = matches!(
        scheduler.step,
        StepRecordV1::Input {
            input: InputRecordV1::RendererLost(_),
            ..
        }
    ) && headless.input == HeadlessInputKindV1::Loss
        && headless.outcome == HeadlessOutcomeV1::Lost;
    if loss {
        if scheduler.frame.is_some() {
            return Some(true);
        }
        if headless.frame != last_offered_frame {
            return Some(false);
        }
    } else if scheduler.frame != headless.frame {
        return Some(scheduler.frame.is_some());
    }
    (scheduler.control != headless.control).then_some(scheduler.control.is_some())
}

fn valid_target(target: HeadlessPointerTargetV1) -> bool {
    matches!(
        target,
        HeadlessPointerTargetV1::None
            | HeadlessPointerTargetV1::StaticControl
            | HeadlessPointerTargetV1::Key(10 | 20 | 30)
    )
}

fn invalid(line: u32) -> HeadlessArtifactDecodeErrorV1 {
    HeadlessArtifactDecodeErrorV1::at(HeadlessArtifactDecodeErrorKindV1::InvalidReference, line)
}
