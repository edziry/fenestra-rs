pub(super) mod scheduler;

use fenestra_ui_ir::prototype::InputPolicy;
use fenestra_ui_runtime::prototype::{
    HeadlessRect, HeadlessSemanticAction, HeadlessSemanticRole, HeadlessSurface,
};

use crate::headless::oracle::{NormalizedHeadlessProjectionV1, NormalizedHeadlessSceneRectangleV1};
use crate::headless::platform::HeadlessPointerTargetV1;
use crate::headless::trace::{
    HeadlessInputKindV1, HeadlessOutcomeV1, HeadlessTraceEventV1, HeadlessTraceProjectionCountsV1,
    HeadlessTraceStageV1,
};
use crate::semantic::NodePathV1;

pub(super) use scheduler::SchedulerEventRecordV1;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct QueueRecordV1 {
    pub(super) items: usize,
    pub(super) bytes: usize,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct TraceEventRecordV1 {
    pub(super) schema: u32,
    pub(super) sequence: u64,
    pub(super) domain: u32,
    pub(super) tick: u64,
    pub(super) stage: HeadlessTraceStageV1,
    pub(super) input: HeadlessInputKindV1,
    pub(super) outcome: HeadlessOutcomeV1,
    pub(super) captured: Option<u64>,
    pub(super) published: Option<u64>,
    pub(super) target: HeadlessPointerTargetV1,
    pub(super) frame: Option<u64>,
    pub(super) control: Option<u64>,
    pub(super) surface: HeadlessSurface,
    pub(super) counts: HeadlessTraceProjectionCountsV1,
    pub(super) deferred: QueueRecordV1,
    pub(super) controls: QueueRecordV1,
    pub(super) visual: QueueRecordV1,
    pub(super) in_flight: QueueRecordV1,
    pub(super) renderer: QueueRecordV1,
}

impl TraceEventRecordV1 {
    pub(super) fn from_event(event: HeadlessTraceEventV1) -> Self {
        Self {
            schema: event.schema_revision(),
            sequence: event.sequence(),
            domain: event.clock_domain().get(),
            tick: event.tick().get(),
            stage: event.stage(),
            input: event.input(),
            outcome: event.outcome(),
            captured: event.captured_generation().map(|value| value.get()),
            published: event.published_generation().map(|value| value.get()),
            target: event.target(),
            frame: event.frame().map(|value| value.get()),
            control: event.control().map(|value| value.get()),
            surface: event.surface(),
            counts: event.projection_counts(),
            deferred: queue(event.deferred().items(), event.deferred().accounted_bytes()),
            controls: queue(event.controls().items(), event.controls().accounted_bytes()),
            visual: queue(event.visual().items(), event.visual().accounted_bytes()),
            in_flight: queue(
                event.in_flight().items(),
                event.in_flight().accounted_bytes(),
            ),
            renderer: queue(event.renderer().items(), event.renderer().accounted_bytes()),
        }
    }
}

const fn queue(items: usize, bytes: usize) -> QueueRecordV1 {
    QueueRecordV1 { items, bytes }
}

#[derive(Clone, Eq, PartialEq)]
pub(super) struct ComputedRecordV1 {
    pub(super) path: NodePathV1,
    pub(super) width: i32,
    pub(super) height: i32,
    pub(super) color: [u8; 4],
    pub(super) visible: bool,
    pub(super) input: InputPolicy,
}

#[derive(Clone, Eq, PartialEq)]
pub(super) struct GeometryRecordV1 {
    pub(super) path: NodePathV1,
    pub(super) bounds: HeadlessRect,
    pub(super) clip: HeadlessRect,
}

#[derive(Clone, Eq, PartialEq)]
pub(super) struct SemanticRecordV1 {
    pub(super) path: NodePathV1,
    pub(super) role: HeadlessSemanticRole,
    pub(super) label: u32,
    pub(super) action: HeadlessSemanticAction,
}

#[derive(Clone, Eq, PartialEq)]
pub(super) struct RectangleRecordV1 {
    pub(super) path: NodePathV1,
    pub(super) rectangle: HeadlessRect,
}

#[derive(Clone, Eq, PartialEq)]
pub(super) struct SceneRecordV1 {
    pub(super) path: NodePathV1,
    pub(super) rectangle: HeadlessRect,
    pub(super) color: [u8; 4],
}

#[derive(Clone, Eq, PartialEq)]
pub(super) struct ProjectionRecordV1 {
    pub(super) surface: HeadlessSurface,
    pub(super) counts: HeadlessTraceProjectionCountsV1,
    pub(super) computed: Vec<ComputedRecordV1>,
    pub(super) geometry: Vec<GeometryRecordV1>,
    pub(super) semantics: Vec<SemanticRecordV1>,
    pub(super) hits: Vec<RectangleRecordV1>,
    pub(super) scene: Vec<SceneRecordV1>,
}

impl ProjectionRecordV1 {
    pub(super) fn from_projection(projection: &NormalizedHeadlessProjectionV1) -> Self {
        let computed = projection
            .computed_styles()
            .iter()
            .map(|record| ComputedRecordV1 {
                path: record.path().clone(),
                width: record.width(),
                height: record.height(),
                color: record.color(),
                visible: record.visible(),
                input: record.input(),
            })
            .collect::<Vec<_>>();
        let geometry = projection
            .geometries()
            .iter()
            .map(|record| GeometryRecordV1 {
                path: record.path().clone(),
                bounds: record.bounds(),
                clip: record.clip(),
            })
            .collect::<Vec<_>>();
        let semantics = projection
            .semantics()
            .iter()
            .map(|record| SemanticRecordV1 {
                path: record.path().clone(),
                role: record.role(),
                label: record.label(),
                action: record.action(),
            })
            .collect::<Vec<_>>();
        let hits = projection
            .hit_regions()
            .iter()
            .map(|record| RectangleRecordV1 {
                path: record.path().clone(),
                rectangle: record.clip(),
            })
            .collect::<Vec<_>>();
        let scene = projection
            .scene_rectangles()
            .iter()
            .map(scene_record)
            .collect::<Vec<_>>();
        Self {
            surface: projection.surface(),
            counts: HeadlessTraceProjectionCountsV1::new(
                computed.len(),
                geometry.len(),
                semantics.len(),
                hits.len(),
                scene.len(),
            ),
            computed,
            geometry,
            semantics,
            hits,
            scene,
        }
    }
}

fn scene_record(record: &NormalizedHeadlessSceneRectangleV1) -> SceneRecordV1 {
    SceneRecordV1 {
        path: record.path().clone(),
        rectangle: record.rectangle(),
        color: record.color(),
    }
}
