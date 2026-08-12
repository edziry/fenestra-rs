use super::{ValidatedHitCoverage, ValidatedHitItemsProof, trusted_hit_ordinal};
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::{SpatialCoverageKindV2, SpatialFillRuleV2};

impl<'a> ValidatedHitItemsProof<'a> {
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
        self.hits
            .iter()
            .enumerate()
            .map(|(index, hit)| {
                let kind = match hit.coverage {
                    ValidatedHitCoverage::Fill { .. } => SpatialCoverageKindV2::Fill,
                    ValidatedHitCoverage::RoundStroke { .. } => SpatialCoverageKindV2::RoundStroke,
                };
                (
                    trusted_hit_ordinal(index),
                    hit.owner,
                    hit.item_ordinal,
                    kind,
                    hit.input_policy,
                    hit.clip,
                )
            })
            .collect()
    }

    pub(in crate::input_validation) fn validated_fill_hit_facts(
        &self,
    ) -> Vec<(u32, u32, SpatialFillRuleV2)> {
        self.hits
            .iter()
            .enumerate()
            .filter_map(|(index, hit)| match hit.coverage {
                ValidatedHitCoverage::Fill { shape, rule } => {
                    Some((trusted_hit_ordinal(index), shape, rule))
                }
                ValidatedHitCoverage::RoundStroke { .. } => None,
            })
            .collect()
    }

    pub(in crate::input_validation) fn validated_stroke_hit_facts(&self) -> Vec<(u32, u32, i64)> {
        self.hits
            .iter()
            .enumerate()
            .filter_map(|(index, hit)| match hit.coverage {
                ValidatedHitCoverage::RoundStroke { shape, stroke } => {
                    Some((trusted_hit_ordinal(index), shape, stroke.width().raw()))
                }
                ValidatedHitCoverage::Fill { .. } => None,
            })
            .collect()
    }

    pub(in crate::input_validation) fn validated_paint_facts(
        &self,
    ) -> Vec<(u32, u32, u32, crate::paint::SpatialPaintKindV2)> {
        self.paints.validated_paint_facts()
    }

    pub(in crate::input_validation) fn validated_fill_paint_facts(
        &self,
    ) -> Vec<(u32, u32, SpatialFillRuleV2, u32, u8, Option<u32>)> {
        self.paints.validated_fill_paint_facts()
    }

    pub(in crate::input_validation) fn validated_stroke_paint_facts(
        &self,
    ) -> Vec<(u32, u32, i64, u32, u8, Option<u32>)> {
        self.paints.validated_stroke_paint_facts()
    }

    pub(in crate::input_validation) fn validated_image_paint_facts(
        &self,
    ) -> Vec<(
        u32,
        u32,
        crate::image::SpatialImageSourceRectV2,
        crate::image::SpatialImageDestinationRectV2,
        u8,
        Option<u32>,
    )> {
        self.paints.validated_image_paint_facts()
    }

    pub(in crate::input_validation) fn validated_image_paint_bytes(
        &self,
        paint: u32,
    ) -> Option<&[u8]> {
        self.paints.validated_image_paint_bytes(paint)
    }

    pub(in crate::input_validation) fn validated_clip_facts(
        &self,
    ) -> Vec<(u32, u32, Option<u32>, u32, SpatialFillRuleV2, usize)> {
        self.paints.validated_clip_facts()
    }

    pub(in crate::input_validation) fn validated_image_facts(
        &self,
    ) -> Vec<(u32, u32, u32, u32, Vec<u8>)> {
        self.paints.validated_image_facts()
    }

    pub(in crate::input_validation) fn accepted_pixel_total(&self) -> u128 {
        self.paints.accepted_pixel_total()
    }

    pub(in crate::input_validation) fn prepared_brush_facts(
        &self,
    ) -> Vec<(u32, crate::brush::SpatialBrushKindV2, usize)> {
        self.paints.prepared_brush_facts()
    }

    pub(in crate::input_validation) fn gradient_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.paints.gradient_range_facts()
    }

    pub(in crate::input_validation) fn validated_shape_facts(
        &self,
    ) -> Vec<(u32, crate::shape::SpatialShapeKindV2, usize)> {
        self.paints.validated_shape_facts()
    }

    pub(in crate::input_validation) fn polygon_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.paints.polygon_range_facts()
    }

    pub(in crate::input_validation) fn validated_path_facts(&self) -> Vec<(u32, usize, usize)> {
        self.paints.validated_path_facts()
    }

    pub(in crate::input_validation) fn subpath_total(&self) -> usize {
        self.paints.subpath_total()
    }

    pub(in crate::input_validation) fn path_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.paints.path_range_facts()
    }

    pub(in crate::input_validation) fn prepared_island_facts(&self) -> Vec<(u32, Vec<u32>)> {
        self.paints.prepared_island_facts()
    }
}
