use std::cell::Cell;

use fenestra_ui_runtime::prototype::{SchedulerState, SchedulerTick};
use fenestra_ui_testkit::prototype::HeadlessPointerTargetV1;

use super::super::surface::NativeSurfaceObservationV1;
use super::super::trace::{
    NativeFailureCauseV1, NativeInputSourceV1, NativeObservationV1, NativeOutcomeV1,
    NativeTraceErrorKindV1, NativeTraceEventV1, NativeTraceLaneStatsV1, NativeTraceLimitKindV1,
    NativeTracePendingV1, NativeTraceStageV1, NativeTraceStepV1, NativeTraceSubmissionV1,
    NativeTraceV1,
};
use super::super::{NativePhysicalExtentV1, NativeSurfaceStateV1, NativeSurfaceTupleV1};
use super::{generation_zero, trace_step};

#[test]
fn representative_scalar_state_is_typed_and_lossless() {
    let mut trace = NativeTraceV1::new();
    let mut pointer = trace_step(
        NativeTraceStageV1::Platform,
        NativeObservationV1::Pointer,
        NativeOutcomeV1::Observed,
    );
    pointer.captured_generation = Some(generation_zero());
    pointer.input_source = Some(NativeInputSourceV1::Native);
    pointer.surface = Some(surface());
    pointer.target = Some(HeadlessPointerTargetV1::StaticControl);
    pointer.scheduler_state = Some(SchedulerState::ShutdownQueued);
    pointer.current_generation = Some(generation_zero());
    pointer.redraw_armed = true;
    pointer.pending = NativeTracePendingV1::new(1, 1, 1);
    pointer.deferred = NativeTraceLaneStatsV1::new(1, 80);
    pointer.visual = NativeTraceLaneStatsV1::new(1, 40);

    trace
        .record(SchedulerTick::new(9), pointer)
        .expect("complete pointer state should be applicable");
    let event = trace.events()[0];

    assert_eq!(event.stage(), NativeTraceStageV1::Platform);
    assert_eq!(event.observation(), NativeObservationV1::Pointer);
    assert_eq!(event.outcome(), NativeOutcomeV1::Observed);
    assert_eq!(event.input_source(), Some(NativeInputSourceV1::Native));
    assert_eq!(event.captured_generation(), Some(generation_zero()));
    assert_eq!(event.published_generation(), None);
    assert_eq!(event.surface(), Some(surface()));
    assert_eq!(event.surface_observation(), None);
    assert_eq!(event.target(), Some(HeadlessPointerTargetV1::StaticControl));
    assert_eq!(event.scheduler_state(), SchedulerState::ShutdownQueued);
    assert_eq!(event.current_generation(), generation_zero());
    assert_eq!(event.pending(), NativeTracePendingV1::new(1, 1, 1));
    assert_eq!(event.pending().surface(), 1);
    assert_eq!(event.pending().pointer(), 1);
    assert_eq!(event.pending().presenter(), 1);
    assert_eq!(event.deferred(), NativeTraceLaneStatsV1::new(1, 80));
    assert_eq!(event.visual(), NativeTraceLaneStatsV1::new(1, 40));
    assert_eq!(event.deferred().items(), 1);
    assert_eq!(event.deferred().accounted_bytes(), 80);
    assert!(event.redraw_armed());
}

