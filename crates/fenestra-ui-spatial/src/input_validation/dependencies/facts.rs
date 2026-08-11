use super::{DependencyGraphProof, DependencyUnitKind};
use crate::aabb::SpatialAabbV2;
use crate::brush::SpatialBrushKindV2;
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::{SpatialCoverageKindV2, SpatialFillRuleV2};
use crate::image::{SpatialImageDestinationRectV2, SpatialImageSourceRectV2};
use crate::paint::SpatialPaintKindV2;
use crate::shape::SpatialShapeKindV2;

type DependencyUnitFact = (u32, Option<(u32, u32)>, Vec<u32>, Vec<u32>);
type FlattenedPathFact = (u32, usize, Vec<(i64, i64)>, Vec<(usize, usize, bool)>);

impl DependencyGraphProof<'_> {
    pub(in crate::input_validation) fn dependency_unit_facts(&self) -> Vec<DependencyUnitFact> {
        self.units
            .iter()
            .map(|unit| {
                let island = match unit.kind {
                    DependencyUnitKind::Free { .. } => None,
                    DependencyUnitKind::Island { index, host } => Some((index, host)),
                };
                let incoming = unit
                    .incoming
                    .iter()
                    .map(|&producer| self.units[producer].ordinal)
                    .collect();
                (unit.ordinal, island, unit.produced.clone(), incoming)
            })
            .collect()
    }

    pub(in crate::input_validation) fn dependency_order_facts(&self) -> Vec<u32> {
        self.order
            .iter()
            .map(|&index| self.units[index].ordinal)
            .collect()
    }

    pub(in crate::input_validation) const fn dependency_edge_count(&self) -> u128 {
        self.edge_count
    }

    pub(in crate::input_validation) fn shape_local_bounds_facts(
        &self,
    ) -> Vec<(u32, SpatialAabbV2, SpatialAabbV2)> {
        self.bounds.shape_local_bounds_facts()
    }

    pub(in crate::input_validation) fn paint_local_bounds_facts(
        &self,
    ) -> Vec<(u32, SpatialAabbV2)> {
        self.bounds.paint_local_bounds_facts()
    }

    pub(in crate::input_validation) fn hit_local_bounds_facts(&self) -> Vec<(u32, SpatialAabbV2)> {
        self.bounds.hit_local_bounds_facts()
    }

    pub(in crate::input_validation) fn finalized_image_paint_facts(
        &self,
    ) -> Vec<(
        u32,
        SpatialImageSourceRectV2,
        SpatialImageDestinationRectV2,
        u8,
        SpatialAabbV2,
    )> {
        self.bounds.finalized_image_paint_facts()
    }

    pub(in crate::input_validation) fn finalized_image_paint_bytes(
        &self,
        paint: u32,
    ) -> Option<&[u8]> {
        self.bounds.finalized_image_paint_bytes(paint)
    }

    pub(in crate::input_validation) fn flattened_path_facts(&self) -> Vec<FlattenedPathFact> {
        self.bounds.flattened_path_facts()
    }

    pub(in crate::input_validation) fn accepted_flattened_segment_total(&self) -> u128 {
        self.bounds.accepted_flattened_segment_total()
    }

    pub(in crate::input_validation) fn validated_semantic_facts(
        &self,
    ) -> Vec<(u32, u32, u32, u32, SpatialFillRuleV2, Option<u32>)> {
        self.bounds.validated_semantic_facts()
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
        self.bounds.validated_hit_facts()
    }

    pub(in crate::input_validation) fn validated_paint_facts(
        &self,
    ) -> Vec<(u32, u32, u32, SpatialPaintKindV2)> {
        self.bounds.validated_paint_facts()
    }

    pub(in crate::input_validation) fn validated_clip_facts(
        &self,
    ) -> Vec<(u32, u32, Option<u32>, u32, SpatialFillRuleV2, usize)> {
        self.bounds.validated_clip_facts()
    }

    pub(in crate::input_validation) fn validated_image_facts(
        &self,
    ) -> Vec<(u32, u32, u32, u32, Vec<u8>)> {
        self.bounds.validated_image_facts()
    }

    pub(in crate::input_validation) fn accepted_pixel_total(&self) -> u128 {
        self.bounds.accepted_pixel_total()
    }

    pub(in crate::input_validation) fn prepared_brush_facts(
        &self,
    ) -> Vec<(u32, SpatialBrushKindV2, usize)> {
        self.bounds.prepared_brush_facts()
    }

    pub(in crate::input_validation) fn gradient_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.bounds.gradient_range_facts()
    }

    pub(in crate::input_validation) fn validated_shape_facts(
        &self,
    ) -> Vec<(u32, SpatialShapeKindV2, usize)> {
        self.bounds.validated_shape_facts()
    }

    pub(in crate::input_validation) fn polygon_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.bounds.polygon_range_facts()
    }

    pub(in crate::input_validation) fn validated_path_facts(&self) -> Vec<(u32, usize, usize)> {
        self.bounds.validated_path_facts()
    }

    pub(in crate::input_validation) fn subpath_total(&self) -> usize {
        self.bounds.subpath_total()
    }

    pub(in crate::input_validation) fn path_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.bounds.path_range_facts()
    }

    pub(in crate::input_validation) fn prepared_island_facts(&self) -> Vec<(u32, Vec<u32>)> {
        self.bounds.prepared_island_facts()
    }
}
