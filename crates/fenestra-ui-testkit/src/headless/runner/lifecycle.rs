use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{
    CommittedRuntimeSnapshot, CompletionWatermark, ControlAdmission, ControlSequence,
    HeadlessPoint, SchedulerAction, SchedulerInput, SchedulerInputResult, SchedulerState,
    SubmissionId,
};

use crate::case::SemanticOperationV1;
use crate::headless::platform::HeadlessPointerTargetV1;
use crate::headless::trace::{HeadlessInputKindV1, HeadlessOutcomeV1, HeadlessTraceStageV1};
use crate::scheduler::{
    FakeControlDeliveryV1, FakeRendererModeV1, FakeRendererOfferOutcomeV1,
    SchedulerTraceInputOutcomeV1, SchedulerTraceStepV1,
};

use super::record::{ActionAttempt, accepted_submission, admission_sequence, step};
use super::state::{
    COLOR, INSERTED_KEY, RunState, StableNodes, VISIBLE, WIDTH, control_path, ensure, item_path,
    projection_error, renderer_error, rgba, root_path, scheduler_error,
};
use super::types::{HeadlessRunErrorV1, HeadlessRunV1};

pub(super) fn run_second_half(
    mut state: RunState,
    stable: StableNodes,
    control: fenestra_ui_runtime::prototype::NodeId,
    inserted: fenestra_ui_runtime::prototype::NodeId,
    token_zero_snapshot: CommittedRuntimeSnapshot,
    token_zero: SubmissionId,
) -> Result<HeadlessRunV1, HeadlessRunErrorV1> {
    state.commit_operation(
        9,
        &SemanticOperationV1::SetProperty {
            node: control_path(),
            property: VISIBLE,
            value: PropertyValue::Bool(false),
        },
        7,
    )?;
    ensure_stable(&state, &stable, inserted)?;
    request_frame(&mut state, 9)?;
    let current = state.capture_pointer(HeadlessPoint::new(5, 5))?;
    let old_hit = token_zero_snapshot
        .headless_projection()
        .and_then(|projection| projection.hit_test(HeadlessPoint::new(5, 5)));
    ensure(
        current.target() == HeadlessPointerTargetV1::None && old_hit == Some(control),
        projection_error,
    )?;
    drop(token_zero_snapshot);

    state.commit_operation(
        10,
        &SemanticOperationV1::SetProperty {
            node: root_path(),
            property: WIDTH,
            value: PropertyValue::ScalarI32(84),
        },
        8,
    )?;
    ensure_stable(&state, &stable, inserted)?;
    idle(
        &mut state,
        10,
        HeadlessInputKindV1::None,
        HeadlessOutcomeV1::Action,
    )?;

    state.frame_ready(11)?;
    let failed = take_offer(&mut state, 11)?;
    ensure(
        failed.id().get() == 1
            && failed.generation().get() == 8
            && failed.earliest_tick().get() == 9
            && failed.latest_tick().get() == 10,
        scheduler_error,
    )?;
    let failed_id = failed.id();
    let failed_snapshot = failed.snapshot().clone();
    let failed_invalidation = failed.invalidation();
    let rejected = state.offer(failed, FakeRendererModeV1::Fail)?;
    ensure(
        rejected == FakeRendererOfferOutcomeV1::Rejected(failed_id),
        renderer_error,
    )?;
    let retry = take_offer(&mut state, 11)?;
    ensure(
        retry.id().get() == 2
            && retry.generation().get() == 8
            && retry.snapshot().shares_state_with(&failed_snapshot)
            && retry.invalidation() == failed_invalidation
            && retry.earliest_tick().get() == 9
            && retry.latest_tick().get() == 10,
        scheduler_error,
    )?;
    drop(failed_snapshot);
    let token_one = accepted_submission(state.offer(retry, FakeRendererModeV1::Late)?)?;
    ensure(
        token_one.epoch().get() == 0 && token_one.token() == 1,
        renderer_error,
    )?;

    let completion_zero = complete(&mut state, 12, token_zero)?;
    ensure(completion_zero.get() == 0, renderer_error)?;
    idle(
        &mut state,
        13,
        HeadlessInputKindV1::Completion,
        HeadlessOutcomeV1::Completed,
    )?;

    state.commit_operation(
        14,
        &SemanticOperationV1::SetProperty {
            node: root_path(),
            property: COLOR,
            value: rgba([9, 9, 9, 255]),
        },
        9,
    )?;
    ensure_stable(&state, &stable, inserted)?;
    request_frame(&mut state, 14)?;

    state.frame_ready(15)?;
    let lost = take_offer(&mut state, 15)?;
    ensure(
        lost.id().get() == 3 && lost.generation().get() == 9,
        scheduler_error,
    )?;
    let loss = state.offer(lost, FakeRendererModeV1::Loss)?;
    let FakeRendererOfferOutcomeV1::Loss(FakeControlDeliveryV1::Accepted(loss_admission)) = loss
    else {
        return Err(renderer_error());
    };
    ensure(
        admission_sequence(loss_admission).get() == 1,
        renderer_error,
    )?;
    let shutdown = request_shutdown(&mut state, 15, false)?;
    ensure(shutdown.get() == 2, scheduler_error)?;
    let duplicate = request_shutdown(&mut state, 15, true)?;
    ensure(duplicate == shutdown, scheduler_error)?;

    idle(
        &mut state,
        16,
        HeadlessInputKindV1::Loss,
        HeadlessOutcomeV1::Lost,
    )?;
    ensure(
        state.scheduler.state() == SchedulerState::Faulted,
        scheduler_error,
    )?;
    let shutdown_step = step(
        HeadlessTraceStageV1::Scheduler,
        HeadlessInputKindV1::Shutdown,
        HeadlessOutcomeV1::Action,
    );
    let ActionAttempt::Ready(Some(SchedulerAction::StopRenderer(stopped_at))) =
        state.next_action(17, shutdown_step, false)?
    else {
        return Err(scheduler_error());
    };
    ensure(
        stopped_at == shutdown && state.scheduler.state() == SchedulerState::Draining,
        scheduler_error,
    )?;

    let completion_one = complete(&mut state, 18, token_one)?;
    ensure(completion_one.get() == 3, renderer_error)?;
    let stopped = step(
        HeadlessTraceStageV1::Scheduler,
        HeadlessInputKindV1::Completion,
        HeadlessOutcomeV1::Stopped,
    );
    ensure(
        matches!(
            state.next_action(19, stopped, false)?,
            ActionAttempt::Ready(None)
        ) && state.scheduler.state() == SchedulerState::Stopped,
        scheduler_error,
    )?;
    let stats = state.scheduler.stats();
    ensure(
        stats.deferred().items() == 0
            && stats.controls().items() == 0
            && stats.visual().items() == 0
            && stats.in_flight().items() == 0
            && state.renderer.stats().items() == 0
            && !state.renderer.stats().has_pending_control()
            && state.headless_trace.len() == 55
            && state.scheduler_trace.len() == 41,
        scheduler_error,
    )?;
    state.finish()
}

