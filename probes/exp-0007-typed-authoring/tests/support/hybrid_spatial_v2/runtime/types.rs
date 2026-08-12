use fenestra_ui_ir::prototype::{InvalidationSet, PropertyValue, SourceSpan};
use fenestra_ui_spatial::prototype::{
    SpatialErrorLocationV2, SpatialResolveErrorKindV2, SpatialViewportV2,
};

use super::path::{FragmentPath, NodePath};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoredSpatialLaneLog {
    pub(super) observations: Vec<NormalizedObservation>,
    pub(super) final_keys: Vec<u64>,
    pub(super) noop: NoopChecks,
    pub(super) failure: NormalizedFailure,
}

impl AuthoredSpatialLaneLog {
    pub fn generations(&self) -> Vec<u64> {
        self.observations
            .iter()
            .map(|observation| observation.generation)
            .collect()
    }

    pub fn viewports(&self) -> Vec<SpatialViewportV2> {
        self.observations
            .iter()
            .map(|observation| observation.viewport)
            .collect()
    }

    pub fn mapping_counts(&self) -> Vec<usize> {
        self.observations
            .iter()
            .map(|observation| observation.projection.mapping.len())
            .collect()
    }

    pub fn geometry_counts(&self) -> Vec<usize> {
        self.observations
            .iter()
            .map(|observation| observation.projection.geometry.len())
            .collect()
    }

    pub fn clip_counts(&self) -> Vec<usize> {
        self.observations
            .iter()
            .map(|observation| observation.projection.clips.len())
            .collect()
    }

    pub fn paint_counts(&self) -> Vec<usize> {
        self.observations
            .iter()
            .map(|observation| observation.projection.paints.len())
            .collect()
    }

    pub fn hit_counts(&self) -> Vec<usize> {
        self.observations
            .iter()
            .map(|observation| observation.projection.hits.len())
            .collect()
    }

    pub fn semantic_counts(&self) -> Vec<usize> {
        self.observations
            .iter()
            .map(|observation| observation.projection.semantics.len())
            .collect()
    }

    pub fn hit_query_counts(&self) -> Vec<usize> {
        self.observations
            .iter()
            .map(|observation| observation.hit_queries.len())
            .collect()
    }

    pub fn raster_byte_counts(&self) -> Vec<usize> {
        self.observations
            .iter()
            .map(|observation| observation.raster.bytes.len())
            .collect()
    }

    pub fn final_keys(&self) -> &[u64] {
        &self.final_keys
    }

    pub const fn noop_checks(&self) -> NoopChecks {
        self.noop
    }

    pub const fn failure(&self) -> &NormalizedFailure {
        &self.failure
    }