#[test]
fn frame_submission_control_and_four_scheduler_lanes_are_closed_scalars() {
    let mut trace = NativeTraceV1::new();
    let mut offered = trace_step(
        NativeTraceStageV1::Scheduler,
        NativeObservationV1::Frame,
        NativeOutcomeV1::Offered,
    );
    offered.scheduler_turn = Some(0);
    offered.surface = Some(surface());
    offered.frame = Some(2);
    offered.controls = NativeTraceLaneStatsV1::new(1, 32);
    offered.in_flight = NativeTraceLaneStatsV1::new(1, 40);
    let mut accepted = trace_step(
        NativeTraceStageV1::Scheduler,
        NativeObservationV1::Frame,
        NativeOutcomeV1::Accepted,
    );
    accepted.scheduler_turn = Some(1);
    accepted.surface = Some(surface());
    accepted.frame = Some(2);
    accepted.submission = Some(NativeTraceSubmissionV1::new(0, 1));
    accepted.staging_digest = Some(0xcbf2_9ce4_8422_2325);
    let mut completion = trace_step(
        NativeTraceStageV1::Scheduler,
        NativeObservationV1::Completion,
        NativeOutcomeV1::Accepted,
    );
    completion.scheduler_turn = Some(2);
    completion.surface = Some(surface());
    completion.submission = Some(NativeTraceSubmissionV1::new(0, 1));
    completion.control = Some(3);

    trace
        .record(SchedulerTick::new(11), offered)
        .expect("offer identity should record");
    trace
        .record(SchedulerTick::new(11), accepted)
        .expect("accepted submission should record");
    trace
        .record(SchedulerTick::new(12), completion)
        .expect("accepted completion control should record");

    assert_eq!(trace.events()[0].scheduler_turn(), Some(0));
    assert_eq!(trace.events()[0].frame(), Some(2));
    assert_eq!(
        trace.events()[0].controls(),
        NativeTraceLaneStatsV1::new(1, 32)
    );
    assert_eq!(
        trace.events()[0].in_flight(),
        NativeTraceLaneStatsV1::new(1, 40)
    );
    assert_eq!(trace.events()[1].frame(), Some(2));
    assert_eq!(
        trace.events()[1].submission(),
        Some(NativeTraceSubmissionV1::new(0, 1))
    );
    assert_eq!(trace.events()[2].control(), Some(3));
}

#[test]
fn invalid_applicability_is_rejected_before_storage_without_a_prefix() {
    let mut target_on_build = observed_build();
    target_on_build.target = Some(HeadlessPointerTargetV1::Key(10));
    let mut published_observation = observed_build();
    published_observation.published_generation = Some(generation_zero());
    let mut oversized_pending = observed_build();
    oversized_pending.pending = NativeTracePendingV1::new(2, 0, 0);
    let mut captured_on_build = observed_build();
    captured_on_build.captured_generation = Some(generation_zero());
    let mut submission_without_frame = trace_step(
        NativeTraceStageV1::Renderer,
        NativeObservationV1::Frame,
        NativeOutcomeV1::Accepted,
    );
    submission_without_frame.surface = Some(surface());
    submission_without_frame.submission = Some(NativeTraceSubmissionV1::new(0, 0));
    let mut frame_without_submission = submission_without_frame;
    frame_without_submission.submission = None;
    frame_without_submission.frame = Some(0);

    for invalid in [
        target_on_build,
        published_observation,
        oversized_pending,
        captured_on_build,
        submission_without_frame,
        frame_without_submission,
    ] {
        let storage_called = Cell::new(false);
        let mut trace = NativeTraceV1::new();
        let error = trace
            .record_with_reserver_for_test(SchedulerTick::new(0), invalid, |_| {
                storage_called.set(true);
                Ok(())
            })
            .expect_err("inapplicable fields must fail closed");
        assert_eq!(error, NativeTraceErrorKindV1::InvalidApplicability);
        assert!(!storage_called.get());
        assert!(trace.events().is_empty());
    }
}

