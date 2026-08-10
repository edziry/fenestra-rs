use std::cell::Cell;

use fenestra_ui_runtime::prototype::SchedulerTick;
use fenestra_ui_testkit::prototype::HeadlessPointerTargetV1;

use super::super::trace::{
    NativeFailureCauseV1, NativeInputSourceV1, NativeObservationV1, NativeOutcomeV1,
    NativeTraceErrorKindV1, NativeTraceStageV1, NativeTraceStepV1, NativeTraceSubmissionV1,
    NativeTraceV1,
};
use super::super::{NativePhysicalExtentV1, NativeSurfaceStateV1, NativeSurfaceTupleV1};
use super::{generation_one, generation_zero, trace_step};

#[test]
fn required_generation_frame_and_control_paths_are_accepted() {
    for valid in [
        deferred_surface(),
        published_surface(),
        oracle_match(),
        rejected_frame(),
        presented_frame(),
        renderer_loss_control(NativeOutcomeV1::Accepted),
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
    let mut control_on_pointer = trace_step(
        NativeTraceStageV1::Platform,
        NativeObservationV1::Pointer,
        NativeOutcomeV1::Observed,
    );
    control_on_pointer.captured_generation = Some(generation_zero());
    control_on_pointer.input_source = Some(NativeInputSourceV1::Native);
    control_on_pointer.surface = Some(surface());
    control_on_pointer.target = Some(HeadlessPointerTargetV1::StaticControl);
    control_on_pointer.control = Some(0);
    let invalid_tuple = trace_step(
        NativeTraceStageV1::Manifest,
        NativeObservationV1::Resumed,
        NativeOutcomeV1::Matched,
    );

    for invalid in [control_on_pointer, invalid_tuple] {
        assert_invalid(invalid);
    }
}

#[test]
fn presentation_loss_and_completion_identity_shapes_are_exact() {
    for (valid, shape) in [
        (
            scheduler_frame(NativeOutcomeV1::Armed),
            (false, false, false),
        ),
        (
            scheduler_frame(NativeOutcomeV1::Offered),
            (true, false, false),
        ),
        (
            scheduler_frame(NativeOutcomeV1::Accepted),
            (true, true, false),
        ),
        (
            scheduler_frame(NativeOutcomeV1::Rejected),
            (true, false, false),
        ),
        (rejected_frame(), (true, false, false)),
        (presented_frame(), (true, true, false)),
        (
            renderer_loss_control(NativeOutcomeV1::Accepted),
            (true, true, true),
        ),
        (
            renderer_loss_control(NativeOutcomeV1::Failed(NativeFailureCauseV1::Presenter)),
            (true, true, true),
        ),
        (completion(NativeOutcomeV1::Accepted), (false, true, true)),
        (completion(NativeOutcomeV1::Completed), (false, true, true)),
    ] {
        assert_exact_identity(valid, shape);
    }
}

#[test]
fn snapshots_require_scheduler_state_and_current_generation() {
    let mut missing_state = trace_step(
        NativeTraceStageV1::Manifest,
        NativeObservationV1::Build,
        NativeOutcomeV1::Observed,
    );
    missing_state.scheduler_state = None;
    let mut missing_generation = missing_state;
    missing_generation.scheduler_state = trace_step(
        NativeTraceStageV1::Manifest,
        NativeObservationV1::Build,
        NativeOutcomeV1::Observed,
    )
    .scheduler_state;
    missing_generation.current_generation = None;
    assert_invalid(missing_state);
    assert_invalid(missing_generation);
}

#[test]
fn publication_generation_is_required_or_optional_only_on_exact_paths() {
    let mut published = published_surface();
    assert_valid(published);
    published.published_generation = None;
    assert_invalid(published);

    let mut matched = oracle_match();
    assert_valid(matched);
    matched.published_generation = None;
    assert_valid(matched);

    let mut combined_offer = scheduler_frame(NativeOutcomeV1::Offered);
    assert_valid(combined_offer);
    combined_offer.published_generation = Some(generation_zero());
    assert_valid(combined_offer);

    for mut invalid in [
        deferred_surface(),
        scheduler_frame(NativeOutcomeV1::Accepted),
        presented_frame(),
        completion(NativeOutcomeV1::Completed),
        oracle_failure(),
    ] {
        invalid.published_generation = Some(generation_zero());
        assert_invalid(invalid);
    }
}

#[test]
fn staging_digest_is_required_only_on_an_accepted_frame() {
    let accepted = scheduler_frame(NativeOutcomeV1::Accepted);
    assert_valid(accepted);
    let mut missing = accepted;
    missing.staging_digest = None;
    assert_invalid(missing);

    for mut invalid in [
        scheduler_frame(NativeOutcomeV1::Offered),
        rejected_frame(),
        presented_frame(),
        completion(NativeOutcomeV1::Completed),
    ] {
        invalid.staging_digest = Some(0xcbf2_9ce4_8422_2325);
        assert_invalid(invalid);
    }
}

#[test]
fn scheduler_post_state_and_generations_must_match_the_typed_outcome() {
    let mut stopped = shutdown_control(NativeOutcomeV1::Stopped);
    assert_valid(stopped);
    stopped.scheduler_state = Some(fenestra_ui_runtime::prototype::SchedulerState::Running);
    assert_invalid(stopped);

    let mut loss = renderer_loss_control(NativeOutcomeV1::Failed(NativeFailureCauseV1::Presenter));
    assert_valid(loss);
    loss.scheduler_state = Some(fenestra_ui_runtime::prototype::SchedulerState::Running);
    assert_invalid(loss);

    for mut invalid in [deferred_surface(), published_surface(), oracle_match()] {
        invalid.current_generation = Some(generation_one());
        assert_invalid(invalid);
    }
}

#[test]
fn observations_that_depend_on_a_surface_reject_a_missing_tuple() {
    let mut pointer = trace_step(
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
        assert_invalid(invalid);
    }
}

fn assert_valid(valid: NativeTraceStepV1) {
    let mut trace = NativeTraceV1::new();
    trace
        .record(SchedulerTick::new(0), valid)
        .expect("exact applicability shape should record");
}

fn assert_invalid(invalid: NativeTraceStepV1) {
    let storage_called = Cell::new(false);
    let mut trace = NativeTraceV1::new();
    assert_eq!(
        trace
            .record_with_reserver_for_test(SchedulerTick::new(0), invalid, |_| {
                storage_called.set(true);
                Ok(())
            })
            .expect_err("invalid applicability must fail before storage"),
        NativeTraceErrorKindV1::InvalidApplicability
    );
    assert!(!storage_called.get());
    assert!(trace.is_empty());
}

fn assert_exact_identity(valid: NativeTraceStepV1, shape: (bool, bool, bool)) {
    assert_valid(valid);
    let mut frame = valid;
    frame.frame = if shape.0 { None } else { Some(0) };
    assert_invalid(frame);
    let mut submission = valid;
    submission.submission = if shape.1 {
        None
    } else {
        Some(NativeTraceSubmissionV1::new(0, 0))
    };
    assert_invalid(submission);
    let mut control = valid;
    control.control = if shape.2 { None } else { Some(0) };
    assert_invalid(control);
}

fn deferred_surface() -> NativeTraceStepV1 {
    let mut step = trace_step(
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
    let mut step = trace_step(
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
    let mut step = trace_step(
        NativeTraceStageV1::Oracle,
        NativeObservationV1::Surface,
        NativeOutcomeV1::Matched,
    );
    step.published_generation = Some(generation_zero());
    step.surface = Some(surface());
    step
}

fn rejected_frame() -> NativeTraceStepV1 {
    let mut step = trace_step(
        NativeTraceStageV1::Renderer,
        NativeObservationV1::Frame,
        NativeOutcomeV1::Rejected,
    );
    step.surface = Some(surface());
    step.frame = Some(0);
    step
}

fn presented_frame() -> NativeTraceStepV1 {
    let mut step = trace_step(
        NativeTraceStageV1::Renderer,
        NativeObservationV1::Present,
        NativeOutcomeV1::Completed,
    );
    step.surface = Some(surface());
    step.frame = Some(0);
    step.submission = Some(NativeTraceSubmissionV1::new(0, 0));
    step
}

fn renderer_loss_control(outcome: NativeOutcomeV1) -> NativeTraceStepV1 {
    let mut step = trace_step(
        NativeTraceStageV1::Scheduler,
        NativeObservationV1::Present,
        outcome,
    );
    step.scheduler_turn = Some(0);
    step.surface = Some(surface());
    step.frame = Some(0);
    step.submission = Some(NativeTraceSubmissionV1::new(0, 0));
    step.control = Some(0);
    if outcome == NativeOutcomeV1::Failed(NativeFailureCauseV1::Presenter) {
        step.scheduler_state = Some(fenestra_ui_runtime::prototype::SchedulerState::Faulted);
    }
    step
}

fn scheduler_frame(outcome: NativeOutcomeV1) -> NativeTraceStepV1 {
    let mut step = trace_step(
        NativeTraceStageV1::Scheduler,
        NativeObservationV1::Frame,
        outcome,
    );
    step.scheduler_turn = Some(0);
    step.surface = Some(surface());
    if outcome != NativeOutcomeV1::Armed {
        step.frame = Some(0);
    }
    if outcome == NativeOutcomeV1::Accepted {
        step.submission = Some(NativeTraceSubmissionV1::new(0, 0));
        step.staging_digest = Some(0xcbf2_9ce4_8422_2325);
    }
    step
}

fn completion(outcome: NativeOutcomeV1) -> NativeTraceStepV1 {
    let mut step = trace_step(
        NativeTraceStageV1::Scheduler,
        NativeObservationV1::Completion,
        outcome,
    );
    step.scheduler_turn = Some(0);
    step.surface = Some(surface());
    step.submission = Some(NativeTraceSubmissionV1::new(0, 0));
    step.control = Some(0);
    step
}

fn oracle_failure() -> NativeTraceStepV1 {
    let mut step = trace_step(
        NativeTraceStageV1::Oracle,
        NativeObservationV1::Surface,
        NativeOutcomeV1::Failed(NativeFailureCauseV1::Oracle),
    );
    step.surface = Some(surface());
    step
}

fn shutdown_control(outcome: NativeOutcomeV1) -> NativeTraceStepV1 {
    let mut step = trace_step(
        NativeTraceStageV1::Scheduler,
        NativeObservationV1::Shutdown,
        outcome,
    );
    step.scheduler_turn = Some(0);
    step.control = Some(0);
    if outcome == NativeOutcomeV1::Stopped {
        step.scheduler_state = Some(fenestra_ui_runtime::prototype::SchedulerState::Stopped);
    }
    step
}

fn surface() -> NativeSurfaceTupleV1 {
    let mut state = NativeSurfaceStateV1::new();
    state
        .observe(NativePhysicalExtentV1::new(240, 180), 2.0)
        .expect("fixed surface should validate");
    state.pending_tuple().expect("surface tuple should exist")
}
