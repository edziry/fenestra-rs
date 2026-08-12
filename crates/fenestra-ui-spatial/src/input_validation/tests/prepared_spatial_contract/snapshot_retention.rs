use super::support::{requested_limits, rich_engine, rich_owned};
use super::*;

#[test]
fn materialization_retains_every_hidden_resource_and_item_plan() {
    let prepared = prepare_spatial_v2(&rich_engine(), rich_owned(), requested_limits())
        .expect("rich owned input prepares successfully");
    let expected = RetainedFacts::capture(&prepared);
    let expected_bytes = identity(prepared.finalized_image_paint_bytes(1).unwrap());

    let snapshot = materialize_reference_spatial_v2(prepared);

    expected.assert_snapshot(&snapshot);
    assert_eq!(
        identity(snapshot.finalized_image_paint_bytes(1).unwrap()),
        expected_bytes
    );
}

type FillPaintFact = (
    u32,
    u32,
    crate::coverage::SpatialFillRuleV2,
    u32,
    u8,
    Option<u32>,
);
type StrokePaintFact = (u32, u32, i64, u32, u8, Option<u32>);
type FlattenedPathFact = (u32, usize, Vec<(i64, i64)>, Vec<(usize, usize, bool)>);
type ShapePlanFact = (
    u32,
    u32,
    crate::shape::SpatialShapeKindV2,
    Option<u32>,
    usize,
    crate::aabb::SpatialAabbV2,
    crate::aabb::SpatialAabbV2,
);
type ImagePaintFact = (
    u32,
    u32,
    crate::image::SpatialImageSourceRectV2,
    crate::image::SpatialImageDestinationRectV2,
    u8,
    Option<u32>,
);
type FillHitFact = (u32, u32, crate::coverage::SpatialFillRuleV2);
type StrokeHitFact = (u32, u32, i64);

pub(super) struct RetainedFacts {
    paths: Vec<(u32, u128, u128)>,
    path_k1: Vec<(u32, usize, usize)>,
    flattened: Vec<FlattenedPathFact>,
    polygons: Vec<(u32, u128, u128)>,
    shapes: Vec<ShapePlanFact>,
    gradients: Vec<(u32, u128, u128)>,
    brushes: Vec<(u32, crate::brush::SpatialBrushKindV2, usize)>,
    solid: crate::brush::SpatialRgba8V2,
    gradient_one: (
        crate::model::SpatialPointV2,
        crate::model::SpatialPointV2,
        Vec<(u16, crate::brush::SpatialRgba8V2)>,
    ),
    gradient_two: (
        crate::model::SpatialPointV2,
        crate::model::SpatialPointV2,
        Vec<(u16, crate::brush::SpatialRgba8V2)>,
    ),
    images: Vec<(u32, u32, u32, u32)>,
    clips: Vec<(
        u32,
        u32,
        Option<u32>,
        u32,
        crate::coverage::SpatialFillRuleV2,
        usize,
    )>,
    paints: Vec<(u32, u32, u32, crate::paint::SpatialPaintKindV2)>,
    fill_paints: Vec<FillPaintFact>,
    stroke_paints: Vec<StrokePaintFact>,
    image_paints: Vec<ImagePaintFact>,
    hits: Vec<(
        u32,
        u32,
        u32,
        crate::coverage::SpatialCoverageKindV2,
        crate::content_item::SpatialInputPolicyV2,
        Option<u32>,
    )>,
    fill_hits: Vec<FillHitFact>,
    stroke_hits: Vec<StrokeHitFact>,
    semantics: Vec<(
        u32,
        u32,
        u32,
        u32,
        crate::coverage::SpatialFillRuleV2,
        Option<u32>,
    )>,
}

impl RetainedFacts {
    pub(super) fn capture(value: &PreparedSpatialV2) -> Self {
        Self {
            paths: value.path_range_facts(),
            path_k1: value.validated_path_facts(),
            flattened: value.flattened_path_facts(),
            polygons: value.polygon_range_facts(),
            shapes: value.shape_plan_facts(),
            gradients: value.gradient_range_facts(),
            brushes: value.prepared_brush_facts(),
            solid: value.prepared_solid_color(0),
            gradient_one: value.prepared_gradient_facts(1),
            gradient_two: value.prepared_gradient_facts(2),
            images: value.image_plan_facts(),
            clips: value.validated_clip_facts(),
            paints: value.validated_paint_facts(),
            fill_paints: value.validated_fill_paint_facts(),
            stroke_paints: value.validated_stroke_paint_facts(),
            image_paints: value.validated_image_paint_facts(),
            hits: value.validated_hit_facts(),
            fill_hits: value.validated_fill_hit_facts(),
            stroke_hits: value.validated_stroke_hit_facts(),
            semantics: value.validated_semantic_facts(),
        }
    }

    pub(super) fn assert_snapshot(self, value: &SpatialResolvedSnapshotV2) {
        assert_eq!(value.path_range_facts(), self.paths);
        assert_eq!(value.validated_path_facts(), self.path_k1);
        assert_eq!(value.flattened_path_facts(), self.flattened);
        assert_eq!(value.polygon_range_facts(), self.polygons);
        assert_eq!(value.shape_plan_facts(), self.shapes);
        assert_eq!(value.gradient_range_facts(), self.gradients);
        assert_eq!(value.prepared_brush_facts(), self.brushes);
        assert_eq!(value.prepared_solid_color(0), self.solid);
        assert_eq!(value.prepared_gradient_facts(1), self.gradient_one);
        assert_eq!(value.prepared_gradient_facts(2), self.gradient_two);
        assert_eq!(value.image_plan_facts(), self.images);
        assert_eq!(value.validated_clip_facts(), self.clips);
        assert_eq!(value.validated_paint_facts(), self.paints);
        assert_eq!(value.validated_fill_paint_facts(), self.fill_paints);
        assert_eq!(value.validated_stroke_paint_facts(), self.stroke_paints);
        assert_eq!(value.validated_image_paint_facts(), self.image_paints);
        assert_eq!(value.validated_hit_facts(), self.hits);
        assert_eq!(value.validated_fill_hit_facts(), self.fill_hits);
        assert_eq!(value.validated_stroke_hit_facts(), self.stroke_hits);
        assert_eq!(value.validated_semantic_facts(), self.semantics);
    }
}

fn identity<T>(slice: &[T]) -> (*const T, usize) {
    (slice.as_ptr(), slice.len())
}
