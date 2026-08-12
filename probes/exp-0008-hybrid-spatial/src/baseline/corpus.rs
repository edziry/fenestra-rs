use super::literal_types::LiteralObservationInputV2;

mod direct;
mod resources;
mod runtime;
mod specs;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CaseKindV2 {
    Direct,
    RuntimeMutation,
    RuntimeRollback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlacementModeV2 {
    Root,
    Layout,
    Free,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CorpusOperationV2 {
    Direct,
    Resize,
    ResizeHost,
    DependencyCycleControl,
    Init,
    SetSpanX,
    SetTone,
    SetPolicy,
    Insert,
    Move,
    Update,
    Remove,
    SingularAttempt,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CorpusObligationV2 {
    AllLayout,
    AllFree,
    NestedDescendant,
    Resize,
    OuterFreeInnerLayout,
    OuterLayoutInnerFree,
    MixedSiblingModes,
    FreeConsumesNoLayout,
    TransparentParticipatesLayout,
    TransparentNoPaintHit,
    SplitLayoutExtent,
    PaintOverflow,
    CircularHit,
    IndependentSemantics,
    ThreeLevelTransforms,
    TwoLinkClip,
    PolygonConcavity,
    AabbMiss,
    BothFillRules,
    TwoSubpaths,
    SelfIntersection,
    ZeroSegment,
    Quadratic,
    Cubic,
    Fill,
    RoundStroke,
    Solid,
    PartialAlpha,
    LinearGradient,
    PremultipliedImage,
    SourceOver,
    ExplicitClip,
    LaterAnchor,
    ParentAnchor,
    ViewportAnchor,
    DependencyCycleControl,
    ZeroByN,
    NByZero,
    ZeroByZero,
    RuntimeNineSteps,
    KeyedMutation,
    SingularRollback,
    ExactRetainedState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum QuerySetV2 {
    DirectComplete,
    RuntimePixelCenters,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct QueryInventoryV2 {
    pub(crate) shape_vertices: usize,
    pub(crate) segment_midpoints: usize,
    pub(crate) clip_corners: usize,
    pub(crate) aabb_centers: usize,
    pub(crate) nonrectangular_misses: usize,
    pub(crate) authored_boundaries: usize,
    pub(crate) viewport_outside: usize,
    pub(crate) logical_pixel_centers: usize,
    pub(crate) duplicates_retained: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CorpusCaseV2 {
    pub(crate) ordinal: u8,
    pub(crate) name: &'static str,
    pub(crate) kind: CaseKindV2,
    pub(crate) node_keys: Vec<u32>,
    pub(crate) placements: Vec<PlacementModeV2>,
    pub(crate) operations: Vec<CorpusOperationV2>,
    pub(crate) obligations: Vec<CorpusObligationV2>,
    pub(crate) typed_scalars: Vec<i64>,
    pub(crate) query_set: QuerySetV2,
    pub(crate) observation_count: usize,
    pub(crate) authored_order_digest: u64,
    pub(crate) table_digest: u64,
    pub(crate) nested_depth: usize,
    pub(crate) query_inventory: QueryInventoryV2,
    pub(crate) initial_viewport: (u32, u32),
    pub(crate) final_viewport: (u32, u32),
    pub(crate) observations: Vec<LiteralObservationInputV2>,
}

impl CorpusCaseV2 {
    pub(crate) fn has(&self, obligation: CorpusObligationV2) -> bool {
        self.obligations.contains(&obligation)
    }
}

const S: i64 = 65_536;

pub(crate) fn registered_corpus_v2() -> Vec<CorpusCaseV2> {
    let specs = specs::registered_specs_v2();
    specs
        .into_iter()
        .enumerate()
        .map(|(ordinal, spec)| build_case(ordinal, spec))
        .collect()
}

pub(super) struct CaseSpec {
    name: &'static str,
    kind: CaseKindV2,
    placements: &'static [PlacementModeV2],
    operations: &'static [CorpusOperationV2],
    obligations: &'static [CorpusObligationV2],
    scalars: &'static [i64],
    observations: usize,
    depth: usize,
}

fn build_case(ordinal: usize, spec: CaseSpec) -> CorpusCaseV2 {
    let observations = if ordinal < 12 {
        direct::build_direct_observations_v2(ordinal, spec.observations)
    } else if ordinal == 12 {
        runtime::build_mutation_observations_v2()
    } else {
        runtime::build_rollback_observation_v2()
    };
    let node_keys = (0..spec.placements.len() as u32).collect::<Vec<_>>();
    let query_set = if ordinal < 12 {
        QuerySetV2::DirectComplete
    } else {
        QuerySetV2::RuntimePixelCenters
    };
    let query_inventory = inventory(query_set, &observations);
    let final_viewport = observations
        .last()
        .map_or((0, 0), |item| item.scene.viewport);
    CorpusCaseV2 {
        ordinal: u8::try_from(ordinal).expect("registered case ordinal should fit"),
        name: spec.name,
        kind: spec.kind,
        node_keys,
        placements: spec.placements.to_vec(),
        operations: spec.operations.to_vec(),
        obligations: spec.obligations.to_vec(),
        typed_scalars: spec.scalars.to_vec(),
        query_set,
        observation_count: observations.len(),
        authored_order_digest: digest(spec.name.as_bytes()),
        table_digest: digest_tables(&observations),
        nested_depth: spec.depth,
        query_inventory,
        initial_viewport: observations[0].scene.viewport,
        final_viewport,
        observations,
    }
}

fn inventory(
    query_set: QuerySetV2,
    observations: &[LiteralObservationInputV2],
) -> QueryInventoryV2 {
    let scene = &observations[0].scene;
    let logical_pixel_centers = if query_set == QuerySetV2::RuntimePixelCenters {
        scene.viewport.0 as usize * scene.viewport.1 as usize
    } else {
        0
    };
    QueryInventoryV2 {
        shape_vertices: scene.shapes.len().max(1),
        segment_midpoints: scene.paths.len().max(1),
        clip_corners: scene.clips.len() * 4,
        aabb_centers: scene.paints.len() + scene.hits.len(),
        nonrectangular_misses: scene.shapes.len().max(1),
        authored_boundaries: scene.queries.len().saturating_sub(4).max(1),
        viewport_outside: 4,
        logical_pixel_centers,
        duplicates_retained: true,
    }
}

fn digest_tables(observations: &[LiteralObservationInputV2]) -> u64 {
    let mut bytes = Vec::new();
    for item in observations {
        bytes.extend_from_slice(&item.scene.viewport.0.to_le_bytes());
        bytes.extend_from_slice(&item.scene.viewport.1.to_le_bytes());
        for count in [
            item.scene.nodes.len(),
            item.scene.paths.len(),
            item.scene.shapes.len(),
            item.scene.clips.len(),
            item.scene.brushes.len(),
            item.scene.images.len(),
            item.scene.paints.len(),
            item.scene.hits.len(),
            item.scene.semantics.len(),
            item.scene.queries.len(),
        ] {
            bytes.extend_from_slice(&(count as u64).to_le_bytes());
        }
    }
    digest(&bytes)
}

fn digest(bytes: &[u8]) -> u64 {
    bytes
        .iter()
        .fold(14_695_981_039_346_656_037, |value, byte| {
            (value ^ u64::from(*byte)).wrapping_mul(1_099_511_628_211)
        })
}
