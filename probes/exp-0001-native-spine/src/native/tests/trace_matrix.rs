use std::cell::Cell;

use fenestra_ui_runtime::prototype::SchedulerTick;
use fenestra_ui_testkit::prototype::HeadlessPointerTargetV1;

use super::super::surface::NativeSurfaceObservationV1;
use super::super::trace::{
    NativeFailureCauseV1, NativeInputSourceV1, NativeObservationV1, NativeOutcomeV1,
    NativeTraceErrorKindV1, NativeTraceStageV1, NativeTraceStepV1, NativeTraceSubmissionV1,
    NativeTraceV1,
};
use super::super::{NativePhysicalExtentV1, NativeSurfaceStateV1, NativeSurfaceTupleV1};
use super::{generation_zero, trace_step};

#[test]
fn stage_observation_and_outcome_matrix_rejects_near_misses_before_storage() {
    let shell_stopped = step(
        NativeTraceStageV1::Shell,
        NativeObservationV1::Resumed,
        NativeOutcomeV1::Stopped,
    );
    let mut scheduler_completed = step(
        NativeTraceStageV1::Scheduler,
        NativeObservationV1::Frame,
        NativeOutcomeV1::Completed,
    );
    scheduler_completed.scheduler_turn = Some(0);
    scheduler_completed.surface = Some(surface());
    let mut platform_published = step(
        NativeTraceStageV1::Platform,
        NativeObservationV1::Surface,
        NativeOutcomeV1::Published,
    );
    platform_published.published_generation = Some(generation_zero());
    platform_published.surface = Some(surface());
    let mut scheduler_observed = step(
        NativeTraceStageV1::Scheduler,
        NativeObservationV1::Surface,
        NativeOutcomeV1::Observed,
    );
    scheduler_observed.scheduler_turn = Some(0);
    scheduler_observed.surface = Some(surface());
    let mut renderer_accepted = step(
        NativeTraceStageV1::Renderer,
        NativeObservationV1::Frame,
        NativeOutcomeV1::Accepted,
    );
    renderer_accepted.surface = Some(surface());
    renderer_accepted.frame = Some(0);
    renderer_accepted.submission = Some(NativeTraceSubmissionV1::new(0, 0));

    for invalid in [
        shell_stopped,
        scheduler_completed,
        platform_published,
        scheduler_observed,
        renderer_accepted,
    ] {
        assert_invalid_before_storage(invalid);
    }
}

#[test]
fn environment_failures_have_one_exact_stage_and_observation() {
    let mut scale_on_surface = step(
        NativeTraceStageV1::Platform,
        NativeObservationV1::Surface,
        NativeOutcomeV1::Failed(NativeFailureCauseV1::EnvironmentScaleChanged),
    );
    scale_on_surface.surface = Some(surface());
    let mut repaint_on_scale = step(
        NativeTraceStageV1::Platform,
        NativeObservationV1::Scale,
        NativeOutcomeV1::Failed(NativeFailureCauseV1::SurfaceRepaintUnavailable),
    );
    repaint_on_scale.surface = Some(surface());
    let mut scale_on_scheduler = step(
        NativeTraceStageV1::Scheduler,
        NativeObservationV1::Surface,
        NativeOutcomeV1::Failed(NativeFailureCauseV1::EnvironmentScaleChanged),
    );
    scale_on_scheduler.scheduler_turn = Some(0);
    scale_on_scheduler.surface = Some(surface());

    for invalid in [scale_on_surface, repaint_on_scale, scale_on_scheduler] {
        assert_invalid_before_storage(invalid);
    }
}

#[test]
fn shell_outcomes_and_failure_causes_are_exact() {
    assert_valid(step(
        NativeTraceStageV1::Shell,
        NativeObservationV1::Resumed,
        NativeOutcomeV1::Observed,
    ));
    assert_valid(step(
        NativeTraceStageV1::Shell,
        NativeObservationV1::Close,
        NativeOutcomeV1::Completed,
    ));

    for cause in NativeFailureCauseV1::ALL {
        assert_applicability(
            step(
                NativeTraceStageV1::Shell,
                NativeObservationV1::Resumed,
                NativeOutcomeV1::Failed(cause),
            ),
            matches!(
                cause,
                NativeFailureCauseV1::Runtime | NativeFailureCauseV1::Presenter
            ),
        );
        assert_applicability(
            step(
                NativeTraceStageV1::Shell,
                NativeObservationV1::Close,
                NativeOutcomeV1::Failed(cause),
            ),
            cause == NativeFailureCauseV1::Runtime,
        );
        assert_applicability(
            step(
                NativeTraceStageV1::Shell,
                NativeObservationV1::Timeout,
                NativeOutcomeV1::Failed(cause),
            ),
            cause == NativeFailureCauseV1::Timeout,
        );
    }
}

