use fenestra_ui_spatial::prototype::{
    Affine2V2, SpatialBrushKeyV2, SpatialClipKeyV2, SpatialImageKeyV2, SpatialNodeKeyV2,
    SpatialScalarV2, SpatialShapeKeyV2,
};

use crate::*;

type GeometryConstructor = fn(
    SpatialNodeKeyV2,
    SpatialScalarV2,
    SpatialScalarV2,
    SpatialScalarV2,
    SpatialScalarV2,
    Affine2V2,
    i128,
    SpatialOutputAabbV2,
) -> SpatialGeometryOutputRecordV2;

type ClipConstructor = fn(
    SpatialClipKeyV2,
    Affine2V2,
    i128,
    SpatialOutputAabbV2,
    SpatialNodeKeyV2,
    Option<SpatialClipKeyV2>,
    SpatialShapeKeyV2,
) -> SpatialClipOutputRecordV2;

type PaintConstructor = fn(
    u32,
    Affine2V2,
    i128,
    SpatialOutputAabbV2,
    SpatialNodeKeyV2,
    SpatialPaintOutputReferenceV2,
    Option<SpatialClipKeyV2>,
    u32,
    u32,
) -> SpatialPaintOutputRecordV2;

type HitConstructor = fn(
    u32,
    Affine2V2,
    i128,
    SpatialOutputAabbV2,
    SpatialNodeKeyV2,
    SpatialShapeKeyV2,
    Option<SpatialClipKeyV2>,
    u32,
    u32,
) -> SpatialHitOutputRecordV2;

type SemanticConstructor = fn(
    u32,
    Affine2V2,
    i128,
    SpatialOutputAabbV2,
    SpatialNodeKeyV2,
    SpatialShapeKeyV2,
    Option<SpatialClipKeyV2>,
    u32,
    u32,
) -> SpatialSemanticOutputRecordV2;

type OutputConstructor<'a> = fn(
    &'a [SpatialGeometryOutputRecordV2],
    &'a [SpatialClipOutputRecordV2],
    &'a [SpatialPaintOutputRecordV2],
    &'a [SpatialHitOutputRecordV2],
    &'a [SpatialSemanticOutputRecordV2],
) -> SpatialOutputV2<'a>;

#[test]
fn raw_output_function_signatures_are_exact() {
    let _: fn(
        bool,
        SpatialScalarV2,
        SpatialScalarV2,
        SpatialScalarV2,
        SpatialScalarV2,
    ) -> SpatialOutputAabbV2 = SpatialOutputAabbV2::new;
    let _: fn(SpatialOutputAabbV2) -> bool = SpatialOutputAabbV2::is_empty;
    let _: fn(SpatialOutputAabbV2) -> SpatialScalarV2 = SpatialOutputAabbV2::min_x;
    let _: fn(SpatialOutputAabbV2) -> SpatialScalarV2 = SpatialOutputAabbV2::min_y;
    let _: fn(SpatialOutputAabbV2) -> SpatialScalarV2 = SpatialOutputAabbV2::max_x;
    let _: fn(SpatialOutputAabbV2) -> SpatialScalarV2 = SpatialOutputAabbV2::max_y;

    let _: GeometryConstructor = SpatialGeometryOutputRecordV2::new;
    let _: fn(SpatialGeometryOutputRecordV2) -> SpatialNodeKeyV2 =
        SpatialGeometryOutputRecordV2::key;
    let _: fn(SpatialGeometryOutputRecordV2) -> SpatialScalarV2 =
        SpatialGeometryOutputRecordV2::base_x;
    let _: fn(SpatialGeometryOutputRecordV2) -> SpatialScalarV2 =
        SpatialGeometryOutputRecordV2::base_y;
    let _: fn(SpatialGeometryOutputRecordV2) -> SpatialScalarV2 =
        SpatialGeometryOutputRecordV2::base_width;
    let _: fn(SpatialGeometryOutputRecordV2) -> SpatialScalarV2 =
        SpatialGeometryOutputRecordV2::base_height;
    let _: fn(SpatialGeometryOutputRecordV2) -> Affine2V2 =
        SpatialGeometryOutputRecordV2::world_from_local;
    let _: fn(SpatialGeometryOutputRecordV2) -> i128 =
        SpatialGeometryOutputRecordV2::world_determinant;
    let _: fn(SpatialGeometryOutputRecordV2) -> SpatialOutputAabbV2 =
        SpatialGeometryOutputRecordV2::world_aabb;

    let _: ClipConstructor = SpatialClipOutputRecordV2::new;
    let _: fn(SpatialClipOutputRecordV2) -> SpatialClipKeyV2 = SpatialClipOutputRecordV2::key;
    let _: fn(SpatialClipOutputRecordV2) -> Affine2V2 = SpatialClipOutputRecordV2::world_from_local;
    let _: fn(SpatialClipOutputRecordV2) -> i128 = SpatialClipOutputRecordV2::world_determinant;
    let _: fn(SpatialClipOutputRecordV2) -> SpatialOutputAabbV2 =
        SpatialClipOutputRecordV2::primitive_world_aabb;
    let _: fn(SpatialClipOutputRecordV2) -> SpatialNodeKeyV2 = SpatialClipOutputRecordV2::owner;
    let _: fn(SpatialClipOutputRecordV2) -> Option<SpatialClipKeyV2> =
        SpatialClipOutputRecordV2::parent;
    let _: fn(SpatialClipOutputRecordV2) -> SpatialShapeKeyV2 = SpatialClipOutputRecordV2::shape;

    let _: PaintConstructor = SpatialPaintOutputRecordV2::new;
    let _: HitConstructor = SpatialHitOutputRecordV2::new;
    let _: SemanticConstructor = SpatialSemanticOutputRecordV2::new;
    assert_item_getter_signatures();
    assert_output_signatures(&());
}