#[test]
fn scheduler_turns_are_dense_and_only_appear_on_scheduler_events() {
    let mut trace = NativeTraceV1::new();
    trace
        .record(SchedulerTick::new(0), scheduler_offer(0))
        .expect("first scheduler turn should fit");
    let prefix = trace.events().to_vec();

    let mut turn_on_build = observed_build();
    turn_on_build.scheduler_turn = Some(1);
    for invalid in [
        turn_on_build,
        scheduler_offer_without_turn(),
        scheduler_offer(0),
        scheduler_offer(2),
    ] {
        assert_eq!(
            trace
                .record(SchedulerTick::new(0), invalid)
                .expect_err("scheduler turn applicability must fail closed"),
            NativeTraceErrorKindV1::InvalidApplicability
        );
        assert_eq!(trace.events(), prefix);
    }

    trace
        .record(SchedulerTick::new(0), scheduler_offer(1))
        .expect("next dense scheduler turn should fit");
    assert_eq!(trace.events()[1].scheduler_turn(), Some(1));
}

#[test]
fn environment_failures_retain_surface_and_reject_presentation_identity() {
    let mut trace = NativeTraceV1::new();
    let failed = environment_failure(NativeFailureCauseV1::EnvironmentScaleChanged);

    trace
        .record(SchedulerTick::new(12), failed)
        .expect("environment failure may retain only its observed surface");
    let event = trace.events()[0];
    assert_eq!(event.surface(), None);
    assert_eq!(event.surface_observation(), Some(surface_observation()));
    assert_eq!(event.frame(), None);
    assert_eq!(event.submission(), None);
    assert_eq!(event.control(), None);

    for cause in [
        NativeFailureCauseV1::EnvironmentScaleChanged,
        NativeFailureCauseV1::SurfaceRepaintUnavailable,
    ] {
        let mut with_frame = environment_failure(cause);
        with_frame.frame = Some(1);
        let mut with_submission = environment_failure(cause);
        with_submission.frame = Some(1);
        with_submission.submission = Some(NativeTraceSubmissionV1::new(0, 1));
        let mut with_control = environment_failure(cause);
        with_control.control = Some(1);
        for invalid in [with_frame, with_submission, with_control] {
            let storage_called = Cell::new(false);
            let mut invalid_trace = NativeTraceV1::new();
            assert_eq!(
                invalid_trace
                    .record_with_reserver_for_test(SchedulerTick::new(12), invalid, |_| {
                        storage_called.set(true);
                        Ok(())
                    })
                    .expect_err("environment failure cannot carry presentation identity"),
                NativeTraceErrorKindV1::InvalidApplicability
            );
            assert!(!storage_called.get());
        }
    }
}

