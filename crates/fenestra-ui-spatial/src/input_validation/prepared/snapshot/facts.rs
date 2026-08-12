use std::sync::Arc;

use super::SpatialResolvedSnapshotV2;
use crate::aabb::SpatialAabbV2;
use crate::brush::{SpatialBrushKindV2, SpatialRgba8V2};
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::{SpatialCoverageKindV2, SpatialFillRuleV2};
use crate::image::{SpatialImageDestinationRectV2, SpatialImageSourceRectV2};
use crate::model::SpatialPointV2;
use crate::owned_input::SpatialOwnedInputV2;
use crate::paint::SpatialPaintKindV2;
use crate::shape::SpatialShapeKindV2;

type FlattenedPathFact = (u32, usize, Vec<(i64, i64)>, Vec<(usize, usize, bool)>);
type ShapePlanFact = (
    u32,
    u32,
    SpatialShapeKindV2,
    Option<u32>,
    usize,
    SpatialAabbV2,
    SpatialAabbV2,
);
type GradientFact = (SpatialPointV2, SpatialPointV2, Vec<(u16, SpatialRgba8V2)>);
type ClipFact = (u32, u32, Option<u32>, u32, SpatialFillRuleV2, usize);
type HitFact = (
    u32,
    u32,
    u32,
    SpatialCoverageKindV2,
    SpatialInputPolicyV2,
    Option<u32>,
);

impl SpatialResolvedSnapshotV2 {
    pub(in crate::input_validation) fn source_arc(&self) -> &Arc<SpatialOwnedInputV2> {
        self.prepared.source_arc()
    }

    pub(in crate::input_validation) fn effective_clip_identity(
        &self,
    ) -> (*const SpatialAabbV2, usize) {
        self.prepared.effective_clip_identity()
    }

    pub(in crate::input_validation) fn path_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.prepared.path_range_facts()
    }

    pub(in crate::input_validation) fn validated_path_facts(&self) -> Vec<(u32, usize, usize)> {
        self.prepared.validated_path_facts()
    }

    pub(in crate::input_validation) fn flattened_path_facts(&self) -> Vec<FlattenedPathFact> {
        self.prepared.flattened_path_facts()
    }

    pub(in crate::input_validation) fn polygon_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.prepared.polygon_range_facts()
    }

    pub(in crate::input_validation) fn shape_plan_facts(&self) -> Vec<ShapePlanFact> {
        self.prepared.shape_plan_facts()
    }

    pub(in crate::input_validation) fn gradient_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.prepared.gradient_range_facts()
    }

    pub(in crate::input_validation) fn prepared_brush_facts(
        &self,
    ) -> Vec<(u32, SpatialBrushKindV2, usize)> {
        self.prepared.prepared_brush_facts()
    }

    pub(in crate::input_validation) fn prepared_solid_color(&self, brush: u32) -> SpatialRgba8V2 {
        self.prepared.prepared_solid_color(brush)
    }

    pub(in crate::input_validation) fn prepared_gradient_facts(&self, brush: u32) -> GradientFact {
        self.prepared.prepared_gradient_facts(brush)
    }

    pub(in crate::input_validation) fn image_plan_facts(&self) -> Vec<(u32, u32, u32, u32)> {
        self.prepared.image_plan_facts()
    }

    pub(in crate::input_validation) fn validated_clip_facts(&self) -> Vec<ClipFact> {
        self.prepared.validated_clip_facts()
    }

    pub(in crate::input_validation) fn validated_paint_facts(
        &self,
    ) -> Vec<(u32, u32, u32, SpatialPaintKindV2)> {
        self.prepared.validated_paint_facts()
    }

    pub(in crate::input_validation) fn validated_fill_paint_facts(
        &self,
    ) -> Vec<(u32, u32, SpatialFillRuleV2, u32, u8, Option<u32>)> {
        self.prepared.validated_fill_paint_facts()
    }

    pub(in crate::input_validation) fn validated_stroke_paint_facts(
        &self,
    ) -> Vec<(u32, u32, i64, u32, u8, Option<u32>)> {
        self.prepared.validated_stroke_paint_facts()
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
        self.prepared.validated_image_paint_facts()
    }

    pub(in crate::input_validation) fn finalized_image_paint_bytes(
        &self,
        paint: u32,
    ) -> Option<&[u8]> {
        self.prepared.finalized_image_paint_bytes(paint)
    }

    pub(in crate::input_validation) fn validated_hit_facts(&self) -> Vec<HitFact> {
        self.prepared.validated_hit_facts()
    }

    pub(in crate::input_validation) fn validated_fill_hit_facts(
        &self,
    ) -> Vec<(u32, u32, SpatialFillRuleV2)> {
        self.prepared.validated_fill_hit_facts()
    }

    pub(in crate::input_validation) fn validated_stroke_hit_facts(&self) -> Vec<(u32, u32, i64)> {
        self.prepared.validated_stroke_hit_facts()
    }

    pub(in crate::input_validation) fn validated_semantic_facts(
        &self,
    ) -> Vec<(u32, u32, u32, u32, SpatialFillRuleV2, Option<u32>)> {
        self.prepared.validated_semantic_facts()
    }
}
