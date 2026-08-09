use fenestra_ui_runtime::prototype::{HeadlessSurface, SchedulerState};

use super::*;
use crate::headless::artifact::model::build_headless_artifact_v1;
use crate::headless::artifact::record::scheduler::{ActionRecordV1, StepRecordV1};
use crate::headless::runner::run_headless_spine_v1;
use crate::headless::trace::{
    HeadlessInputKindV1, HeadlessTraceProjectionCountsV1, HeadlessTraceStageV1,
};

fn canonical() -> HeadlessArtifactV1 {
    let run = run_headless_spine_v1().expect("fixed headless run should succeed");
    build_headless_artifact_v1(&run)
}

#[test]
fn headless_fields_follow_flat_wire_order() {
    let artifact = canonical();
    let left = artifact.headless_events[0];
    let mut right = left;
    right.renderer.bytes = right.renderer.bytes.saturating_add(1);
    right.surface = HeadlessSurface::new(
        right.surface.width().saturating_add(1),
        right.surface.height().saturating_add(1),
    );
    right.input = different_input(right.input);
    right.stage = different_stage(right.stage);

    assert_eq!(
        headless_event_field(&left, &right),
        Some(HeadlessTraceFieldV1::Stage)
    );

    right.stage = left.stage;
    assert_eq!(
        headless_event_field(&left, &right),
        Some(HeadlessTraceFieldV1::Input)
    );

    right.input = left.input;
    assert_eq!(
        headless_event_field(&left, &right),
        Some(HeadlessTraceFieldV1::SurfaceWidth)
    );
}

#[test]
fn headless_aggregate_fields_preserve_wire_priority() {
    let artifact = canonical();
    let left = artifact.headless_events[0];
    let mut right = left;
    let counts = left.counts;
    right.counts = HeadlessTraceProjectionCountsV1::new(
        counts.computed_styles().saturating_add(1),
        counts.geometries().saturating_add(1),
        counts.semantics().saturating_add(1),
        counts.hit_regions().saturating_add(1),
        counts.scene_rectangles().saturating_add(1),
    );
    right.deferred.items = right.deferred.items.saturating_add(1);

    assert_eq!(
        headless_event_field(&left, &right),
        Some(HeadlessTraceFieldV1::ComputedStyles)
    );

    right.counts = left.counts;
    right.deferred.bytes = right.deferred.bytes.saturating_add(1);
    assert_eq!(
        headless_event_field(&left, &right),
        Some(HeadlessTraceFieldV1::DeferredItems)
    );
}

#[test]
fn scheduler_fields_follow_flat_wire_order() {
    let artifact = canonical();
    let left = artifact.scheduler_events[0];
    let mut right = left;
    right.renderer.pending = !right.renderer.pending;
    right.generation = right.generation.saturating_add(1);
    right.lifecycle = different_lifecycle(right.lifecycle);

    assert_eq!(
        scheduler_event_field(&left, &right),
        Some(SchedulerTraceFieldV1::Lifecycle)
    );

    right.step = different_step(right.step);
    assert_eq!(
        scheduler_event_field(&left, &right),
        Some(SchedulerTraceFieldV1::Step)
    );
}

#[test]
fn scheduler_lane_fields_precede_renderer_fields() {
    let artifact = canonical();
    let left = artifact.scheduler_events[0];
    let mut right = left;
    right.renderer.items = right.renderer.items.saturating_add(1);
    right.deferred.residence = different_option(right.deferred.residence);
    right.deferred.bytes = right.deferred.bytes.saturating_add(1);

    assert_eq!(
        scheduler_event_field(&left, &right),
        Some(SchedulerTraceFieldV1::DeferredBytes)
    );
}

#[test]
fn trace_cardinality_uses_the_shorter_length() {
    let artifact = canonical();
    let headless_count = artifact.headless_events.len();
    let scheduler_count = artifact.scheduler_events.len();
    assert!(headless_count > 1);
    assert!(scheduler_count > 1);

    let headless = first_headless_difference_v1(
        &artifact.headless_events[..headless_count - 1],
        &artifact.headless_events,
    )
    .expect("headless cardinality should differ");
    assert_eq!(headless.index, headless_count - 1);
    assert_eq!(headless.field, HeadlessTraceFieldV1::Cardinality);

    let scheduler = first_scheduler_difference_v1(
        &artifact.scheduler_events,
        &artifact.scheduler_events[..scheduler_count - 1],
    )
    .expect("scheduler cardinality should differ");
    assert_eq!(scheduler.index, scheduler_count - 1);
    assert_eq!(scheduler.field, SchedulerTraceFieldV1::Cardinality);
}

fn different_stage(value: HeadlessTraceStageV1) -> HeadlessTraceStageV1 {
    if value == HeadlessTraceStageV1::Build {
        HeadlessTraceStageV1::Input
    } else {
        HeadlessTraceStageV1::Build
    }
}

fn different_input(value: HeadlessInputKindV1) -> HeadlessInputKindV1 {
    if value == HeadlessInputKindV1::None {
        HeadlessInputKindV1::Pointer
    } else {
        HeadlessInputKindV1::None
    }
}

fn different_lifecycle(value: SchedulerState) -> SchedulerState {
    if value == SchedulerState::Running {
        SchedulerState::Faulted
    } else {
        SchedulerState::Running
    }
}

fn different_step(value: StepRecordV1) -> StepRecordV1 {
    if value == StepRecordV1::Action(ActionRecordV1::Idle) {
        StepRecordV1::Action(ActionRecordV1::RequestFrame)
    } else {
        StepRecordV1::Action(ActionRecordV1::Idle)
    }
}

fn different_option(value: Option<u64>) -> Option<u64> {
    match value {
        Some(value) => Some(value.saturating_add(1)),
        None => Some(0),
    }
}
