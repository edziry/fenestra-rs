use super::super::error::{HeadlessArtifactDecodeErrorKindV1, HeadlessArtifactDecodeErrorV1};
use super::scan::ScannedLineV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CapacityRowV1 {
    Ir,
    Style,
    Runtime,
    Projection,
    Scheduler,
    Renderer,
    SchedulerTrace,
    HeadlessTrace,
    Artifact,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RecordKindV1 {
    Header,
    Versions,
    Fixture,
    Environment,
    ProjectionChoices,
    Capacity(CapacityRowV1),
    HeadlessBegin,
    HeadlessEvent,
    HeadlessEnd,
    SchedulerBegin,
    SchedulerEvent,
    SchedulerEnd,
    ProjectionBegin,
    ComputedBegin,
    Computed,
    ComputedEnd,
    GeometryBegin,
    Geometry,
    GeometryEnd,
    SemanticBegin,
    Semantic,
    SemanticEnd,
    HitBegin,
    Hit,
    HitEnd,
    SceneBegin,
    Scene,
    SceneEnd,
    ProjectionEnd,
    Result,
    End,
}

pub(super) fn validate_closed_grammar_v1(
    lines: &[ScannedLineV1<'_>],
) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    for line in lines {
        let kind = classify_line_v1(line)?;
        if kind == RecordKindV1::End {
            break;
        }
    }
    Ok(())
}

pub(super) fn classify_line_v1(
    line: &ScannedLineV1<'_>,
) -> Result<RecordKindV1, HeadlessArtifactDecodeErrorV1> {
    let fields = split_fields_v1(line)?;
    let fields = fields.as_slice();
    let Some(tag) = fields.first().copied() else {
        return Err(malformed(line.number));
    };
    let kind = match tag {
        "fenestra-headless-spine" if fields.len() == 2 => RecordKindV1::Header,
        "versions" if versions_shape(fields) => RecordKindV1::Versions,
        "fixture" if fields.len() == 8 && fields[1] == "headless-spine" => RecordKindV1::Fixture,
        "environment" if environment_shape(fields) => RecordKindV1::Environment,
        "projection-choices" if projection_choices_shape(fields) => RecordKindV1::ProjectionChoices,
        "capacity-ir" if fields.len() == 10 => RecordKindV1::Capacity(CapacityRowV1::Ir),
        "capacity-style" if fields.len() == 2 => RecordKindV1::Capacity(CapacityRowV1::Style),
        "capacity-runtime" if fields.len() == 7 => RecordKindV1::Capacity(CapacityRowV1::Runtime),
        "capacity-projection" if fields.len() == 6 => {
            RecordKindV1::Capacity(CapacityRowV1::Projection)
        }
        "capacity-scheduler" if fields.len() == 13 => {
            RecordKindV1::Capacity(CapacityRowV1::Scheduler)
        }
        "capacity-renderer" if fields.len() == 4 => RecordKindV1::Capacity(CapacityRowV1::Renderer),
        "capacity-scheduler-trace" if fields.len() == 4 && fields[3] == "96" => {
            RecordKindV1::Capacity(CapacityRowV1::SchedulerTrace)
        }
        "capacity-headless-trace" if fields.len() == 4 && fields[3] == "160" => {
            RecordKindV1::Capacity(CapacityRowV1::HeadlessTrace)
        }
        "capacity-artifact" if fields.len() == 4 => RecordKindV1::Capacity(CapacityRowV1::Artifact),
        "headless-trace-begin" if fields.len() == 3 => RecordKindV1::HeadlessBegin,
        "h-event" if headless_event_shape(fields) => RecordKindV1::HeadlessEvent,
        "headless-trace-end" if fields.len() == 1 => RecordKindV1::HeadlessEnd,
        "scheduler-trace-begin" if fields.len() == 3 => RecordKindV1::SchedulerBegin,
        "s-event" if scheduler_event_shape(fields) => RecordKindV1::SchedulerEvent,
        "scheduler-trace-end" if fields.len() == 1 => RecordKindV1::SchedulerEnd,
        "projection-begin" if fields.len() == 9 => RecordKindV1::ProjectionBegin,
        "computed-begin" if fields.len() == 1 => RecordKindV1::ComputedBegin,
        "computed" if computed_shape(fields) => RecordKindV1::Computed,
        "computed-end" if fields.len() == 1 => RecordKindV1::ComputedEnd,
        "geometry-begin" if fields.len() == 1 => RecordKindV1::GeometryBegin,
        "geometry" if fields.len() == 10 => RecordKindV1::Geometry,
        "geometry-end" if fields.len() == 1 => RecordKindV1::GeometryEnd,
        "semantic-begin" if fields.len() == 1 => RecordKindV1::SemanticBegin,
        "semantic" if semantic_shape(fields) => RecordKindV1::Semantic,
        "semantic-end" if fields.len() == 1 => RecordKindV1::SemanticEnd,
        "hit-begin" if fields.len() == 1 => RecordKindV1::HitBegin,
        "hit" if fields.len() == 6 => RecordKindV1::Hit,
        "hit-end" if fields.len() == 1 => RecordKindV1::HitEnd,
        "scene-begin" if fields.len() == 1 => RecordKindV1::SceneBegin,
        "scene" if scene_shape(fields) => RecordKindV1::Scene,
        "scene-end" if fields.len() == 1 => RecordKindV1::SceneEnd,
        "projection-end" if fields.len() == 1 => RecordKindV1::ProjectionEnd,
        "result" if result_shape(fields) => RecordKindV1::Result,
        "end" if fields.len() == 1 => RecordKindV1::End,
        _ => return Err(malformed(line.number)),
    };
    Ok(kind)
}

const MAX_FIELDS: usize = 32;

pub(super) struct FieldsV1<'source> {
    values: [&'source str; MAX_FIELDS],
    len: usize,
}

impl<'source> FieldsV1<'source> {
    pub(super) fn as_slice(&self) -> &[&'source str] {
        &self.values[..self.len]
    }
}

pub(super) fn split_fields_v1<'source>(
    line: &ScannedLineV1<'source>,
) -> Result<FieldsV1<'source>, HeadlessArtifactDecodeErrorV1> {
    let mut fields = FieldsV1 {
        values: [""; MAX_FIELDS],
        len: 0,
    };
    for field in line.text.split('|') {
        let Some(slot) = fields.values.get_mut(fields.len) else {
            return Err(malformed(line.number));
        };
        *slot = field;
        fields.len += 1;
    }
    Ok(fields)
}