#[test]
fn vocabularies_are_closed_ordered_and_privacy_safe() {
    assert_eq!(
        NativeInputSourceV1::ALL,
        [NativeInputSourceV1::Native, NativeInputSourceV1::Scripted]
    );
    assert_eq!(
        NativeTraceStageV1::ALL,
        [
            NativeTraceStageV1::Manifest,
            NativeTraceStageV1::Shell,
            NativeTraceStageV1::Platform,
            NativeTraceStageV1::Scheduler,
            NativeTraceStageV1::Renderer,
            NativeTraceStageV1::Oracle,
        ]
    );
    assert_eq!(
        NativeObservationV1::ALL,
        [
            NativeObservationV1::Build,
            NativeObservationV1::Resumed,
            NativeObservationV1::Surface,
            NativeObservationV1::Scale,
            NativeObservationV1::Pointer,
            NativeObservationV1::Redraw,
            NativeObservationV1::Frame,
            NativeObservationV1::Present,
            NativeObservationV1::Close,
            NativeObservationV1::Completion,
            NativeObservationV1::Shutdown,
            NativeObservationV1::Timeout,
        ]
    );
    assert_eq!(
        NativeOutcomeV1::ALL,
        [
            NativeOutcomeV1::Observed,
            NativeOutcomeV1::Coalesced,
            NativeOutcomeV1::Ignored,
            NativeOutcomeV1::Deferred,
            NativeOutcomeV1::Published,
            NativeOutcomeV1::Armed,
            NativeOutcomeV1::Offered,
            NativeOutcomeV1::Accepted,
            NativeOutcomeV1::Rejected,
            NativeOutcomeV1::Completed,
            NativeOutcomeV1::Matched,
            NativeOutcomeV1::Stopped,
            NativeOutcomeV1::Failed(NativeFailureCauseV1::Trace),
        ]
    );
    assert_eq!(
        NativeFailureCauseV1::ALL,
        [
            NativeFailureCauseV1::InvalidScale,
            NativeFailureCauseV1::InvalidPoint,
            NativeFailureCauseV1::Arithmetic,
            NativeFailureCauseV1::WidthLimit,
            NativeFailureCauseV1::HeightLimit,
            NativeFailureCauseV1::PixelLimit,
            NativeFailureCauseV1::ByteLimit,
            NativeFailureCauseV1::UnsupportedAlpha,
            NativeFailureCauseV1::Storage,
            NativeFailureCauseV1::EnvironmentScaleChanged,
            NativeFailureCauseV1::EnvironmentSurfaceChanged,
            NativeFailureCauseV1::SurfaceRepaintUnavailable,
            NativeFailureCauseV1::Runtime,
            NativeFailureCauseV1::Oracle,
            NativeFailureCauseV1::Scheduler,
            NativeFailureCauseV1::PrePresent,
            NativeFailureCauseV1::Presenter,
            NativeFailureCauseV1::Trace,
            NativeFailureCauseV1::Timeout,
            NativeFailureCauseV1::Invariant,
        ]
    );
    assert_eq!(
        NativeTraceLimitKindV1::ALL,
        [
            NativeTraceLimitKindV1::Events,
            NativeTraceLimitKindV1::AccountedBytes,
        ]
    );
    assert_copy::<NativeTraceEventV1>();
    assert_copy::<NativeTraceStepV1>();

    let mut trace = NativeTraceV1::new();
    trace
        .record(SchedulerTick::new(0), observed_build())
        .expect("one event should fit");
    assert_eq!(
        format!("{trace:?}"),
        "NativeTraceV1 { event_count: 1, accounted_bytes: 192 }"
    );
}

fn observed_build() -> NativeTraceStepV1 {
    trace_step(
        NativeTraceStageV1::Manifest,
        NativeObservationV1::Build,
        NativeOutcomeV1::Observed,
    )
}

fn scheduler_offer(turn: u64) -> NativeTraceStepV1 {
    let mut step = scheduler_offer_without_turn();
    step.scheduler_turn = Some(turn);
    step
}

fn scheduler_offer_without_turn() -> NativeTraceStepV1 {
    let mut step = trace_step(
        NativeTraceStageV1::Scheduler,
        NativeObservationV1::Frame,
        NativeOutcomeV1::Offered,
    );
    step.surface = Some(surface());
    step.frame = Some(0);
    step
}

fn environment_failure(cause: NativeFailureCauseV1) -> NativeTraceStepV1 {
    let observation = if cause == NativeFailureCauseV1::EnvironmentScaleChanged {
        NativeObservationV1::Scale
    } else {
        NativeObservationV1::Surface
    };
    let mut step = trace_step(
        NativeTraceStageV1::Platform,
        observation,
        NativeOutcomeV1::Failed(cause),
    );
    if cause == NativeFailureCauseV1::EnvironmentScaleChanged {
        step.surface_observation = Some(surface_observation());
    } else {
        step.surface = Some(surface());
    }
    step
}

fn surface_observation() -> NativeSurfaceObservationV1 {
    NativeSurfaceObservationV1::try_new(NativePhysicalExtentV1::new(241, 181), 2.01)
        .expect("fixed observation should normalize")
}

fn surface() -> NativeSurfaceTupleV1 {
    let mut state = NativeSurfaceStateV1::new();
    state
        .observe(NativePhysicalExtentV1::new(240, 180), 2.0)
        .expect("fixed surface should validate");
    state.pending_tuple().expect("surface tuple should exist")
}

fn assert_copy<T: Copy>() {}
