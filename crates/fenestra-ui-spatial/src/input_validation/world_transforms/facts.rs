use super::WorldTransformProof;
use crate::aabb::SpatialAabbV2;
use crate::brush::SpatialBrushKindV2;
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::{SpatialCoverageKindV2, SpatialFillRuleV2};
use crate::paint::SpatialPaintKindV2;
use crate::shape::SpatialShapeKindV2;

type PlacementFact = (u32, i64, i64, i32, i32, i64, i64, i64, i64);
type FlattenedPathFact = (u32, usize, Vec<(i64, i64)>, Vec<(usize, usize, bool)>);

impl WorldTransformProof<'_> {
    pub(in crate::input_validation) fn world_transform_facts(&self) -> Vec<(u32, [i64; 6])> {
        self.world
            .iter()
            .copied()
            .enumerate()
            .map(|(index, transform)| {
                (
                    trusted_ordinal(index),
                    [
                        transform.a().raw(),
                        transform.b().raw(),
                        transform.c().raw(),
                        transform.d().raw(),
                        transform.tx().raw(),
                        transform.ty().raw(),
                    ],
                )
            })
            .collect()
    }

    pub(in crate::input_validation) fn placement_facts(&self) -> Vec<PlacementFact> {
        self.placements.placement_facts()
    }

    pub(in crate::input_validation) fn dependency_order_facts(&self) -> Vec<u32> {
        self.placements.dependency_order_facts()
    }

    pub(in crate::input_validation) fn prepared_island_facts(&self) -> Vec<(u32, Vec<u32>)> {
        self.placements.prepared_island_facts()
    }

    pub(in crate::input_validation) fn path_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.placements.path_range_facts()
    }

    pub(in crate::input_validation) fn validated_path_facts(&self) -> Vec<(u32, usize, usize)> {
        self.placements.validated_path_facts()
    }

    pub(in crate::input_validation) fn polygon_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.placements.polygon_range_facts()
    }

    pub(in crate::input_validation) fn validated_shape_facts(
        &self,
    ) -> Vec<(u32, SpatialShapeKindV2, usize)> {
        self.placements.validated_shape_facts()
    }

    pub(in crate::input_validation) fn gradient_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.placements.gradient_range_facts()
    }

    pub(in crate::input_validation) fn prepared_brush_facts(
        &self,
    ) -> Vec<(u32, SpatialBrushKindV2, usize)> {
        self.placements.prepared_brush_facts()
    }

    pub(in crate::input_validation) fn validated_image_facts(
        &self,
    ) -> Vec<(u32, u32, u32, u32, Vec<u8>)> {
        self.placements.validated_image_facts()
    }

    pub(in crate::input_validation) fn validated_clip_facts(
        &self,
    ) -> Vec<(u32, u32, Option<u32>, u32, SpatialFillRuleV2, usize)> {
        self.placements.validated_clip_facts()
    }

    pub(in crate::input_validation) fn validated_paint_facts(
        &self,
    ) -> Vec<(u32, u32, u32, SpatialPaintKindV2)> {
        self.placements.validated_paint_facts()
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
        self.placements.validated_hit_facts()
    }

    pub(in crate::input_validation) fn validated_semantic_facts(
        &self,
    ) -> Vec<(u32, u32, u32, u32, SpatialFillRuleV2, Option<u32>)> {
        self.placements.validated_semantic_facts()
    }

    pub(in crate::input_validation) fn flattened_path_facts(&self) -> Vec<FlattenedPathFact> {
        self.placements.flattened_path_facts()
    }

    pub(in crate::input_validation) fn shape_local_bounds_facts(
        &self,
    ) -> Vec<(u32, SpatialAabbV2, SpatialAabbV2)> {
        self.placements.shape_local_bounds_facts()
    }

    pub(in crate::input_validation) fn paint_local_bounds_facts(
        &self,
    ) -> Vec<(u32, SpatialAabbV2)> {
        self.placements.paint_local_bounds_facts()
    }

    pub(in crate::input_validation) fn hit_local_bounds_facts(&self) -> Vec<(u32, SpatialAabbV2)> {
        self.placements.hit_local_bounds_facts()
    }

    pub(in crate::input_validation) fn finalized_image_paint_bytes(
        &self,
        paint: u32,
    ) -> Option<&[u8]> {
        self.placements.finalized_image_paint_bytes(paint)
    }
}

fn trusted_ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("phase one validated the spatial node capacity")
}
