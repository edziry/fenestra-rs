use std::cell::Cell;

use fenestra_ui_runtime::prototype::SchedulerTick;
use fenestra_ui_testkit::prototype::HeadlessPointerTargetV1;

use super::super::trace::{
    NativeInputSourceV1, NativeObservationV1, NativeOutcomeV1, NativeTraceErrorKindV1,
    NativeTraceStageV1, NativeTraceStepV1, NativeTraceSubmissionV1, NativeTraceV1,
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
    control_on_pointer.input_source = Some(NativeInputSourceV1::Native);
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

#[test]
fn incomplete_or_extraneous_presentation_identity_is_rejected() {
    let mut present_without_submission = presented_frame();
    present_without_submission.submission = None;
    let mut offer_with_submission = rejected_frame();
    offer_with_submission.stage = NativeTraceStageV1::Scheduler;
    offer_with_submission.outcome = NativeOutcomeV1::Offered;
    offer_with_submission.scheduler_turn = Some(0);
    offer_with_submission.submission = Some(NativeTraceSubmissionV1::new(0, 0));
    let mut reject_with_submission = rejected_frame();
    reject_with_submission.submission = Some(NativeTraceSubmissionV1::new(0, 0));
    let mut loss_without_control = renderer_loss_control();
    loss_without_control.control = None;
    let mut accepted_shutdown_without_control = shutdown_control(NativeOutcomeV1::Accepted);
    accepted_shutdown_without_control.control = None;
    let mut stopped_shutdown_without_control = shutdown_control(NativeOutcomeV1::Stopped);
    stopped_shutdown_without_control.control = None;
    let environment_without_surface = environment_failure_without_surface(
        super::super::trace::NativeFailureCauseV1::EnvironmentScaleChanged,
    );
    let repaint_without_surface = environment_failure_without_surface(
        super::super::trace::NativeFailureCauseV1::SurfaceRepaintUnavailable,
    );

    for invalid in [
        present_without_submission,
        offer_with_submission,
        reject_with_submission,
        loss_without_control,
        accepted_shutdown_without_control,
        stopped_shutdown_without_control,
        environment_without_surface,
        repaint_without_surface,
    ] {
        let storage_called = Cell::new(false);
        let mut trace = NativeTraceV1::new();
        assert_eq!(
            trace
                .record_with_reserver_for_test(SchedulerTick::new(0), invalid, |_| {
                    storage_called.set(true);
                    Ok(())
                })
                .expect_err("identity shape must fail before storage"),
            NativeTraceErrorKindV1::InvalidApplicability
        );
        assert!(!storage_called.get());
        assert!(trace.is_empty());
    }
}

#[test]
fn observations_that_depend_on_a_surface_reject_a_missing_tuple() {
    let mut pointer = NativeTraceStepV1::new(
        NativeTraceStageV1::Platform,
        NativeObservationV1::Pointer,
        NativeOutcomeV1::Observed,
    );
    pointer.captured_generation = Some(generation_zero());
    pointer.input_source = Some(NativeInputSourceV1::Native);
    pointer.target = Some(HeadlessPointerTargetV1::StaticControl);
    let mut deferred = deferred_surface();
    deferred.surface = None;
    let mut published = published_surface();
    published.surface = None;
    let mut matched = oracle_match();
    matched.surface = None;
    let mut rejected = rejected_frame();
    rejected.surface = None;
    let mut presented = presented_frame();
    presented.surface = None;

    for invalid in [pointer, deferred, published, matched, rejected, presented] {
        let storage_called = Cell::new(false);
        let mut trace = NativeTraceV1::new();
        assert_eq!(
            trace
                .record_with_reserver_for_test(SchedulerTick::new(0), invalid, |_| {
                    storage_called.set(true);
                    Ok(())
                })
                .expect_err("surface-dependent step must carry its tuple"),
            NativeTraceErrorKindV1::InvalidApplicability
        );
        assert!(!storage_called.get());
        assert!(trace.is_empty());
    }
}

fn deferred_surface() -> NativeTraceStepV1 {
    let mut step = NativeTraceStepV1::new(
        NativeTraceStageV1::Scheduler,
        NativeObservationV1::Surface,
        NativeOutcomeV1::Deferred,
    );
    step.scheduler_turn = Some(0);
    step.captured_generation = Some(generation_zero());
    step.surface = Some(surface());
    step
}

fn published_surface() -> NativeTraceStepV1 {
    let mut step = NativeTraceStepV1::new(
        NativeTraceStageV1::Scheduler,
        NativeObservationV1::Surface,
        NativeOutcomeV1::Published,
    );
    step.scheduler_turn = Some(0);
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

fn environment_failure_without_surface(
    cause: super::super::trace::NativeFailureCauseV1,
) -> NativeTraceStepV1 {
    NativeTraceStepV1::new(
        NativeTraceStageV1::Platform,
        NativeObservationV1::Scale,
        NativeOutcomeV1::Failed(cause),
    )
}

fn surface() -> NativeSurfaceTupleV1 {
    let mut state = NativeSurfaceStateV1::new();
    state
        .observe(NativePhysicalExtentV1::new(240, 180), 2.0)
        .expect("fixed surface should validate");
    state.pending_tuple().expect("surface tuple should exist")
}