fn assert_item_getter_signatures() {
    let _: fn(SpatialPaintOutputRecordV2) -> u32 = SpatialPaintOutputRecordV2::key;
    let _: fn(SpatialPaintOutputRecordV2) -> Affine2V2 =
        SpatialPaintOutputRecordV2::world_from_local;
    let _: fn(SpatialPaintOutputRecordV2) -> i128 = SpatialPaintOutputRecordV2::world_determinant;
    let _: fn(SpatialPaintOutputRecordV2) -> SpatialOutputAabbV2 =
        SpatialPaintOutputRecordV2::world_aabb;
    let _: fn(SpatialPaintOutputRecordV2) -> SpatialNodeKeyV2 = SpatialPaintOutputRecordV2::owner;
    let _: fn(SpatialPaintOutputRecordV2) -> SpatialPaintOutputReferenceV2 =
        SpatialPaintOutputRecordV2::reference;
    let _: fn(SpatialPaintOutputRecordV2) -> Option<SpatialClipKeyV2> =
        SpatialPaintOutputRecordV2::clip;
    let _: fn(SpatialPaintOutputRecordV2) -> u32 = SpatialPaintOutputRecordV2::stack_ordinal;
    let _: fn(SpatialPaintOutputRecordV2) -> u32 = SpatialPaintOutputRecordV2::item_ordinal;

    let _: fn(SpatialHitOutputRecordV2) -> u32 = SpatialHitOutputRecordV2::key;
    let _: fn(SpatialHitOutputRecordV2) -> Affine2V2 = SpatialHitOutputRecordV2::world_from_local;
    let _: fn(SpatialHitOutputRecordV2) -> i128 = SpatialHitOutputRecordV2::world_determinant;
    let _: fn(SpatialHitOutputRecordV2) -> SpatialOutputAabbV2 =
        SpatialHitOutputRecordV2::world_aabb;
    let _: fn(SpatialHitOutputRecordV2) -> SpatialNodeKeyV2 = SpatialHitOutputRecordV2::owner;
    let _: fn(SpatialHitOutputRecordV2) -> SpatialShapeKeyV2 = SpatialHitOutputRecordV2::shape;
    let _: fn(SpatialHitOutputRecordV2) -> Option<SpatialClipKeyV2> =
        SpatialHitOutputRecordV2::clip;
    let _: fn(SpatialHitOutputRecordV2) -> u32 = SpatialHitOutputRecordV2::stack_ordinal;
    let _: fn(SpatialHitOutputRecordV2) -> u32 = SpatialHitOutputRecordV2::item_ordinal;

    let _: fn(SpatialSemanticOutputRecordV2) -> u32 = SpatialSemanticOutputRecordV2::key;
    let _: fn(SpatialSemanticOutputRecordV2) -> Affine2V2 =
        SpatialSemanticOutputRecordV2::world_from_local;
    let _: fn(SpatialSemanticOutputRecordV2) -> i128 =
        SpatialSemanticOutputRecordV2::world_determinant;
    let _: fn(SpatialSemanticOutputRecordV2) -> SpatialOutputAabbV2 =
        SpatialSemanticOutputRecordV2::world_aabb;
    let _: fn(SpatialSemanticOutputRecordV2) -> SpatialNodeKeyV2 =
        SpatialSemanticOutputRecordV2::owner;
    let _: fn(SpatialSemanticOutputRecordV2) -> SpatialShapeKeyV2 =
        SpatialSemanticOutputRecordV2::shape;
    let _: fn(SpatialSemanticOutputRecordV2) -> Option<SpatialClipKeyV2> =
        SpatialSemanticOutputRecordV2::clip;
    let _: fn(SpatialSemanticOutputRecordV2) -> u32 = SpatialSemanticOutputRecordV2::stack_ordinal;
    let _: fn(SpatialSemanticOutputRecordV2) -> u32 = SpatialSemanticOutputRecordV2::item_ordinal;
}

fn assert_output_signatures<'a>(_: &'a ()) {
    let _: OutputConstructor<'a> = SpatialOutputV2::new;
    let _: fn(SpatialOutputV2<'a>) -> &'a [SpatialGeometryOutputRecordV2] =
        SpatialOutputV2::geometry;
    let _: fn(SpatialOutputV2<'a>) -> &'a [SpatialClipOutputRecordV2] = SpatialOutputV2::clips;
    let _: fn(SpatialOutputV2<'a>) -> &'a [SpatialPaintOutputRecordV2] = SpatialOutputV2::paints;
    let _: fn(SpatialOutputV2<'a>) -> &'a [SpatialHitOutputRecordV2] = SpatialOutputV2::hits;
    let _: fn(SpatialOutputV2<'a>) -> &'a [SpatialSemanticOutputRecordV2] =
        SpatialOutputV2::semantics;
}

#[allow(dead_code)]
fn paint_reference_field_types(value: SpatialPaintOutputReferenceV2) {
    match value {
        SpatialPaintOutputReferenceV2::Coverage { shape, brush } => {
            let _: SpatialShapeKeyV2 = shape;
            let _: SpatialBrushKeyV2 = brush;
        }
        SpatialPaintOutputReferenceV2::Image { image } => {
            let _: SpatialImageKeyV2 = image;
        }
    }
}
