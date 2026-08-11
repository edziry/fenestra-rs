use super::fixture::RawInputFixture;
pub(super) use super::validated_clip_support::clip;
pub(super) use super::validated_shape_support::rect_values;
pub(super) use super::world_aabb_support::{
    SCALE, ScriptedLayoutEngine, VIEWPORT, aabb, empty, expect_valid, fact, fill, fixture_with,
    free, hit, identity, limits, owner_node, root, semantic_fill,
};
use crate::coverage::{SpatialClipV2, SpatialFillRuleV2};
use crate::shape::SpatialShapeV2;

pub(super) fn clips_only_fixture(
    shapes: Vec<SpatialShapeV2>,
    clips: Vec<SpatialClipV2>,
) -> RawInputFixture {
    fixture_with(
        vec![root(), owner_node(1, identity(), 20, 20)],
        shapes,
        clips,
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}

pub(super) const fn nonzero_clip(
    key: u32,
    owner: u32,
    parent: Option<u32>,
    shape: u32,
) -> SpatialClipV2 {
    clip(key, owner, parent, shape, SpatialFillRuleV2::NonZero)
}

pub(super) const fn even_odd_clip(
    key: u32,
    owner: u32,
    parent: Option<u32>,
    shape: u32,
) -> SpatialClipV2 {
    clip(key, owner, parent, shape, SpatialFillRuleV2::EvenOdd)
}
