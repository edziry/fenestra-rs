use std::cell::Cell;

use fenestra_ui_runtime::prototype::SchedulerTick;
use fenestra_ui_testkit::prototype::HeadlessPointerTargetV1;

use super::super::trace::{
    NativeObservationV1, NativeOutcomeV1, NativeTraceErrorKindV1, NativeTraceStageV1,
    NativeTraceStepV1, NativeTraceSubmissionV1, NativeTraceV1,
};
use super::super::{NativePhysicalExtentV1, NativeSurfaceStateV1, NativeSurfaceTupleV1};
use super::generation_zero;

#[test]
fn required_generation_frame_and_control_paths_are_accepted() {
    for valid in [
        deferred_surface(),
        published_surface(),
        oracle_match(),
        rejected_frame(),
        presented_frame(),
        renderer_loss_control(),
        shutdown_control(NativeOutcomeV1::Accepted),
        shutdown_control(NativeOutcomeV1::Stopped),
    ] {
        let mut trace = NativeTraceV1::new();
        trace
            .record(SchedulerTick::new(0), valid)
            .expect("required applicability path should record");
        assert_eq!(trace.len(), 1);
    }
}

#[test]
fn control_near_miss_and_invalid_stage_tuple_fail_before_storage() {
    let mut control_on_pointer = NativeTraceStepV1::new(
        NativeTraceStageV1::Platform,
        NativeObservationV1::Pointer,
        NativeOutcomeV1::Observed,
    );
    control_on_pointer.captured_generation = Some(generation_zero());
    control_on_pointer.surface = Some(surface());
    control_on_pointer.target = Some(HeadlessPointerTargetV1::StaticControl);
    control_on_pointer.control = Some(0);
    let invalid_tuple = NativeTraceStepV1::new(
        NativeTraceStageV1::Manifest,
        NativeObservationV1::Resumed,
        NativeOutcomeV1::Matched,
    );

    for invalid in [control_on_pointer, invalid_tuple] {
        let storage_called = Cell::new(false);
        let mut trace = NativeTraceV1::new();
        assert_eq!(
            trace
                .record_with_reserver_for_test(SchedulerTick::new(0), invalid, |_| {
                    storage_called.set(true);
                    Ok(())
                })
                .expect_err("invalid applicability must fail"),
            NativeTraceErrorKindV1::InvalidApplicability
        );
        assert!(!storage_called.get());
        assert!(trace.is_empty());
    }
}

fn deferred_surface() -> NativeTraceStepV1 {
    let mut step = NativeTraceStepV1::new(
        NativeTraceStageV1::Platform,
        NativeObservationV1::Surface,
        NativeOutcomeV1::Deferred,
    );
    step.captured_generation = Some(generation_zero());
    step.surface = Some(surface());
    step
}

fn published_surface() -> NativeTraceStepV1 {
    let mut step = NativeTraceStepV1::new(
        NativeTraceStageV1::Platform,
        NativeObservationV1::Surface,
        NativeOutcomeV1::Published,
    );
    step.published_generation = Some(generation_zero());
    step.surface = Some(surface());
    step
}

fn oracle_match() -> NativeTraceStepV1 {
    let mut step = NativeTraceStepV1::new(
        NativeTraceStageV1::Oracle,
        NativeObservationV1::Surface,
        NativeOutcomeV1::Matched,
    );
    step.published_generation = Some(generation_zero());
    step.surface = Some(surface());
    step
}

fn rejected_frame() -> NativeTraceStepV1 {
    let mut step = NativeTraceStepV1::new(
        NativeTraceStageV1::Renderer,
        NativeObservationV1::Frame,
        NativeOutcomeV1::Rejected,
    );
    step.surface = Some(surface());
    step.frame = Some(0);
    step
}

fn presented_frame() -> NativeTraceStepV1 {
    let mut step = NativeTraceStepV1::new(
        NativeTraceStageV1::Renderer,
        NativeObservationV1::Present,
        NativeOutcomeV1::Completed,
    );
    step.surface = Some(surface());
    step.frame = Some(0);
    step.submission = Some(NativeTraceSubmissionV1::new(0, 0));
    step
}

fn renderer_loss_control() -> NativeTraceStepV1 {
    let mut step = NativeTraceStepV1::new(
        NativeTraceStageV1::Scheduler,
        NativeObservationV1::Present,
        NativeOutcomeV1::Accepted,
    );
    step.scheduler_turn = Some(0);
    step.surface = Some(surface());
    step.frame = Some(0);
    step.submission = Some(NativeTraceSubmissionV1::new(0, 0));
    step.control = Some(0);
    step
}

fn shutdown_control(outcome: NativeOutcomeV1) -> NativeTraceStepV1 {
    let mut step = NativeTraceStepV1::new(
        NativeTraceStageV1::Scheduler,
        NativeObservationV1::Shutdown,
        outcome,
    );
    step.scheduler_turn = Some(0);
    step.control = Some(0);
    step
}

fn surface() -> NativeSurfaceTupleV1 {
    let mut state = NativeSurfaceStateV1::new();
    state
        .observe(NativePhysicalExtentV1::new(240, 180), 2.0)
        .expect("fixed surface should validate");
    state.pending_tuple().expect("surface tuple should exist")
}
