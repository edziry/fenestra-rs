use super::{
    ValidatedPaintContent, ValidatedPaintCoverage, ValidatedPaintItemsProof, trusted_paint_ordinal,
};
use crate::coverage::SpatialFillRuleV2;
use crate::paint::SpatialPaintKindV2;

impl<'a> ValidatedPaintItemsProof<'a> {
    pub(in crate::input_validation) fn validated_paint_facts(
        &self,
    ) -> Vec<(u32, u32, u32, SpatialPaintKindV2)> {
        self.paints
            .iter()
            .enumerate()
            .map(|(index, paint)| {
                let kind = match paint.content {
                    ValidatedPaintContent::Coverage { .. } => SpatialPaintKindV2::CoveragePaint,
                    ValidatedPaintContent::Image { .. } => SpatialPaintKindV2::ImagePaint,
                };
                (
                    trusted_paint_ordinal(index),
                    paint.owner,
                    paint.item_ordinal,
                    kind,
                )
            })
            .collect()
    }

    pub(in crate::input_validation) fn validated_fill_paint_facts(
        &self,
    ) -> Vec<(u32, u32, SpatialFillRuleV2, u32, u8, Option<u32>)> {
        self.paints
            .iter()
            .enumerate()
            .filter_map(|(index, paint)| match paint.content {
                ValidatedPaintContent::Coverage {
                    coverage: ValidatedPaintCoverage::Fill { shape, rule },
                    brush,
                    opacity,
                    clip,
                } => Some((
                    trusted_paint_ordinal(index),
                    shape,
                    rule,
                    brush,
                    opacity,
                    clip,
                )),
                _ => None,
            })
            .collect()
    }

    pub(in crate::input_validation) fn validated_stroke_paint_facts(
        &self,
    ) -> Vec<(u32, u32, i64, u32, u8, Option<u32>)> {
        self.paints
            .iter()
            .enumerate()
            .filter_map(|(index, paint)| match paint.content {
                ValidatedPaintContent::Coverage {
                    coverage: ValidatedPaintCoverage::RoundStroke { shape, stroke },
                    brush,
                    opacity,
                    clip,
                } => Some((
                    trusted_paint_ordinal(index),
                    shape,
                    stroke.width().raw(),
                    brush,
                    opacity,
                    clip,
                )),
                _ => None,
            })
            .collect()
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
        self.paints
            .iter()
            .enumerate()
            .filter_map(|(index, paint)| match &paint.content {
                ValidatedPaintContent::Image {
                    image,
                    preclip,
                    clip,
                } => {
                    let (source, destination, opacity, _) = preclip.facts();
                    Some((
                        trusted_paint_ordinal(index),
                        *image,
                        source,
                        destination,
                        opacity,
                        *clip,
                    ))
                }
                ValidatedPaintContent::Coverage { .. } => None,
            })
            .collect()
    }

    pub(in crate::input_validation) fn validated_image_paint_bytes(
        &self,
        paint: u32,
    ) -> Option<&[u8]> {
        match &self.paints.get(paint as usize)?.content {
            ValidatedPaintContent::Image { preclip, .. } => Some(preclip.facts().3),
            ValidatedPaintContent::Coverage { .. } => None,
        }
    }

    pub(in crate::input_validation) fn validated_clip_facts(
        &self,
    ) -> Vec<(u32, u32, Option<u32>, u32, SpatialFillRuleV2, usize)> {
        self.clips.validated_clip_facts()
    }

    pub(in crate::input_validation) fn validated_image_facts(
        &self,
    ) -> Vec<(u32, u32, u32, u32, Vec<u8>)> {
        self.clips.validated_image_facts()
    }

    pub(in crate::input_validation) fn accepted_pixel_total(&self) -> u128 {
        self.clips.accepted_pixel_total()
    }

    pub(in crate::input_validation) fn prepared_brush_facts(
        &self,
    ) -> Vec<(u32, crate::brush::SpatialBrushKindV2, usize)> {
        self.clips.prepared_brush_facts()
    }

    pub(in crate::input_validation) fn gradient_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.clips.gradient_range_facts()
    }

    pub(in crate::input_validation) fn validated_shape_facts(
        &self,
    ) -> Vec<(u32, crate::shape::SpatialShapeKindV2, usize)> {
        self.clips.validated_shape_facts()
    }

    pub(in crate::input_validation) fn polygon_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.clips.polygon_range_facts()
    }

    pub(in crate::input_validation) fn validated_path_facts(&self) -> Vec<(u32, usize, usize)> {
        self.clips.validated_path_facts()
    }

    pub(in crate::input_validation) fn subpath_total(&self) -> usize {
        self.clips.subpath_total()
    }

    pub(in crate::input_validation) fn path_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.clips.path_range_facts()
    }

    pub(in crate::input_validation) fn prepared_island_facts(&self) -> Vec<(u32, Vec<u32>)> {
        self.clips.prepared_island_facts()
    }
}
