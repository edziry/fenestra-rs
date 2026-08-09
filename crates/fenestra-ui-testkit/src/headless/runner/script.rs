use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{
    CallbackFinish, HeadlessPoint, HeadlessSurface, SchedulerAction, SchedulerErrorKind,
    TransactionErrorKind,
};

use crate::case::SemanticOperationV1;
use crate::headless::platform::{
    HeadlessPointerMutationV1, HeadlessPointerScriptV1, HeadlessPointerTargetV1,
};
use crate::headless::trace::{
    HeadlessFailureCauseV1, HeadlessInputKindV1, HeadlessOutcomeV1, HeadlessTraceStageV1,
};
use crate::scheduler::{FakeCallbackDepthV1, FakeRendererModeV1};

use super::lifecycle;
use super::record::{ActionAttempt, accepted_submission, step};
use super::state::{
    COLOR, HEIGHT, INSERTED_KEY, RunState, SECOND_KEY, WIDTH, container_path, control_path, ensure,
    item_path, items_path, oracle_error, projection_error, rgba, root_path, runtime_error,
    scheduler_error,
};
use super::types::{HeadlessRunErrorV1, HeadlessRunV1};

pub(super) fn run() -> Result<HeadlessRunV1, HeadlessRunErrorV1> {
    let mut state = RunState::new()?;
    state.record_build()?;
    let initial = state.scheduler.committed();
    let root = state.node(&initial, &root_path())?;
    let container = state.node(&initial, &container_path())?;
    let control = state.node(&initial, &control_path())?;
    let first = state.node(&initial, &item_path(10))?;
    let second = state.node(&initial, &item_path(SECOND_KEY))?;
    let stable = [
        (root_path(), root),
        (container_path(), container),
        (control_path(), control),
        (item_path(10), first),
    ];

    let control_capture = state.capture_pointer(HeadlessPoint::new(5, 5))?;
    let retired_capture = state.capture_pointer(HeadlessPoint::new(5, 22))?;
    ensure(
        control_capture.target() == HeadlessPointerTargetV1::StaticControl
            && retired_capture.target() == HeadlessPointerTargetV1::Key(SECOND_KEY),
        projection_error,
    )?;

    let color = SemanticOperationV1::SetProperty {
        node: control_path(),
        property: COLOR,
        value: rgba([20, 30, 40, 255]),
    };
    let callback = state.pointer_callback(
        1,
        HeadlessPointerScriptV1::new(
            HeadlessPoint::new(5, 5),
            FakeCallbackDepthV1::Nested,
            Some(HeadlessPointerMutationV1::new(
                COLOR,
                rgba([20, 30, 40, 255]),
            )),
        ),
    )?;
    ensure(
        callback.target() == HeadlessPointerTargetV1::StaticControl
            && callback.deepest_depth() == 2
            && callback.shares_entry_snapshot()
            && callback.finish()
                == CallbackFinish::Deferred {
                    operation_count: 1,
                    accounted_bytes: 80,
                },
        scheduler_error,
    )?;
    ensure(
        state.scheduler.committed().shares_state_with(&initial)
            && state.scheduler.committed().generation().get() == 0,
        runtime_error,
    )?;
    state
        .oracle
        .apply_operation(&color)
        .map_err(|_| super::state::oracle_error())?;
    let mut publish_pointer = step(
        HeadlessTraceStageV1::Scheduler,
        HeadlessInputKindV1::Pointer,
        HeadlessOutcomeV1::Action,
    );
    publish_pointer.captured_generation = Some(control_capture.generation());
    publish_pointer.target = control_capture.target();
    let action = state.next_action(2, publish_pointer, true)?;
    ensure(
        matches!(
            action,
            ActionAttempt::Ready(Some(SchedulerAction::RequestFrame))
        ) && state.scheduler.committed().generation().get() == 1,
        scheduler_error,
    )?;
    state.record_projection(state.scheduler.committed().generation())?;
    state.ensure_nodes(&state.scheduler.committed(), &stable)?;
    drop(initial);

    state.commit_operation(
        3,
        &SemanticOperationV1::InsertKeyed {
            fragment: items_path(),
            key: INSERTED_KEY,
            final_index: 1,
        },
        2,
    )?;
    let inserted = state.node(&state.scheduler.committed(), &item_path(INSERTED_KEY))?;
    state.ensure_nodes(&state.scheduler.committed(), &stable)?;
    idle(&mut state, 3, HeadlessInputKindV1::None)?;
    state.commit_operation(
        4,
        &SemanticOperationV1::MoveKeyed {
            fragment: items_path(),
            key: INSERTED_KEY,
            final_index: 2,
        },
        3,
    )?;
    state.ensure_nodes(&state.scheduler.committed(), &stable)?;
    ensure(
        state.node(&state.scheduler.committed(), &item_path(INSERTED_KEY))? == inserted,
        projection_error,
    )?;
    idle(&mut state, 4, HeadlessInputKindV1::None)?;
    state.commit_operation(
        5,
        &SemanticOperationV1::UpdateKeyed {
            fragment: items_path(),
            key: INSERTED_KEY,
            property: HEIGHT,
            value: PropertyValue::ScalarI32(14),
        },
        4,
    )?;
    state.ensure_nodes(&state.scheduler.committed(), &stable)?;
    ensure(
        state.node(&state.scheduler.committed(), &item_path(INSERTED_KEY))? == inserted,
        projection_error,
    )?;
    idle(&mut state, 5, HeadlessInputKindV1::None)?;
    state.commit_operation(
        6,
        &SemanticOperationV1::RemoveKeyed {
            fragment: items_path(),
            key: SECOND_KEY,
        },
        5,
    )?;
    state.ensure_nodes(&state.scheduler.committed(), &stable)?;
    ensure(
        state.node(&state.scheduler.committed(), &item_path(INSERTED_KEY))? == inserted
            && state.scheduler.committed().template(second).is_none(),
        projection_error,
    )?;

    let before_stale = state.scheduler.committed();
    let stats_before_stale = state.scheduler.stats();
    let stale = state.captured_callback(
        6,
        &retired_capture,
        HeadlessPointerMutationV1::new(WIDTH, PropertyValue::ScalarI32(41)),
    )?;
    ensure(
        stale.finish()
            == CallbackFinish::Deferred {
                operation_count: 1,
                accounted_bytes: 80,
            },
        scheduler_error,
    )?;
    let mut failed = step(
        HeadlessTraceStageV1::Scheduler,
        HeadlessInputKindV1::Pointer,
        HeadlessOutcomeV1::Failed(HeadlessFailureCauseV1::Runtime),
    );
    failed.captured_generation = Some(retired_capture.generation());
    failed.target = retired_capture.target();
    let ActionAttempt::Failed(error) = state.next_action(6, failed, false)? else {
        return Err(runtime_error());
    };
    ensure(
        error.kind() == SchedulerErrorKind::Transaction(TransactionErrorKind::MissingNode)
            && error.operation_index() == Some(0)
            && state.scheduler.committed().shares_state_with(&before_stale)
            && state.scheduler.stats() == stats_before_stale,
        runtime_error,
    )?;
    drop(before_stale);
    idle(&mut state, 6, HeadlessInputKindV1::None)?;

    let resized = HeadlessSurface::new(90, 70);
    state.oracle.resize(resized).map_err(|_| oracle_error())?;
    let before_resize = state.scheduler.committed();
    let visual_before_resize = state.scheduler.stats().visual();
    let resize = state.resize_callback(7, resized)?;
    ensure(
        state
            .scheduler
            .committed()
            .shares_state_with(&before_resize)
            && state.scheduler.stats().visual() == visual_before_resize,
        runtime_error,
    )?;
    let mut publish_resize = step(
        HeadlessTraceStageV1::Scheduler,
        HeadlessInputKindV1::Resize,
        HeadlessOutcomeV1::Published,
    );
    publish_resize.captured_generation = Some(resize.captured_generation());
    ensure(
        matches!(
            state.next_action(7, publish_resize, true)?,
            ActionAttempt::Ready(None)
        ) && state.scheduler.committed().generation().get() == 6,
        runtime_error,
    )?;
    state.record_projection(state.scheduler.committed().generation())?;
    drop(before_resize);
    let generation_six = state.scheduler.committed();
    state.ensure_nodes(&generation_six, &stable)?;
    ensure(
        state.node(&generation_six, &item_path(INSERTED_KEY))? == inserted,
        projection_error,
    )?;
    let visual_at_seven = state.scheduler.stats().visual();
    ensure(
        visual_at_seven
            .latest_tick()
            .is_some_and(|tick| tick.get() == 7),
        scheduler_error,
    )?;

    state.oracle.resize(resized).map_err(|_| oracle_error())?;
    let repeated = state.resize_callback(8, resized)?;
    let mut no_change = step(
        HeadlessTraceStageV1::Scheduler,
        HeadlessInputKindV1::Resize,
        HeadlessOutcomeV1::NoChange,
    );
    no_change.captured_generation = Some(repeated.captured_generation());
    ensure(
        matches!(
            state.next_action(8, no_change, false)?,
            ActionAttempt::Ready(None)
        ) && state
            .scheduler
            .committed()
            .shares_state_with(&generation_six)
            && state.scheduler.stats().visual() == visual_at_seven,
        runtime_error,
    )?;

    state.frame_ready(8)?;
    let offer = step(
        HeadlessTraceStageV1::Scheduler,
        HeadlessInputKindV1::None,
        HeadlessOutcomeV1::Action,
    );
    let ActionAttempt::Ready(Some(SchedulerAction::OfferFrame(frame))) =
        state.next_action(8, offer, false)?
    else {
        return Err(scheduler_error());
    };
    ensure(
        frame.id().get() == 0
            && frame.generation().get() == 6
            && frame.earliest_tick().get() == 2
            && frame.latest_tick().get() == 7,
        scheduler_error,
    )?;
    let token_zero_snapshot = frame.snapshot().clone();
    let token_zero = accepted_submission(state.offer(frame, FakeRendererModeV1::Late)?)?;
    ensure(
        token_zero.epoch().get() == 0 && token_zero.token() == 0,
        scheduler_error,
    )?;
    drop(generation_six);

    lifecycle::run_second_half(
        state,
        stable,
        control,
        inserted,
        token_zero_snapshot,
        token_zero,
    )
}

fn idle(
    state: &mut RunState,
    tick: u64,
    input: HeadlessInputKindV1,
) -> Result<(), HeadlessRunErrorV1> {
    let headless = step(
        HeadlessTraceStageV1::Scheduler,
        input,
        HeadlessOutcomeV1::Action,
    );
    ensure(
        matches!(
            state.next_action(tick, headless, false)?,
            ActionAttempt::Ready(None)
        ),
        scheduler_error,
    )
}
