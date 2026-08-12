use super::{LocalBoundsProof, PaintLocalBounds};
use crate::aabb::SpatialAabbV2;
use crate::brush::SpatialBrushKindV2;
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::{SpatialCoverageKindV2, SpatialFillRuleV2};
use crate::geometry_kernel::fill_bounds_k3;
use crate::image::{SpatialImageDestinationRectV2, SpatialImageSourceRectV2};
use crate::paint::SpatialPaintKindV2;
use crate::shape::SpatialShapeKindV2;

type FlattenedPathFact = (u32, usize, Vec<(i64, i64)>, Vec<(usize, usize, bool)>);

impl LocalBoundsProof<'_> {
    pub(in crate::input_validation) fn shape_local_bounds_facts(
        &self,
    ) -> Vec<(u32, SpatialAabbV2, SpatialAabbV2)> {
        self.shapes
            .iter()
            .enumerate()
            .map(|(index, bounds)| {
                (
                    trusted_ordinal(index),
                    bounds.base_bounds(),
                    fill_bounds_k3(bounds),
                )
            })
            .collect()
    }

    pub(in crate::input_validation) fn paint_local_bounds_facts(
        &self,
    ) -> Vec<(u32, SpatialAabbV2)> {
        self.paints
            .iter()
            .enumerate()
            .map(|(index, bounds)| (trusted_ordinal(index), bounds.local_bounds()))
            .collect()
    }

    pub(in crate::input_validation) fn hit_local_bounds_facts(&self) -> Vec<(u32, SpatialAabbV2)> {
        self.hits
            .iter()
            .copied()
            .enumerate()
            .map(|(index, bounds)| (trusted_ordinal(index), bounds))
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
        self.paints
            .iter()
            .enumerate()
            .filter_map(|(index, bounds)| match bounds {
                PaintLocalBounds::Coverage(_) => None,
                PaintLocalBounds::Image(image) => Some((
                    trusted_ordinal(index),
                    image.source(),
                    image.destination(),
                    image.opacity(),
                    image.local_bounds(),
                )),
            })
            .collect()
    }

    pub(in crate::input_validation) fn finalized_image_paint_bytes(
        &self,
        paint: u32,
    ) -> Option<&[u8]> {
        match self.paints.get(usize::try_from(paint).ok()?)? {
            PaintLocalBounds::Coverage(_) => None,
            PaintLocalBounds::Image(image) => Some(image.image_bytes()),
        }
    }

    pub(in crate::input_validation) fn flattened_path_facts(&self) -> Vec<FlattenedPathFact> {
        self.flattened.flattened_path_facts()
    }

    pub(in crate::input_validation) fn accepted_flattened_segment_total(&self) -> u128 {
        self.flattened.accepted_flattened_segment_total()
    }

    pub(in crate::input_validation) fn validated_semantic_facts(
        &self,
    ) -> Vec<(u32, u32, u32, u32, SpatialFillRuleV2, Option<u32>)> {
        self.flattened.validated_semantic_facts()
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
        self.flattened.validated_hit_facts()
    }

    pub(in crate::input_validation) fn validated_paint_facts(
        &self,
    ) -> Vec<(u32, u32, u32, SpatialPaintKindV2)> {
        self.flattened.validated_paint_facts()
    }

    pub(in crate::input_validation) fn validated_clip_facts(
        &self,
    ) -> Vec<(u32, u32, Option<u32>, u32, SpatialFillRuleV2, usize)> {
        self.flattened.validated_clip_facts()
    }

    pub(in crate::input_validation) fn validated_image_facts(
        &self,
    ) -> Vec<(u32, u32, u32, u32, Vec<u8>)> {
        self.flattened.validated_image_facts()
    }

    pub(in crate::input_validation) fn accepted_pixel_total(&self) -> u128 {
        self.flattened.accepted_pixel_total()
    }

    pub(in crate::input_validation) fn prepared_brush_facts(
        &self,
    ) -> Vec<(u32, SpatialBrushKindV2, usize)> {
        self.flattened.prepared_brush_facts()
    }

    pub(in crate::input_validation) fn gradient_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.flattened.gradient_range_facts()
    }

    pub(in crate::input_validation) fn validated_shape_facts(
        &self,
    ) -> Vec<(u32, SpatialShapeKindV2, usize)> {
        self.flattened.validated_shape_facts()
    }

    pub(in crate::input_validation) fn polygon_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.flattened.polygon_range_facts()
    }

    pub(in crate::input_validation) fn validated_path_facts(&self) -> Vec<(u32, usize, usize)> {
        self.flattened.validated_path_facts()
    }

    pub(in crate::input_validation) fn subpath_total(&self) -> usize {
        self.flattened.subpath_total()
    }

    pub(in crate::input_validation) fn path_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.flattened.path_range_facts()
    }

    pub(in crate::input_validation) fn prepared_island_facts(&self) -> Vec<(u32, Vec<u32>)> {
        self.flattened.prepared_island_facts()
    }
}

fn trusted_ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("phase one validated the local-bound row capacity")
}
