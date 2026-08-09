use super::super::error::{
    HeadlessArtifactVerificationErrorKindV1 as Kind, HeadlessArtifactVerificationErrorV1 as Error,
};
use super::super::model::HeadlessArtifactV1;
use super::super::record::{SchedulerEventRecordV1, TraceEventRecordV1};

pub(super) fn verify_traces_v1(
    stored: &HeadlessArtifactV1,
    expected: &HeadlessArtifactV1,
) -> Result<(), Error> {
    if let Some(difference) =
        first_headless_difference_v1(&stored.headless_events, &expected.headless_events)
    {
        return Err(Error::at(Kind::HeadlessTraceMismatch, difference.index));
    }
    if let Some(difference) =
        first_scheduler_difference_v1(&stored.scheduler_events, &expected.scheduler_events)
    {
        return Err(Error::at(Kind::SchedulerTraceMismatch, difference.index));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HeadlessTraceFieldV1 {
    Schema,
    Sequence,
    Domain,
    Tick,
    Stage,
    Input,
    Outcome,
    Captured,
    Published,
    Target,
    Frame,
    Control,
    SurfaceWidth,
    SurfaceHeight,
    ComputedStyles,
    Geometries,
    Semantics,
    HitRegions,
    SceneRectangles,
    DeferredItems,
    DeferredBytes,
    ControlItems,
    ControlBytes,
    VisualItems,
    VisualBytes,
    InFlightItems,
    InFlightBytes,
    RendererItems,
    RendererBytes,
    Cardinality,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SchedulerTraceFieldV1 {
    Schema,
    Sequence,
    Domain,
    Tick,
    Step,
    Lifecycle,
    Generation,
    Frame,
    Control,
    DeferredItems,
    DeferredBytes,
    DeferredResidence,
    ControlItems,
    ControlBytes,
    ControlResidence,
    VisualItems,
    VisualBytes,
    VisualResidence,
    InFlightItems,
    InFlightBytes,
    InFlightResidence,
    RendererItems,
    RendererBytes,
    RendererResidence,
    RendererLast,
    RendererCompleted,
    RendererPending,
    Cardinality,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TraceDifferenceV1<Field> {
    index: usize,
    field: Field,
}

fn first_headless_difference_v1(
    stored: &[TraceEventRecordV1],
    expected: &[TraceEventRecordV1],
) -> Option<TraceDifferenceV1<HeadlessTraceFieldV1>> {
    first_difference(
        stored,
        expected,
        headless_event_field,
        HeadlessTraceFieldV1::Cardinality,
    )
}

fn first_scheduler_difference_v1(
    stored: &[SchedulerEventRecordV1],
    expected: &[SchedulerEventRecordV1],
) -> Option<TraceDifferenceV1<SchedulerTraceFieldV1>> {
    first_difference(
        stored,
        expected,
        scheduler_event_field,
        SchedulerTraceFieldV1::Cardinality,
    )
}

fn first_difference<T, Field: Copy>(
    stored: &[T],
    expected: &[T],
    compare: fn(&T, &T) -> Option<Field>,
    cardinality: Field,
) -> Option<TraceDifferenceV1<Field>> {
    for (index, (left, right)) in stored.iter().zip(expected).enumerate() {
        if let Some(field) = compare(left, right) {
            return Some(TraceDifferenceV1 { index, field });
        }
    }
    (stored.len() != expected.len()).then_some(TraceDifferenceV1 {
        index: stored.len().min(expected.len()),
        field: cardinality,
    })
}

fn headless_event_field(
    left: &TraceEventRecordV1,
    right: &TraceEventRecordV1,
) -> Option<HeadlessTraceFieldV1> {
    use HeadlessTraceFieldV1 as Field;

    let left_counts = left.counts;
    let right_counts = right.counts;
    if left.schema != right.schema {
        Some(Field::Schema)
    } else if left.sequence != right.sequence {
        Some(Field::Sequence)
    } else if left.domain != right.domain {
        Some(Field::Domain)
    } else if left.tick != right.tick {
        Some(Field::Tick)
    } else if left.stage != right.stage {
        Some(Field::Stage)
    } else if left.input != right.input {
        Some(Field::Input)
    } else if left.outcome != right.outcome {
        Some(Field::Outcome)
    } else if left.captured != right.captured {
        Some(Field::Captured)
    } else if left.published != right.published {
        Some(Field::Published)
    } else if left.target != right.target {
        Some(Field::Target)
    } else if left.frame != right.frame {
        Some(Field::Frame)
    } else if left.control != right.control {
        Some(Field::Control)
    } else if left.surface.width() != right.surface.width() {
        Some(Field::SurfaceWidth)
    } else if left.surface.height() != right.surface.height() {
        Some(Field::SurfaceHeight)
    } else if left_counts.computed_styles() != right_counts.computed_styles() {
        Some(Field::ComputedStyles)
    } else if left_counts.geometries() != right_counts.geometries() {
        Some(Field::Geometries)
    } else if left_counts.semantics() != right_counts.semantics() {
        Some(Field::Semantics)
    } else if left_counts.hit_regions() != right_counts.hit_regions() {
        Some(Field::HitRegions)
    } else if left_counts.scene_rectangles() != right_counts.scene_rectangles() {
        Some(Field::SceneRectangles)
    } else if left.deferred.items != right.deferred.items {
        Some(Field::DeferredItems)
    } else if left.deferred.bytes != right.deferred.bytes {
        Some(Field::DeferredBytes)
    } else if left.controls.items != right.controls.items {
        Some(Field::ControlItems)
    } else if left.controls.bytes != right.controls.bytes {
        Some(Field::ControlBytes)
    } else if left.visual.items != right.visual.items {
        Some(Field::VisualItems)
    } else if left.visual.bytes != right.visual.bytes {
        Some(Field::VisualBytes)
    } else if left.in_flight.items != right.in_flight.items {
        Some(Field::InFlightItems)
    } else if left.in_flight.bytes != right.in_flight.bytes {
        Some(Field::InFlightBytes)
    } else if left.renderer.items != right.renderer.items {
        Some(Field::RendererItems)
    } else if left.renderer.bytes != right.renderer.bytes {
        Some(Field::RendererBytes)
    } else {
        None
    }
}

fn scheduler_event_field(
    left: &SchedulerEventRecordV1,
    right: &SchedulerEventRecordV1,
) -> Option<SchedulerTraceFieldV1> {
    use SchedulerTraceFieldV1 as Field;

    if left.schema != right.schema {
        Some(Field::Schema)
    } else if left.sequence != right.sequence {
        Some(Field::Sequence)
    } else if left.domain != right.domain {
        Some(Field::Domain)
    } else if left.tick != right.tick {
        Some(Field::Tick)
    } else if left.step != right.step {
        Some(Field::Step)
    } else if left.lifecycle != right.lifecycle {
        Some(Field::Lifecycle)
    } else if left.generation != right.generation {
        Some(Field::Generation)
    } else if left.frame != right.frame {
        Some(Field::Frame)
    } else if left.control != right.control {
        Some(Field::Control)
    } else if left.deferred.items != right.deferred.items {
        Some(Field::DeferredItems)
    } else if left.deferred.bytes != right.deferred.bytes {
        Some(Field::DeferredBytes)
    } else if left.deferred.residence != right.deferred.residence {
        Some(Field::DeferredResidence)
    } else if left.controls.items != right.controls.items {
        Some(Field::ControlItems)
    } else if left.controls.bytes != right.controls.bytes {
        Some(Field::ControlBytes)
    } else if left.controls.residence != right.controls.residence {
        Some(Field::ControlResidence)
    } else if left.visual.items != right.visual.items {
        Some(Field::VisualItems)
    } else if left.visual.bytes != right.visual.bytes {
        Some(Field::VisualBytes)
    } else if left.visual.residence != right.visual.residence {
        Some(Field::VisualResidence)
    } else if left.in_flight.items != right.in_flight.items {
        Some(Field::InFlightItems)
    } else if left.in_flight.bytes != right.in_flight.bytes {
        Some(Field::InFlightBytes)
    } else if left.in_flight.residence != right.in_flight.residence {
        Some(Field::InFlightResidence)
    } else if left.renderer.items != right.renderer.items {
        Some(Field::RendererItems)
    } else if left.renderer.bytes != right.renderer.bytes {
        Some(Field::RendererBytes)
    } else if left.renderer.residence != right.renderer.residence {
        Some(Field::RendererResidence)
    } else if left.renderer.last != right.renderer.last {
        Some(Field::RendererLast)
    } else if left.renderer.completed != right.renderer.completed {
        Some(Field::RendererCompleted)
    } else if left.renderer.pending != right.renderer.pending {
        Some(Field::RendererPending)
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