#[test]
fn platform_surface_failure_causes_and_tuple_shapes_are_exact() {
    for outcome in [NativeOutcomeV1::Observed, NativeOutcomeV1::Coalesced] {
        let mut valid = step(
            NativeTraceStageV1::Platform,
            NativeObservationV1::Surface,
            outcome,
        );
        valid.surface = Some(surface());
        assert_valid(valid);
        valid.surface = None;
        assert_invalid_before_storage(valid);
    }

    for cause in NativeFailureCauseV1::ALL {
        let allowed_without_tuple = matches!(
            cause,
            NativeFailureCauseV1::Arithmetic
                | NativeFailureCauseV1::WidthLimit
                | NativeFailureCauseV1::HeightLimit
        );
        let allowed_with_tuple = cause == NativeFailureCauseV1::SurfaceRepaintUnavailable;
        let mut candidate = step(
            NativeTraceStageV1::Platform,
            NativeObservationV1::Surface,
            NativeOutcomeV1::Failed(cause),
        );
        if allowed_with_tuple {
            candidate.surface = Some(surface());
        }
        assert_applicability(candidate, allowed_without_tuple || allowed_with_tuple);

        if allowed_without_tuple || allowed_with_tuple {
            candidate.surface = if allowed_with_tuple {
                None
            } else {
                Some(surface())
            };
            assert_invalid_before_storage(candidate);
        }
    }
}

#[test]
fn platform_scale_failure_causes_and_observation_shapes_are_exact() {
    for cause in NativeFailureCauseV1::ALL {
        let allowed = matches!(
            cause,
            NativeFailureCauseV1::InvalidScale | NativeFailureCauseV1::EnvironmentScaleChanged
        );
        let mut candidate = step(
            NativeTraceStageV1::Platform,
            NativeObservationV1::Scale,
            NativeOutcomeV1::Failed(cause),
        );
        if cause == NativeFailureCauseV1::EnvironmentScaleChanged {
            candidate.surface_observation = Some(surface_observation());
        }
        assert_applicability(candidate, allowed);

        if allowed {
            let mut with_surface = candidate;
            with_surface.surface = Some(surface());
            assert_invalid_before_storage(with_surface);
            candidate.surface_observation = if candidate.surface_observation.is_some() {
                None
            } else {
                Some(surface_observation())
            };
            assert_invalid_before_storage(candidate);
        }
    }
}

#[test]
fn cursor_move_and_press_fields_are_exact() {
    for source in NativeInputSourceV1::ALL {
        let mut moved = step(
            NativeTraceStageV1::Platform,
            NativeObservationV1::Pointer,
            NativeOutcomeV1::Coalesced,
        );
        moved.input_source = Some(source);
        assert_valid(moved);

        let mut pressed = step(
            NativeTraceStageV1::Platform,
            NativeObservationV1::Pointer,
            NativeOutcomeV1::Observed,
        );
        pressed.input_source = Some(source);
        pressed.captured_generation = Some(generation_zero());
        pressed.surface = Some(surface());
        pressed.target = Some(HeadlessPointerTargetV1::StaticControl);
        assert_valid(pressed);
    }

    let mut moved = pointer(NativeOutcomeV1::Coalesced);
    for field in 0..4 {
        let mut invalid = moved;
        match field {
            0 => invalid.input_source = None,
            1 => invalid.captured_generation = Some(generation_zero()),
            2 => invalid.surface = Some(surface()),
            _ => invalid.target = Some(HeadlessPointerTargetV1::None),
        }
        assert_invalid_before_storage(invalid);
    }
    moved.outcome = NativeOutcomeV1::Observed;
    for field in 0..4 {
        let mut invalid = moved;
        invalid.captured_generation = Some(generation_zero());
        invalid.surface = Some(surface());
        invalid.target = Some(HeadlessPointerTargetV1::None);
        match field {
            0 => invalid.input_source = None,
            1 => invalid.captured_generation = None,
            2 => invalid.surface = None,
            _ => invalid.target = None,
        }
        assert_invalid_before_storage(invalid);
    }
}

