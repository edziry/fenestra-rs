use super::{ValidatedSemanticItemsProof, trusted_semantic_ordinal};
use crate::coverage::SpatialFillRuleV2;

impl<'a> ValidatedSemanticItemsProof<'a> {
    pub(in crate::input_validation) fn validated_semantic_facts(
        &self,
    ) -> Vec<(u32, u32, u32, u32, SpatialFillRuleV2, Option<u32>)> {
        self.semantics
            .iter()
            .enumerate()
            .map(|(index, semantic)| {
                (
                    trusted_semantic_ordinal(index),
                    semantic.owner,
                    semantic.item_ordinal,
                    semantic.shape,
                    semantic.fill_rule,
                    semantic.clip,
                )
            })
            .collect()
    }

    pub(in crate::input_validation) fn validated_hit_facts(
        &self,
    ) -> Vec<(
        u32,
        u32,
        u32,
        crate::coverage::SpatialCoverageKindV2,
        crate::content_item::SpatialInputPolicyV2,
        Option<u32>,
    )> {
        self.hits.validated_hit_facts()
    }

    pub(in crate::input_validation) fn validated_fill_hit_facts(
        &self,
    ) -> Vec<(u32, u32, SpatialFillRuleV2)> {
        self.hits.validated_fill_hit_facts()
    }

    pub(in crate::input_validation) fn validated_stroke_hit_facts(&self) -> Vec<(u32, u32, i64)> {
        self.hits.validated_stroke_hit_facts()
    }

    pub(in crate::input_validation) fn validated_paint_facts(
        &self,
    ) -> Vec<(u32, u32, u32, crate::paint::SpatialPaintKindV2)> {
        self.hits.validated_paint_facts()
    }

    pub(in crate::input_validation) fn validated_image_paint_bytes(
        &self,
        paint: u32,
    ) -> Option<&[u8]> {
        self.hits.validated_image_paint_bytes(paint)
    }

    pub(in crate::input_validation) fn validated_clip_facts(
        &self,
    ) -> Vec<(u32, u32, Option<u32>, u32, SpatialFillRuleV2, usize)> {
        self.hits.validated_clip_facts()
    }

    pub(in crate::input_validation) fn validated_image_facts(
        &self,
    ) -> Vec<(u32, u32, u32, u32, Vec<u8>)> {
        self.hits.validated_image_facts()
    }

    pub(in crate::input_validation) fn accepted_pixel_total(&self) -> u128 {
        self.hits.accepted_pixel_total()
    }

    pub(in crate::input_validation) fn prepared_brush_facts(
        &self,
    ) -> Vec<(u32, crate::brush::SpatialBrushKindV2, usize)> {
        self.hits.prepared_brush_facts()
    }

    pub(in crate::input_validation) fn gradient_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.hits.gradient_range_facts()
    }

    pub(in crate::input_validation) fn validated_shape_facts(
        &self,
    ) -> Vec<(u32, crate::shape::SpatialShapeKindV2, usize)> {
        self.hits.validated_shape_facts()
    }

    pub(in crate::input_validation) fn polygon_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.hits.polygon_range_facts()
    }

    pub(in crate::input_validation) fn validated_path_facts(&self) -> Vec<(u32, usize, usize)> {
        self.hits.validated_path_facts()
    }

    pub(in crate::input_validation) fn subpath_total(&self) -> usize {
        self.hits.subpath_total()
    }

    pub(in crate::input_validation) fn path_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.hits.path_range_facts()
    }

    pub(in crate::input_validation) fn prepared_island_facts(&self) -> Vec<(u32, Vec<u32>)> {
        self.hits.prepared_island_facts()
    }
}
