use super::EffectiveClipAabbProof;
use crate::aabb::SpatialAabbV2;
use crate::brush::SpatialBrushKindV2;
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::{SpatialCoverageKindV2, SpatialFillRuleV2};
use crate::paint::SpatialPaintKindV2;
use crate::shape::SpatialShapeKindV2;

type AabbFact = (u32, SpatialAabbV2);
type PlacementFact = (u32, i64, i64, i32, i32, i64, i64, i64, i64);
type FlattenedPathFact = (u32, usize, Vec<(i64, i64)>, Vec<(usize, usize, bool)>);

impl EffectiveClipAabbProof<'_> {
    pub(in crate::input_validation) fn effective_clip_world_aabb_facts(&self) -> Vec<AabbFact> {
        self.effective_clips
            .iter()
            .copied()
            .enumerate()
            .map(|(index, bounds)| (trusted_ordinal(index), bounds))
            .collect()
    }

    pub(in crate::input_validation) fn geometry_world_aabb_facts(&self) -> Vec<AabbFact> {
        self.world.geometry_world_aabb_facts()
    }

    pub(in crate::input_validation) fn clip_world_aabb_facts(&self) -> Vec<AabbFact> {
        self.world.clip_world_aabb_facts()
    }

    pub(in crate::input_validation) fn paint_world_aabb_facts(&self) -> Vec<AabbFact> {
        self.world.paint_world_aabb_facts()
    }

    pub(in crate::input_validation) fn hit_world_aabb_facts(&self) -> Vec<AabbFact> {
        self.world.hit_world_aabb_facts()
    }

    pub(in crate::input_validation) fn semantic_world_aabb_facts(&self) -> Vec<AabbFact> {
        self.world.semantic_world_aabb_facts()
    }

    pub(in crate::input_validation) fn world_transform_facts(&self) -> Vec<(u32, [i64; 6])> {
        self.world.world_transform_facts()
    }

    pub(in crate::input_validation) fn placement_facts(&self) -> Vec<PlacementFact> {
        self.world.placement_facts()
    }

    pub(in crate::input_validation) fn dependency_order_facts(&self) -> Vec<u32> {
        self.world.dependency_order_facts()
    }

    pub(in crate::input_validation) fn prepared_island_facts(&self) -> Vec<(u32, Vec<u32>)> {
        self.world.prepared_island_facts()
    }

    pub(in crate::input_validation) fn path_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.world.path_range_facts()
    }

    pub(in crate::input_validation) fn validated_path_facts(&self) -> Vec<(u32, usize, usize)> {
        self.world.validated_path_facts()
    }

    pub(in crate::input_validation) fn polygon_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.world.polygon_range_facts()
    }

    pub(in crate::input_validation) fn validated_shape_facts(
        &self,
    ) -> Vec<(u32, SpatialShapeKindV2, usize)> {
        self.world.validated_shape_facts()
    }

    pub(in crate::input_validation) fn gradient_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.world.gradient_range_facts()
    }

    pub(in crate::input_validation) fn prepared_brush_facts(
        &self,
    ) -> Vec<(u32, SpatialBrushKindV2, usize)> {
        self.world.prepared_brush_facts()
    }

    pub(in crate::input_validation) fn validated_image_facts(
        &self,
    ) -> Vec<(u32, u32, u32, u32, Vec<u8>)> {
        self.world.validated_image_facts()
    }

    pub(in crate::input_validation) fn validated_clip_facts(
        &self,
    ) -> Vec<(u32, u32, Option<u32>, u32, SpatialFillRuleV2, usize)> {
        self.world.validated_clip_facts()
    }

    pub(in crate::input_validation) fn validated_paint_facts(
        &self,
    ) -> Vec<(u32, u32, u32, SpatialPaintKindV2)> {
        self.world.validated_paint_facts()
    }

    pub(in crate::input_validation) fn validated_hit_facts(
        &self,
    ) -> Vec<(
        u32,
        u32,
        u32,
        SpatialCoverageKindV2,
        SpatialInputPolicyV2,
        Option<u32>,
    )> {
        self.world.validated_hit_facts()
    }

    pub(in crate::input_validation) fn validated_semantic_facts(
        &self,
    ) -> Vec<(u32, u32, u32, u32, SpatialFillRuleV2, Option<u32>)> {
        self.world.validated_semantic_facts()
    }

    pub(in crate::input_validation) fn flattened_path_facts(&self) -> Vec<FlattenedPathFact> {
        self.world.flattened_path_facts()
    }

    pub(in crate::input_validation) fn shape_local_bounds_facts(
        &self,
    ) -> Vec<(u32, SpatialAabbV2, SpatialAabbV2)> {
        self.world.shape_local_bounds_facts()
    }

    pub(in crate::input_validation) fn paint_local_bounds_facts(
        &self,
    ) -> Vec<(u32, SpatialAabbV2)> {
        self.world.paint_local_bounds_facts()
    }

    pub(in crate::input_validation) fn hit_local_bounds_facts(&self) -> Vec<(u32, SpatialAabbV2)> {
        self.world.hit_local_bounds_facts()
    }

    pub(in crate::input_validation) fn finalized_image_paint_bytes(
        &self,
        paint: u32,
    ) -> Option<&[u8]> {
        self.world.finalized_image_paint_bytes(paint)
    }
}

fn trusted_ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("phase one validated the effective-clip row capacity")
}