#[test]
fn scheduler_and_oracle_surface_shapes_are_exact() {
    for outcome in [
        NativeOutcomeV1::Deferred,
        NativeOutcomeV1::Published,
        NativeOutcomeV1::Coalesced,
        NativeOutcomeV1::Ignored,
    ] {
        let mut valid = scheduler_surface(outcome);
        assert_valid(valid);
        valid.surface = if valid.surface.is_some() {
            None
        } else {
            Some(surface())
        };
        assert_invalid_before_storage(valid);
    }

    let mut matched = oracle(NativeOutcomeV1::Matched);
    assert_valid(matched);
    matched.published_generation = Some(generation_zero());
    assert_valid(matched);
    matched.surface = None;
    assert_invalid_before_storage(matched);

    for cause in NativeFailureCauseV1::ALL {
        assert_applicability(
            oracle(NativeOutcomeV1::Failed(cause)),
            cause == NativeFailureCauseV1::Oracle,
        );
    }
    let mut failed = oracle(NativeOutcomeV1::Failed(NativeFailureCauseV1::Oracle));
    failed.surface = None;
    assert_invalid_before_storage(failed);
    failed.surface = Some(surface());
    failed.published_generation = Some(generation_zero());
    assert_invalid_before_storage(failed);
}

fn assert_invalid_before_storage(invalid: NativeTraceStepV1) {
    let storage_called = Cell::new(false);
    let mut trace = NativeTraceV1::new();
    assert_eq!(
        trace
            .record_with_reserver_for_test(SchedulerTick::new(0), invalid, |_| {
                storage_called.set(true);
                Ok(())
            })
            .expect_err("near-miss trace shape must fail"),
        NativeTraceErrorKindV1::InvalidApplicability
    );
    assert!(!storage_called.get());
    assert!(trace.is_empty());
}

fn assert_valid(valid: NativeTraceStepV1) {
    let mut trace = NativeTraceV1::new();
    trace
        .record(SchedulerTick::new(0), valid)
        .expect("exact trace shape should record");
}

fn assert_applicability(candidate: NativeTraceStepV1, expected: bool) {
    if expected {
        assert_valid(candidate);
    } else {
        assert_invalid_before_storage(candidate);
    }
}

fn step(
    stage: NativeTraceStageV1,
    observation: NativeObservationV1,
    outcome: NativeOutcomeV1,
) -> NativeTraceStepV1 {
    trace_step(stage, observation, outcome)
}

fn surface() -> NativeSurfaceTupleV1 {
    let mut state = NativeSurfaceStateV1::new();
    state
        .observe(NativePhysicalExtentV1::new(640, 480), 2.0)
        .expect("surface should normalize");
    let pending = state.pending_tuple().expect("surface should be pending");
    state
        .promote_pending(pending)
        .expect("surface should promote")
}

fn surface_observation() -> NativeSurfaceObservationV1 {
    NativeSurfaceObservationV1::try_new(NativePhysicalExtentV1::new(641, 481), 2.01)
        .expect("surface observation should normalize")
}

fn pointer(outcome: NativeOutcomeV1) -> NativeTraceStepV1 {
    let mut step = step(
        NativeTraceStageV1::Platform,
        NativeObservationV1::Pointer,
        outcome,
    );
    step.input_source = Some(NativeInputSourceV1::Native);
    step
}

fn scheduler_surface(outcome: NativeOutcomeV1) -> NativeTraceStepV1 {
    let mut step = step(
        NativeTraceStageV1::Scheduler,
        NativeObservationV1::Surface,
        outcome,
    );
    step.scheduler_turn = Some(0);
    if outcome != NativeOutcomeV1::Ignored {
        step.surface = Some(surface());
    }
    if outcome == NativeOutcomeV1::Deferred {
        step.captured_generation = Some(generation_zero());
    }
    if outcome == NativeOutcomeV1::Published {
        step.published_generation = Some(generation_zero());
    }
    step
}

fn oracle(outcome: NativeOutcomeV1) -> NativeTraceStepV1 {
    let mut step = step(
        NativeTraceStageV1::Oracle,
        NativeObservationV1::Surface,
        outcome,
    );
    step.surface = Some(surface());
    step
}
