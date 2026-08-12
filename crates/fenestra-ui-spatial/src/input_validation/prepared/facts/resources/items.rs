use super::super::super::PreparedSpatialV2;
use super::super::super::model::{PreparedCoverage, PreparedPaintContent};
use super::ordinal;
use crate::aabb::SpatialAabbV2;
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::{SpatialCoverageKindV2, SpatialFillRuleV2};
use crate::image::{SpatialImageDestinationRectV2, SpatialImageSourceRectV2};
use crate::paint::SpatialPaintKindV2;

impl PreparedSpatialV2 {
    pub(in crate::input_validation) fn validated_clip_facts(
        &self,
    ) -> Vec<(u32, u32, Option<u32>, u32, SpatialFillRuleV2, usize)> {
        self.state
            .clips
            .iter()
            .enumerate()
            .map(|(index, clip)| {
                (
                    ordinal(index),
                    clip.owner,
                    clip.parent,
                    clip.shape,
                    clip.fill_rule,
                    clip.depth,
                )
            })
            .collect()
    }

    pub(in crate::input_validation) fn validated_paint_facts(
        &self,
    ) -> Vec<(u32, u32, u32, SpatialPaintKindV2)> {
        self.state
            .paints
            .iter()
            .enumerate()
            .map(|(index, paint)| {
                let kind = match &paint.content {
                    PreparedPaintContent::Coverage { .. } => SpatialPaintKindV2::CoveragePaint,
                    PreparedPaintContent::Image { .. } => SpatialPaintKindV2::ImagePaint,
                };
                (ordinal(index), paint.owner, paint.item_ordinal, kind)
            })
            .collect()
    }

    pub(in crate::input_validation) fn validated_fill_paint_facts(
        &self,
    ) -> Vec<(u32, u32, SpatialFillRuleV2, u32, u8, Option<u32>)> {
        self.state
            .paints
            .iter()
            .enumerate()
            .filter_map(|(index, paint)| match &paint.content {
                PreparedPaintContent::Coverage {
                    coverage: PreparedCoverage::Fill { shape, rule },
                    brush,
                    opacity,
                    clip,
                } => Some((ordinal(index), *shape, *rule, *brush, *opacity, *clip)),
                _ => None,
            })
            .collect()
    }

    pub(in crate::input_validation) fn validated_stroke_paint_facts(
        &self,
    ) -> Vec<(u32, u32, i64, u32, u8, Option<u32>)> {
        self.state
            .paints
            .iter()
            .enumerate()
            .filter_map(|(index, paint)| match &paint.content {
                PreparedPaintContent::Coverage {
                    coverage: PreparedCoverage::RoundStroke { shape, width },
                    brush,
                    opacity,
                    clip,
                } => Some((ordinal(index), *shape, width.raw(), *brush, *opacity, *clip)),
                _ => None,
            })
            .collect()
    }

    pub(in crate::input_validation) fn validated_image_paint_facts(
        &self,
    ) -> Vec<(
        u32,
        u32,
        SpatialImageSourceRectV2,
        SpatialImageDestinationRectV2,
        u8,
        Option<u32>,
    )> {
        self.state
            .paints
            .iter()
            .enumerate()
            .filter_map(|(index, paint)| match &paint.content {
                PreparedPaintContent::Image {
                    image,
                    source,
                    destination,
                    opacity,
                    clip,
                } => Some((
                    ordinal(index),
                    *image,
                    *source,
                    *destination,
                    *opacity,
                    *clip,
                )),
                _ => None,
            })
            .collect()
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
        self.state
            .paints
            .iter()
            .enumerate()
            .filter_map(|(index, paint)| match &paint.content {
                PreparedPaintContent::Image {
                    source,
                    destination,
                    opacity,
                    ..
                } => Some((
                    ordinal(index),
                    *source,
                    *destination,
                    *opacity,
                    paint.local_bounds,
                )),
                _ => None,
            })
            .collect()
    }

    pub(in crate::input_validation) fn finalized_image_paint_bytes(
        &self,
        paint: u32,
    ) -> Option<&[u8]> {
        let PreparedPaintContent::Image { image, .. } =
            &self.state.paints.get(paint as usize)?.content
        else {
            return None;
        };
        self.source
            .as_input()
            .resources()
            .images()
            .get(*image as usize)
            .map(|image| image.bytes())
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
        self.state
            .hits
            .iter()
            .enumerate()
            .map(|(index, hit)| {
                let kind = match &hit.coverage {
                    PreparedCoverage::Fill { .. } => SpatialCoverageKindV2::Fill,
                    PreparedCoverage::RoundStroke { .. } => SpatialCoverageKindV2::RoundStroke,
                };
                (
                    ordinal(index),
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
        self.state
            .hits
            .iter()
            .enumerate()
            .filter_map(|(index, hit)| match &hit.coverage {
                PreparedCoverage::Fill { shape, rule } => Some((ordinal(index), *shape, *rule)),
                PreparedCoverage::RoundStroke { .. } => None,
            })
            .collect()
    }

    pub(in crate::input_validation) fn validated_stroke_hit_facts(&self) -> Vec<(u32, u32, i64)> {
        self.state
            .hits
            .iter()
            .enumerate()
            .filter_map(|(index, hit)| match &hit.coverage {
                PreparedCoverage::RoundStroke { shape, width } => {
                    Some((ordinal(index), *shape, width.raw()))
                }
                PreparedCoverage::Fill { .. } => None,
            })
            .collect()
    }

    pub(in crate::input_validation) fn validated_semantic_facts(
        &self,
    ) -> Vec<(u32, u32, u32, u32, SpatialFillRuleV2, Option<u32>)> {
        self.state
            .semantics
            .iter()
            .enumerate()
            .map(|(index, semantic)| {
                (
                    ordinal(index),
                    semantic.owner,
                    semantic.item_ordinal,
                    semantic.shape,
                    semantic.fill_rule,
                    semantic.clip,
                )
            })
            .collect()
    }
}