    pub fn authored_factor_span(&self) -> SourceSpan {
        self.failure.authored_factor_span
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NoopChecks {
    empty: bool,
    same_value: bool,
    round_trip: bool,
}

impl NoopChecks {
    pub(super) const fn new(empty: bool, same_value: bool, round_trip: bool) -> Self {
        Self {
            empty,
            same_value,
            round_trip,
        }
    }

    pub const fn all_preserved(self) -> bool {
        self.empty && self.same_value && self.round_trip
    }

    pub const fn empty_preserved(self) -> bool {
        self.empty
    }

    pub const fn same_value_preserved(self) -> bool {
        self.same_value
    }

    pub const fn round_trip_preserved(self) -> bool {
        self.round_trip
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedFailure {
    resolve_kind: SpatialResolveErrorKindV2,
    resolve_location: SpatialErrorLocationV2,
    ir_span: SourceSpan,
    operation_index: Option<usize>,
    outer_state_preserved: bool,
    spatial_snapshot_preserved: bool,
    complete_observation_preserved: bool,
    authored_factor_span: SourceSpan,
}

impl NormalizedFailure {
    #[allow(clippy::too_many_arguments)]
    pub(super) const fn new(
        resolve_kind: SpatialResolveErrorKindV2,
        resolve_location: SpatialErrorLocationV2,
        ir_span: SourceSpan,
        operation_index: Option<usize>,
        outer_state_preserved: bool,
        spatial_snapshot_preserved: bool,
        complete_observation_preserved: bool,
        authored_factor_span: SourceSpan,
    ) -> Self {
        Self {
            resolve_kind,
            resolve_location,
            ir_span,
            operation_index,
            outer_state_preserved,
            spatial_snapshot_preserved,
            complete_observation_preserved,
            authored_factor_span,
        }
    }

    pub const fn resolve_kind(&self) -> SpatialResolveErrorKindV2 {
        self.resolve_kind
    }

    pub const fn resolve_location(&self) -> SpatialErrorLocationV2 {
        self.resolve_location
    }

    pub const fn ir_span(&self) -> SourceSpan {
        self.ir_span
    }

    pub const fn operation_index(&self) -> Option<usize> {
        self.operation_index
    }

    pub const fn outer_state_preserved(&self) -> bool {
        self.outer_state_preserved
    }

    pub const fn spatial_snapshot_preserved(&self) -> bool {
        self.spatial_snapshot_preserved
    }

    pub const fn complete_observation_preserved(&self) -> bool {
        self.complete_observation_preserved
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NormalizedObservation {
    pub(super) generation: u64,
    pub(super) viewport: SpatialViewportV2,
    pub(super) receipt: NormalizedReceipt,
    pub(super) state: NormalizedState,
    pub(super) projection: NormalizedProjection,
    pub(super) hit_queries: Vec<NormalizedHitQuery>,
    pub(super) raster: NormalizedRaster,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NormalizedState {
    pub(super) nodes: Vec<NormalizedNode>,
    pub(super) fragments: Vec<NormalizedFragment>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NormalizedNode {
    pub(super) path: NodePath,
    pub(super) parent: Option<NodePath>,
    pub(super) template: u32,
    pub(super) component: u32,
    pub(super) properties: Vec<(u32, PropertyValue)>,
    pub(super) children: Vec<NormalizedChild>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum NormalizedChild {
    Static(NodePath),
    Region(FragmentPath),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NormalizedFragment {
    pub(super) path: FragmentPath,
    pub(super) descriptor: u32,
    pub(super) members: Vec<(u64, NodePath)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NormalizedReceipt {
    pub(super) generation: u64,
    pub(super) invalidation: InvalidationSet,
    pub(super) mutations: Vec<NormalizedMutation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum NormalizedMutation {
    Property {
        node: NodePath,
        property: u32,
        old: PropertyValue,
        new: PropertyValue,
    },
    Insert {
        fragment: FragmentPath,
        key: u64,
        root: NodePath,
        final_index: usize,
        created: Vec<NormalizedManifestEntry>,
    },
    Move {
        fragment: FragmentPath,
        key: u64,
        root: NodePath,
        old_index: usize,
        final_index: usize,
    },
    Remove {
        fragment: FragmentPath,
        key: u64,
        root: NodePath,
        old_index: usize,
        retired: Vec<NormalizedManifestEntry>,
    },
    Viewport {
        old: SpatialViewportV2,
        new: SpatialViewportV2,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum NormalizedManifestEntry {
    Node(NodePath),
    Fragment(FragmentPath),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NormalizedProjection {
    pub(super) mapping: Vec<(u32, Option<NodePath>)>,
    pub(super) geometry: Vec<NormalizedGeometry>,
    pub(super) clips: Vec<NormalizedClip>,
    pub(super) paints: Vec<NormalizedPaint>,
    pub(super) hits: Vec<NormalizedItem>,
    pub(super) semantics: Vec<NormalizedItem>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct NormalizedAffine(pub(super) [i64; 6]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct NormalizedAabb {
    pub(super) empty: bool,
    pub(super) edges: [i64; 4],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NormalizedGeometry {
    pub(super) key: u32,
    pub(super) path: Option<NodePath>,
    pub(super) base: [i64; 4],
    pub(super) affine: NormalizedAffine,
    pub(super) determinant: i128,
    pub(super) aabb: NormalizedAabb,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NormalizedClip {
    pub(super) key: u32,
    pub(super) owner: u32,
    pub(super) path: NodePath,
    pub(super) parent: Option<u32>,
    pub(super) shape: u32,
    pub(super) affine: NormalizedAffine,
    pub(super) determinant: i128,
    pub(super) primitive: NormalizedAabb,
    pub(super) effective: NormalizedAabb,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NormalizedPaintReference {
    Coverage { shape: u32, brush: u32 },
    Image { image: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NormalizedPaint {
    pub(super) key: u32,
    pub(super) owner: u32,
    pub(super) path: NodePath,
    pub(super) affine: NormalizedAffine,
    pub(super) determinant: i128,
    pub(super) aabb: NormalizedAabb,
    pub(super) reference: NormalizedPaintReference,
    pub(super) clip: Option<u32>,
    pub(super) stack: u32,
    pub(super) item: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NormalizedItem {
    pub(super) key: u32,
    pub(super) owner: u32,
    pub(super) path: NodePath,
    pub(super) affine: NormalizedAffine,
    pub(super) determinant: i128,
    pub(super) aabb: NormalizedAabb,
    pub(super) shape: u32,
    pub(super) clip: Option<u32>,
    pub(super) stack: u32,
    pub(super) item: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NormalizedHitQuery {
    pub(super) scene: [i64; 2],
    pub(super) result: Option<NormalizedHit>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NormalizedHit {
    pub(super) key: u32,
    pub(super) owner: u32,
    pub(super) path: NodePath,
    pub(super) item: u32,
    pub(super) local: [i64; 2],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NormalizedRaster {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) stride: u64,
    pub(super) bytes: Box<[u8]>,
}
