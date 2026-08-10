#[path = "runtime/build.rs"]
mod build;
#[path = "runtime/fault.rs"]
mod fault;
#[path = "runtime/lines.rs"]
mod lines;

use fenestra_ui_ir::prototype::InvalidationSet;
use fenestra_ui_runtime::prototype::{HeadlessRect, HeadlessSurface};
use fenestra_ui_testkit::prototype::NodePathV1;

pub(super) use self::build::build_runtime_artifact_v1;
pub(super) use self::fault::inject_runtime_artifact_fault_v1;
pub(super) use self::lines::runtime_lines_v1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RuntimeStepV1 {
    Initial,
    Color,
    Insert,
    Move,
    Update,
    Remove,
    Resize,
}

impl RuntimeStepV1 {
    pub(super) const ALL: [Self; 7] = [
        Self::Initial,
        Self::Color,
        Self::Insert,
        Self::Move,
        Self::Update,
        Self::Remove,
        Self::Resize,
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RuntimeExecutionLaneV1 {
    Reference,
    Candidate,
}

impl RuntimeExecutionLaneV1 {
    pub(super) const ALL: [Self; 2] = [Self::Reference, Self::Candidate];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RuntimeProjectionLaneV1 {
    Oracle,
    Reference,
    Candidate,
}

impl RuntimeProjectionLaneV1 {
    pub(super) const ALL: [Self; 3] = [Self::Oracle, Self::Reference, Self::Candidate];
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RuntimeArtifactV1 {
    pub(super) milestones: Vec<RuntimeMilestoneV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RuntimeMilestoneV1 {
    pub(super) step: RuntimeStepV1,
    pub(super) reference_receipt: RuntimeReceiptV1,
    pub(super) candidate_receipt: RuntimeReceiptV1,
    pub(super) oracle_projection: RuntimeProjectionV1,
    pub(super) reference_projection: RuntimeProjectionV1,
    pub(super) candidate_projection: RuntimeProjectionV1,
}

impl RuntimeMilestoneV1 {
    pub(super) const fn receipt(&self, lane: RuntimeExecutionLaneV1) -> &RuntimeReceiptV1 {
        match lane {
            RuntimeExecutionLaneV1::Reference => &self.reference_receipt,
            RuntimeExecutionLaneV1::Candidate => &self.candidate_receipt,
        }
    }

    pub(super) const fn projection(&self, lane: RuntimeProjectionLaneV1) -> &RuntimeProjectionV1 {
        match lane {
            RuntimeProjectionLaneV1::Oracle => &self.oracle_projection,
            RuntimeProjectionLaneV1::Reference => &self.reference_projection,
            RuntimeProjectionLaneV1::Candidate => &self.candidate_projection,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RuntimeReceiptV1 {
    pub(super) receipt_generation: Option<u64>,
    pub(super) projection_generation: u64,
    pub(super) invalidation: InvalidationSet,
    pub(super) mutation_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RuntimeProjectionV1 {
    pub(super) surface: HeadlessSurface,
    pub(super) counts: RuntimeProjectionCountsV1,
    pub(super) geometries: Vec<RuntimeGeometryV1>,
    pub(super) hits: Vec<RuntimeHitV1>,
    pub(super) scenes: Vec<RuntimeSceneV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RuntimeProjectionCountsV1 {
    pub(super) computed_styles: usize,
    pub(super) geometry: usize,
    pub(super) semantics: usize,
    pub(super) hit_regions: usize,
    pub(super) scene_rectangles: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RuntimeGeometryV1 {
    pub(super) path: NodePathV1,
    pub(super) bounds: HeadlessRect,
    pub(super) clip: HeadlessRect,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RuntimeHitV1 {
    pub(super) path: NodePathV1,
    pub(super) clip: HeadlessRect,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RuntimeSceneV1 {
    pub(super) path: NodePathV1,
    pub(super) rectangle: HeadlessRect,
    pub(super) color: [u8; 4],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RuntimeReceiptFieldV1 {
    ReceiptGeneration,
    ProjectionGeneration,
    Invalidation,
    MutationCount,
}

impl RuntimeReceiptFieldV1 {
    pub(super) const ALL: [Self; 4] = [
        Self::ReceiptGeneration,
        Self::ProjectionGeneration,
        Self::Invalidation,
        Self::MutationCount,
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RuntimeProjectionFieldV1 {
    Surface,
    ComputedStyleCount,
    GeometryCount,
    SemanticsCount,
    HitRegionCount,
    SceneRectangleCount,
}

impl RuntimeProjectionFieldV1 {
    pub(super) const ALL: [Self; 6] = [
        Self::Surface,
        Self::ComputedStyleCount,
        Self::GeometryCount,
        Self::SemanticsCount,
        Self::HitRegionCount,
        Self::SceneRectangleCount,
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RuntimeGeometryFieldV1 {
    Path,
    Bounds,
    Clip,
}

impl RuntimeGeometryFieldV1 {
    pub(super) const ALL: [Self; 3] = [Self::Path, Self::Bounds, Self::Clip];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RuntimeHitFieldV1 {
    Path,
    Clip,
}

impl RuntimeHitFieldV1 {
    pub(super) const ALL: [Self; 2] = [Self::Path, Self::Clip];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RuntimeSceneFieldV1 {
    Path,
    Rectangle,
    Color,
}

impl RuntimeSceneFieldV1 {
    pub(super) const ALL: [Self; 3] = [Self::Path, Self::Rectangle, Self::Color];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RuntimeArtifactFaultV1 {
    Receipt {
        milestone: usize,
        lane: RuntimeExecutionLaneV1,
        field: RuntimeReceiptFieldV1,
    },
    Projection {
        milestone: usize,
        lane: RuntimeProjectionLaneV1,
        field: RuntimeProjectionFieldV1,
    },
    Geometry {
        milestone: usize,
        lane: RuntimeProjectionLaneV1,
        record: usize,
        field: RuntimeGeometryFieldV1,
    },
    Hit {
        milestone: usize,
        lane: RuntimeProjectionLaneV1,
        record: usize,
        field: RuntimeHitFieldV1,
    },
    Scene {
        milestone: usize,
        lane: RuntimeProjectionLaneV1,
        record: usize,
        field: RuntimeSceneFieldV1,
    },
}

pub(super) type RuntimeArtifactSliceV1 = RuntimeArtifactFaultV1;