fn versions_shape(fields: &[&str]) -> bool {
    fields.len() == 13
        && fields[1] == "fixture"
        && fields[3] == "schema"
        && fields[5] == "construction"
        && fields[7] == "style"
        && fields[9] == "trace"
        && fields[11] == "projection"
}

fn environment_shape(fields: &[&str]) -> bool {
    fields.len() == 7
        && fields[1] == "platform"
        && fields[2] == "headless-fake"
        && fields[3] == "clock"
        && fields[4] == "scheduler"
        && fields[5] == "domain"
        && fields[6] == "8001"
}

fn projection_choices_shape(fields: &[&str]) -> bool {
    fields
        == [
            "projection-choices",
            "full",
            "vertical",
            "rebuilt",
            "reverse",
        ]
}

fn headless_event_shape(fields: &[&str]) -> bool {
    fields.len() == 30
        && matches!(
            fields[5],
            "build"
                | "input"
                | "callback"
                | "transaction"
                | "projection"
                | "scheduler"
                | "renderer"
        )
        && matches!(
            fields[6],
            "none"
                | "pointer"
                | "direct"
                | "insert"
                | "move"
                | "update"
                | "remove"
                | "resize"
                | "frame-ready"
                | "completion"
                | "loss"
                | "shutdown"
        )
        && matches!(
            fields[7],
            "observed"
                | "deferred"
                | "published"
                | "no-change"
                | "matched"
                | "action"
                | "accepted"
                | "rejected"
                | "completed"
                | "lost"
                | "stopped"
                | "failed:runtime"
                | "failed:projection"
                | "failed:oracle"
                | "failed:scheduler"
                | "failed:renderer"
                | "failed:trace"
        )
        && (matches!(fields[10], "none" | "static-control") || fields[10].starts_with("key:"))
}

fn scheduler_event_shape(fields: &[&str]) -> bool {
    fields.len() == 32
        && super::scheduler::step_shape_v1(&fields[5..10])
        && matches!(
            fields[10],
            "running" | "shutdown-queued" | "draining" | "stopped" | "faulted"
        )
        && matches!(fields[31], "false" | "true")
}

fn computed_shape(fields: &[&str]) -> bool {
    fields.len() == 7
        && fields[4].starts_with("rgba8:")
        && matches!(fields[5], "false" | "true")
        && matches!(fields[6], "accept" | "ignore")
}

fn semantic_shape(fields: &[&str]) -> bool {
    fields.len() == 5 && fields[2] == "control" && fields[4] == "activate"
}

fn scene_shape(fields: &[&str]) -> bool {
    fields.len() == 7 && fields[6].starts_with("rgba8:")
}

fn result_shape(fields: &[&str]) -> bool {
    fields.len() == 2 && matches!(fields[1], "pass" | "adapt" | "stop")
}

fn malformed(line: u32) -> HeadlessArtifactDecodeErrorV1 {
    HeadlessArtifactDecodeErrorV1::at(HeadlessArtifactDecodeErrorKindV1::MalformedRecord, line)
}
