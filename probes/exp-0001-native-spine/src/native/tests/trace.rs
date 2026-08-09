use std::cell::Cell;

use fenestra_ui_runtime::prototype::{HeadlessSurface, SchedulerTick};
use fenestra_ui_testkit::prototype::HeadlessPointerTargetV1;

use super::super::trace::{
    NativeFailureCauseV1, NativeObservationV1, NativeOutcomeV1, NativeTraceErrorKindV1,
    NativeTraceEventV1, NativeTraceLaneStatsV1, NativeTraceLimitKindV1, NativeTracePendingV1,
    NativeTraceStageV1, NativeTraceStepV1, NativeTraceSubmissionV1, NativeTraceSurfaceV1,
    NativeTraceV1,
};
use super::super::{NativePhysicalExtentV1, NativeScaleFactorV1};
use super::generation_zero;

#[test]
fn representative_scalar_state_is_typed_and_lossless() {
    let mut trace = NativeTraceV1::new();
    let mut pointer = NativeTraceStepV1::new(
        NativeTraceStageV1::Platform,
        NativeObservationV1::Pointer,
        NativeOutcomeV1::Observed,
    );
    pointer.captured_generation = Some(generation_zero());
    pointer.surface = Some(surface());
    pointer.target = Some(HeadlessPointerTargetV1::StaticControl);
    pointer.redraw_armed = true;
    pointer.pending = NativeTracePendingV1::new(1, 1, 0);
    pointer.deferred = NativeTraceLaneStatsV1::new(1, 80);
    pointer.visual = NativeTraceLaneStatsV1::new(1, 40);

    trace
        .record(SchedulerTick::new(9), pointer)
        .expect("complete pointer state should be applicable");
    let event = trace.events()[0];

    assert_eq!(event.stage(), NativeTraceStageV1::Platform);
    assert_eq!(event.observation(), NativeObservationV1::Pointer);
    assert_eq!(event.outcome(), NativeOutcomeV1::Observed);
    assert_eq!(event.captured_generation(), Some(generation_zero()));
    assert_eq!(event.published_generation(), None);
    assert_eq!(event.surface(), Some(surface()));
    assert_eq!(event.target(), Some(HeadlessPointerTargetV1::StaticControl));
    assert_eq!(event.pending(), NativeTracePendingV1::new(1, 1, 0));
    assert_eq!(event.deferred(), NativeTraceLaneStatsV1::new(1, 80));
    assert_eq!(event.visual(), NativeTraceLaneStatsV1::new(1, 40));
    assert!(event.redraw_armed());
}

#[test]
fn frame_submission_control_and_scheduler_turn_are_closed_scalars() {
    let mut trace = NativeTraceV1::new();
    let mut offered = NativeTraceStepV1::new(
        NativeTraceStageV1::Scheduler,
        NativeObservationV1::Frame,
        NativeOutcomeV1::Offered,
    );
    offered.scheduler_turn = Some(4);
    offered.surface = Some(surface());
    offered.frame = Some(2);
    offered.in_flight = NativeTraceLaneStatsV1::new(1, 40);
    offered.renderer = NativeTraceLaneStatsV1::new(1, 96);
    let mut accepted = NativeTraceStepV1::new(
        NativeTraceStageV1::Renderer,
        NativeObservationV1::Frame,
        NativeOutcomeV1::Accepted,
    );
    accepted.surface = Some(surface());
    accepted.frame = Some(2);
    accepted.submission = Some(NativeTraceSubmissionV1::new(0, 1));
    let mut completion = NativeTraceStepV1::new(
        NativeTraceStageV1::Scheduler,
        NativeObservationV1::Completion,
        NativeOutcomeV1::Accepted,
    );
    completion.scheduler_turn = Some(5);
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

    assert_eq!(trace.events()[0].scheduler_turn(), Some(4));
    assert_eq!(trace.events()[0].frame(), Some(2));
    assert_eq!(
        trace.events()[0].in_flight(),
        NativeTraceLaneStatsV1::new(1, 40)
    );
    assert_eq!(
        trace.events()[0].renderer(),
        NativeTraceLaneStatsV1::new(1, 96)
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
    let mut submission_without_frame = observed_build();
    submission_without_frame.submission = Some(NativeTraceSubmissionV1::new(0, 0));

    for invalid in [
        target_on_build,
        published_observation,
        oversized_pending,
        submission_without_frame,
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
fn environment_failure_retains_surface_without_presentation_identity() {
    let mut trace = NativeTraceV1::new();
    let mut failed = NativeTraceStepV1::new(
        NativeTraceStageV1::Platform,
        NativeObservationV1::Scale,
        NativeOutcomeV1::Failed(NativeFailureCauseV1::EnvironmentScaleChanged),
    );
    failed.surface = Some(surface());

    trace
        .record(SchedulerTick::new(12), failed)
        .expect("environment failure may retain only its observed surface");
    let event = trace.events()[0];
    assert_eq!(event.surface(), Some(surface()));
    assert_eq!(event.frame(), None);
    assert_eq!(event.submission(), None);
    assert_eq!(event.control(), None);
}

#[test]
fn vocabularies_are_closed_ordered_and_privacy_safe() {
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
            NativeFailureCauseV1::EnvironmentScaleChanged,
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
    NativeTraceStepV1::new(
        NativeTraceStageV1::Manifest,
        NativeObservationV1::Build,
        NativeOutcomeV1::Observed,
    )
}

fn surface() -> NativeTraceSurfaceV1 {
    NativeTraceSurfaceV1::new(
        2,
        NativePhysicalExtentV1::new(240, 180),
        NativeScaleFactorV1::try_from_f64(2.0).expect("fixed scale should validate"),
        HeadlessSurface::new(120, 90),
    )
}

fn assert_copy<T: Copy>() {}
