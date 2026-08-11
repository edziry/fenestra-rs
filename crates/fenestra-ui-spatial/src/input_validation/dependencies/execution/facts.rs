use super::BasePlacementProof;
use crate::aabb::SpatialAabbV2;
use crate::brush::SpatialBrushKindV2;
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::{SpatialCoverageKindV2, SpatialFillRuleV2};
use crate::paint::SpatialPaintKindV2;
use crate::shape::SpatialShapeKindV2;

type PlacementFact = (u32, i64, i64, i32, i32, i64, i64, i64, i64);
type DependencyUnitFact = (u32, Option<(u32, u32)>, Vec<u32>, Vec<u32>);
type FlattenedPathFact = (u32, usize, Vec<(i64, i64)>, Vec<(usize, usize, bool)>);

impl BasePlacementProof<'_> {
    pub(in crate::input_validation) fn placement_facts(&self) -> Vec<PlacementFact> {
        self.placements
            .iter()
            .enumerate()
            .map(|(index, placement)| {
                (
                    trusted_ordinal(index),
                    placement.origin().x().raw(),
                    placement.origin().y().raw(),
                    placement.width,
                    placement.height,
                    placement.far_x.raw(),
                    placement.far_y.raw(),
                    placement.local_origin().x().raw(),
                    placement.local_origin().y().raw(),
                )
            })
            .collect()
    }

    pub(in crate::input_validation) fn dependency_unit_facts(&self) -> Vec<DependencyUnitFact> {
        self.graph.dependency_unit_facts()
    }

    pub(in crate::input_validation) fn dependency_order_facts(&self) -> Vec<u32> {
        self.graph.dependency_order_facts()
    }

    pub(in crate::input_validation) fn shape_local_bounds_facts(
        &self,
    ) -> Vec<(u32, SpatialAabbV2, SpatialAabbV2)> {
        self.graph.shape_local_bounds_facts()
    }

    pub(in crate::input_validation) fn paint_local_bounds_facts(
        &self,
    ) -> Vec<(u32, SpatialAabbV2)> {
        self.graph.paint_local_bounds_facts()
    }

    pub(in crate::input_validation) fn hit_local_bounds_facts(&self) -> Vec<(u32, SpatialAabbV2)> {
        self.graph.hit_local_bounds_facts()
    }

    pub(in crate::input_validation) fn finalized_image_paint_bytes(
        &self,
        paint: u32,
    ) -> Option<&[u8]> {
        self.graph.finalized_image_paint_bytes(paint)
    }

    pub(in crate::input_validation) fn flattened_path_facts(&self) -> Vec<FlattenedPathFact> {
        self.graph.flattened_path_facts()
    }

    pub(in crate::input_validation) fn validated_semantic_facts(
        &self,
    ) -> Vec<(u32, u32, u32, u32, SpatialFillRuleV2, Option<u32>)> {
        self.graph.validated_semantic_facts()
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
        self.graph.validated_hit_facts()
    }

    pub(in crate::input_validation) fn validated_paint_facts(
        &self,
    ) -> Vec<(u32, u32, u32, SpatialPaintKindV2)> {
        self.graph.validated_paint_facts()
    }

    pub(in crate::input_validation) fn validated_clip_facts(
        &self,
    ) -> Vec<(u32, u32, Option<u32>, u32, SpatialFillRuleV2, usize)> {
        self.graph.validated_clip_facts()
    }

    pub(in crate::input_validation) fn validated_image_facts(
        &self,
    ) -> Vec<(u32, u32, u32, u32, Vec<u8>)> {
        self.graph.validated_image_facts()
    }

    pub(in crate::input_validation) fn prepared_brush_facts(
        &self,
    ) -> Vec<(u32, SpatialBrushKindV2, usize)> {
        self.graph.prepared_brush_facts()
    }

    pub(in crate::input_validation) fn gradient_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.graph.gradient_range_facts()
    }

    pub(in crate::input_validation) fn validated_shape_facts(
        &self,
    ) -> Vec<(u32, SpatialShapeKindV2, usize)> {
        self.graph.validated_shape_facts()
    }

    pub(in crate::input_validation) fn polygon_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.graph.polygon_range_facts()
    }

    pub(in crate::input_validation) fn validated_path_facts(&self) -> Vec<(u32, usize, usize)> {
        self.graph.validated_path_facts()
    }

    pub(in crate::input_validation) fn path_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.graph.path_range_facts()
    }

    pub(in crate::input_validation) fn prepared_island_facts(&self) -> Vec<(u32, Vec<u32>)> {
        self.graph.prepared_island_facts()
    }
}

fn trusted_ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("phase one validated the spatial node capacity")
}