fn ensure_stable(
    state: &RunState,
    stable: &StableNodes,
    inserted: fenestra_ui_runtime::prototype::NodeId,
) -> Result<(), HeadlessRunErrorV1> {
    let snapshot = state.scheduler.committed();
    state.ensure_nodes(&snapshot, stable)?;
    ensure(
        state.node(&snapshot, &item_path(INSERTED_KEY))? == inserted,
        projection_error,
    )
}

fn request_frame(state: &mut RunState, tick: u64) -> Result<(), HeadlessRunErrorV1> {
    let event = step(
        HeadlessTraceStageV1::Scheduler,
        HeadlessInputKindV1::None,
        HeadlessOutcomeV1::Action,
    );
    ensure(
        matches!(
            state.next_action(tick, event, false)?,
            ActionAttempt::Ready(Some(SchedulerAction::RequestFrame))
        ),
        scheduler_error,
    )
}

fn idle(
    state: &mut RunState,
    tick: u64,
    input: HeadlessInputKindV1,
    outcome: HeadlessOutcomeV1,
) -> Result<(), HeadlessRunErrorV1> {
    let event = step(HeadlessTraceStageV1::Scheduler, input, outcome);
    ensure(
        matches!(
            state.next_action(tick, event, false)?,
            ActionAttempt::Ready(None)
        ),
        scheduler_error,
    )
}

fn take_offer(
    state: &mut RunState,
    tick: u64,
) -> Result<fenestra_ui_runtime::prototype::FrameWork, HeadlessRunErrorV1> {
    let event = step(
        HeadlessTraceStageV1::Scheduler,
        HeadlessInputKindV1::None,
        HeadlessOutcomeV1::Action,
    );
    match state.next_action(tick, event, false)? {
        ActionAttempt::Ready(Some(SchedulerAction::OfferFrame(frame))) => Ok(frame),
        _ => Err(scheduler_error()),
    }
}

fn complete(
    state: &mut RunState,
    tick: u64,
    submission: SubmissionId,
) -> Result<ControlSequence, HeadlessRunErrorV1> {
    state.advance_to(tick)?;
    let watermark = CompletionWatermark::from_submission(submission);
    let delivery = state
        .renderer
        .complete(&mut state.scheduler, watermark, state.clock.now())
        .map_err(|_| renderer_error())?;
    let FakeControlDeliveryV1::Accepted(admission) = delivery else {
        return Err(renderer_error());
    };
    let control = admission_sequence(admission);
    let scheduler = SchedulerTraceStepV1::Input {
        input: SchedulerInput::Complete(watermark),
        outcome: SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::Control(admission)),
    };
    let mut headless = step(
        HeadlessTraceStageV1::Renderer,
        HeadlessInputKindV1::Completion,
        HeadlessOutcomeV1::Completed,
    );
    headless.control = Some(control);
    state.record_both(scheduler, headless)?;
    Ok(control)
}

fn request_shutdown(
    state: &mut RunState,
    tick: u64,
    duplicate: bool,
) -> Result<ControlSequence, HeadlessRunErrorV1> {
    state.advance_to(tick)?;
    let result = state
        .scheduler
        .process_input(SchedulerInput::RequestShutdown, state.clock.now())
        .map_err(|_| scheduler_error())?;
    let SchedulerInputResult::Control(admission) = result else {
        return Err(scheduler_error());
    };
    ensure(
        matches!(
            (admission, duplicate),
            (ControlAdmission::Accepted(_), false) | (ControlAdmission::AlreadyAccepted(_), true)
        ),
        scheduler_error,
    )?;
    let control = admission_sequence(admission);
    let scheduler = SchedulerTraceStepV1::Input {
        input: SchedulerInput::RequestShutdown,
        outcome: SchedulerTraceInputOutcomeV1::Accepted(result),
    };
    let mut headless = step(
        HeadlessTraceStageV1::Input,
        HeadlessInputKindV1::Shutdown,
        if duplicate {
            HeadlessOutcomeV1::NoChange
        } else {
            HeadlessOutcomeV1::Accepted
        },
    );
    headless.control = Some(control);
    state.record_both(scheduler, headless)?;
    Ok(control)
}
