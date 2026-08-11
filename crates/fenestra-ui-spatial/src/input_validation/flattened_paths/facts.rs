use super::FlattenedPathsProof;
use crate::brush::SpatialBrushKindV2;
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::{SpatialCoverageKindV2, SpatialFillRuleV2};
use crate::paint::SpatialPaintKindV2;
use crate::shape::SpatialShapeKindV2;

type FlattenedPathFact = (u32, usize, Vec<(i64, i64)>, Vec<(usize, usize, bool)>);

impl FlattenedPathsProof<'_> {
    pub(in crate::input_validation) fn flattened_path_facts(&self) -> Vec<FlattenedPathFact> {
        self.paths
            .iter()
            .enumerate()
            .map(|(index, path)| {
                let points = path
                    .points()
                    .iter()
                    .map(|point| (point.x().raw(), point.y().raw()))
                    .collect();
                let subpaths = path
                    .subpaths()
                    .iter()
                    .copied()
                    .map(|subpath| {
                        (
                            subpath.point_start(),
                            subpath.point_length(),
                            subpath.is_explicitly_closed(),
                        )
                    })
                    .collect();
                (
                    u32::try_from(index).expect("phase one validated the path row capacity"),
                    path.segment_count(),
                    points,
                    subpaths,
                )
            })
            .collect()
    }

    pub(in crate::input_validation) fn accepted_flattened_segment_total(&self) -> u128 {
        self.accepted_segments as u128
    }

    pub(in crate::input_validation) fn validated_semantic_facts(
        &self,
    ) -> Vec<(u32, u32, u32, u32, SpatialFillRuleV2, Option<u32>)> {
        self.semantics.validated_semantic_facts()
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
        self.semantics.validated_hit_facts()
    }

    pub(in crate::input_validation) fn validated_fill_hit_facts(
        &self,
    ) -> Vec<(u32, u32, SpatialFillRuleV2)> {
        self.semantics.validated_fill_hit_facts()
    }

    pub(in crate::input_validation) fn validated_stroke_hit_facts(&self) -> Vec<(u32, u32, i64)> {
        self.semantics.validated_stroke_hit_facts()
    }

    pub(in crate::input_validation) fn validated_paint_facts(
        &self,
    ) -> Vec<(u32, u32, u32, SpatialPaintKindV2)> {
        self.semantics.validated_paint_facts()
    }

    pub(in crate::input_validation) fn validated_image_paint_bytes(
        &self,
        paint: u32,
    ) -> Option<&[u8]> {
        self.semantics.validated_image_paint_bytes(paint)
    }

    pub(in crate::input_validation) fn validated_clip_facts(
        &self,
    ) -> Vec<(u32, u32, Option<u32>, u32, SpatialFillRuleV2, usize)> {
        self.semantics.validated_clip_facts()
    }

    pub(in crate::input_validation) fn validated_image_facts(
        &self,
    ) -> Vec<(u32, u32, u32, u32, Vec<u8>)> {
        self.semantics.validated_image_facts()
    }

    pub(in crate::input_validation) fn accepted_pixel_total(&self) -> u128 {
        self.semantics.accepted_pixel_total()
    }

    pub(in crate::input_validation) fn prepared_brush_facts(
        &self,
    ) -> Vec<(u32, SpatialBrushKindV2, usize)> {
        self.semantics.prepared_brush_facts()
    }

    pub(in crate::input_validation) fn gradient_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.semantics.gradient_range_facts()
    }

    pub(in crate::input_validation) fn validated_shape_facts(
        &self,
    ) -> Vec<(u32, SpatialShapeKindV2, usize)> {
        self.semantics.validated_shape_facts()
    }

    pub(in crate::input_validation) fn polygon_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.semantics.polygon_range_facts()
    }

    pub(in crate::input_validation) fn validated_path_facts(&self) -> Vec<(u32, usize, usize)> {
        self.semantics.validated_path_facts()
    }

    pub(in crate::input_validation) fn subpath_total(&self) -> usize {
        self.semantics.subpath_total()
    }

    pub(in crate::input_validation) fn path_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.semantics.path_range_facts()
    }

    pub(in crate::input_validation) fn prepared_island_facts(&self) -> Vec<(u32, Vec<u32>)> {
        self.semantics.prepared_island_facts()
    }
}
