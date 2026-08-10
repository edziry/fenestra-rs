use std::cell::Cell;

use fenestra_ui_runtime::prototype::SchedulerTick;

use super::super::trace::{
    NativeFailureCauseV1, NativeObservationV1, NativeOutcomeV1, NativeTraceErrorKindV1,
    NativeTraceStageV1, NativeTraceStepV1, NativeTraceSubmissionV1, NativeTraceV1,
};
use super::super::{NativePhysicalExtentV1, NativeSurfaceStateV1, NativeSurfaceTupleV1};
use super::generation_zero;

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

const fn step(
    stage: NativeTraceStageV1,
    observation: NativeObservationV1,
    outcome: NativeOutcomeV1,
) -> NativeTraceStepV1 {
    NativeTraceStepV1::new(stage, observation, outcome)
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
