use std::sync::Arc;

use super::super::fixture::RawInputFixture;
use super::super::validated_hit_support::{fill, stroke};
use super::super::world_aabb_support::owner_node;
use super::super::world_transform_support::{SCALE, VIEWPORT, identity, root};
use super::support::{requested_limits, zero_call_engine};
use super::validator_support::CandidateTables;
use super::*;
use crate::content_item::{SpatialHitV2, SpatialInputPolicyV2};
use crate::coverage::{SpatialClipV2, SpatialFillRuleV2};
use crate::model::{SpatialPointV2, SpatialScalarV2};
use crate::owned_input::SpatialOwnedInputV2;
use crate::path::{SpatialPathV2, SpatialPathVerbV2};
use crate::shape::SpatialShapeV2;
use crate::topology::SpatialNodeV2;

pub(super) const S: i64 = SCALE;

#[allow(clippy::too_many_arguments)]
pub(super) fn owned_fixture(
    nodes: Vec<SpatialNodeV2>,
    shapes: Vec<SpatialShapeV2>,
    polygon_points: Vec<SpatialPointV2>,
    paths: Vec<SpatialPathV2>,
    path_verbs: Vec<SpatialPathVerbV2>,
    clips: Vec<SpatialClipV2>,
    hits: Vec<SpatialHitV2>,
) -> Arc<SpatialOwnedInputV2> {
    Arc::new(
        super::super::world_transform_support::fixture(nodes)
            .with_paths(paths, path_verbs)
            .with_shapes(shapes, polygon_points)
            .with_brushes(Vec::new(), Vec::new())
            .with_images(Vec::new())
            .with_clips(clips)
            .with_paint_items(Vec::new())
            .with_hit_items(hits)
            .with_semantic_items(Vec::new())
            .into_owned(VIEWPORT),
    )
}

pub(super) fn root_and_owners(count: u32, width: i32, height: i32) -> Vec<SpatialNodeV2> {
    let mut nodes = vec![root()];
    nodes.extend((1..=count).map(|key| owner_node(key, identity(), width, height)));
    nodes
}

pub(super) fn reference_snapshot(source: Arc<SpatialOwnedInputV2>) -> SpatialResolvedSnapshotV2 {
    let prepared = prepare_spatial_v2(&zero_call_engine(), source, requested_limits())
        .expect("hit fixture prepares");
    materialize_reference_spatial_v2(prepared)
}

pub(super) fn candidate_case(
    source: Arc<SpatialOwnedInputV2>,
) -> (PreparedSpatialV2, CandidateTables) {
    let prepared = prepare_spatial_v2(&zero_call_engine(), source.clone(), requested_limits())
        .expect("hit fixture prepares");
    let reference = reference_snapshot(source);
    (prepared, CandidateTables::from_snapshot(&reference))
}

pub(super) fn accepting_fill(
    owner: u32,
    item: u32,
    shape: u32,
    clip: Option<u32>,
    rule: SpatialFillRuleV2,
) -> SpatialHitV2 {
    fill(owner, item, shape, clip, rule, SpatialInputPolicyV2::Accept)
}

pub(super) fn ignored_fill(owner: u32, item: u32, shape: u32) -> SpatialHitV2 {
    fill(
        owner,
        item,
        shape,
        None,
        SpatialFillRuleV2::NonZero,
        SpatialInputPolicyV2::Ignore,
    )
}

pub(super) fn accepting_stroke(
    owner: u32,
    item: u32,
    shape: u32,
    width: i64,
    clip: Option<u32>,
) -> SpatialHitV2 {
    stroke(
        owner,
        item,
        shape,
        width,
        clip,
        SpatialInputPolicyV2::Accept,
    )
}

pub(super) const fn point(x: i64, y: i64) -> SpatialPointV2 {
    SpatialPointV2::new(SpatialScalarV2::new(x), SpatialScalarV2::new(y))
}

pub(super) fn assert_hit(
    result: Option<SpatialHitResultV2>,
    key: u32,
    owner: u32,
    item: u32,
    local: SpatialPointV2,
) {
    let result = result.expect("expected exact hit");
    assert_eq!(result.key(), key);
    assert_eq!(result.owner().get(), owner);
    assert_eq!(result.item_ordinal(), item);
    assert_eq!(result.local_point(), local);
}

pub(super) fn empty_fixture() -> RawInputFixture {
    super::super::world_transform_support::fixture(vec![root()])
        .with_paths(Vec::new(), Vec::new())
        .with_shapes(Vec::new(), Vec::new())
        .with_brushes(Vec::new(), Vec::new())
        .with_images(Vec::new())
        .with_clips(Vec::new())
        .with_paint_items(Vec::new())
        .with_hit_items(Vec::new())
        .with_semantic_items(Vec::new())
}
