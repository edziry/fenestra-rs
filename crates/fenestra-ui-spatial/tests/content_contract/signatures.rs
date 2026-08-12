use fenestra_ui_spatial::prototype::{
    SpatialClipKeyV2, SpatialCoverageV2, SpatialFillRuleV2, SpatialNodeKeyV2, SpatialScalarV2,
    SpatialShapeKeyV2,
};

use crate::*;

type ResourceInputConstructor<'a> = fn(
    &'a [SpatialGradientStopV2],
    &'a [SpatialBrushV2],
    &'a [SpatialImageV2],
) -> SpatialResourceInputV2<'a>;

type ItemInputConstructor<'a> = fn(
    &'a [SpatialPaintV2],
    &'a [SpatialHitV2],
    &'a [SpatialSemanticGeometryV2],
) -> SpatialItemInputV2<'a>;

#[test]
fn raw_content_function_signatures_are_exact() {
    let _: fn(u32) -> SpatialBrushKeyV2 = SpatialBrushKeyV2::new;
    let _: fn(SpatialBrushKeyV2) -> u32 = SpatialBrushKeyV2::get;
    let _: fn(u32) -> SpatialImageKeyV2 = SpatialImageKeyV2::new;
    let _: fn(SpatialImageKeyV2) -> u32 = SpatialImageKeyV2::get;

    let _: fn(u8, u8, u8, u8) -> SpatialRgba8V2 = SpatialRgba8V2::new;
    let _: fn(SpatialRgba8V2) -> u8 = SpatialRgba8V2::r;
    let _: fn(SpatialRgba8V2) -> u8 = SpatialRgba8V2::g;
    let _: fn(SpatialRgba8V2) -> u8 = SpatialRgba8V2::b;
    let _: fn(SpatialRgba8V2) -> u8 = SpatialRgba8V2::a;

    let _: fn(u16, SpatialRgba8V2) -> SpatialGradientStopV2 = SpatialGradientStopV2::new;
    let _: fn(SpatialGradientStopV2) -> u16 = SpatialGradientStopV2::offset;
    let _: fn(SpatialGradientStopV2) -> SpatialRgba8V2 = SpatialGradientStopV2::color;
    let _: fn(SpatialBrushKeyV2, SpatialBrushContentV2) -> SpatialBrushV2 = SpatialBrushV2::new;
    let _: fn(SpatialBrushV2) -> SpatialBrushKeyV2 = SpatialBrushV2::key;
    let _: fn(SpatialBrushV2) -> SpatialBrushContentV2 = SpatialBrushV2::content;

    let _: fn(SpatialImageKeyV2, u32, u32, u32, Box<[u8]>) -> SpatialImageV2 = SpatialImageV2::new;
    let _: fn(&SpatialImageV2) -> SpatialImageKeyV2 = SpatialImageV2::key;
    let _: fn(&SpatialImageV2) -> u32 = SpatialImageV2::width;
    let _: fn(&SpatialImageV2) -> u32 = SpatialImageV2::height;
    let _: fn(&SpatialImageV2) -> u32 = SpatialImageV2::stride;
    let _: for<'a> fn(&'a SpatialImageV2) -> &'a [u8] = SpatialImageV2::bytes;

    let _: fn(u32, u32, u32, u32) -> SpatialImageSourceRectV2 = SpatialImageSourceRectV2::new;
    let _: fn(SpatialImageSourceRectV2) -> u32 = SpatialImageSourceRectV2::x;
    let _: fn(SpatialImageSourceRectV2) -> u32 = SpatialImageSourceRectV2::y;
    let _: fn(SpatialImageSourceRectV2) -> u32 = SpatialImageSourceRectV2::width;
    let _: fn(SpatialImageSourceRectV2) -> u32 = SpatialImageSourceRectV2::height;

    let _: fn(
        SpatialScalarV2,
        SpatialScalarV2,
        SpatialScalarV2,
        SpatialScalarV2,
    ) -> SpatialImageDestinationRectV2 = SpatialImageDestinationRectV2::new;
    let _: fn(SpatialImageDestinationRectV2) -> SpatialScalarV2 = SpatialImageDestinationRectV2::x;
    let _: fn(SpatialImageDestinationRectV2) -> SpatialScalarV2 = SpatialImageDestinationRectV2::y;
    let _: fn(SpatialImageDestinationRectV2) -> SpatialScalarV2 =
        SpatialImageDestinationRectV2::width;
    let _: fn(SpatialImageDestinationRectV2) -> SpatialScalarV2 =
        SpatialImageDestinationRectV2::height;

    let _: fn(SpatialNodeKeyV2, u32, SpatialPaintContentV2) -> SpatialPaintV2 = SpatialPaintV2::new;
    let _: fn(SpatialPaintV2) -> SpatialNodeKeyV2 = SpatialPaintV2::owner;
    let _: fn(SpatialPaintV2) -> u32 = SpatialPaintV2::item_ordinal;
    let _: fn(SpatialPaintV2) -> SpatialPaintContentV2 = SpatialPaintV2::content;

    let _: fn(
        SpatialNodeKeyV2,
        u32,
        SpatialCoverageV2,
        Option<SpatialClipKeyV2>,
        SpatialInputPolicyV2,
    ) -> SpatialHitV2 = SpatialHitV2::new;
    let _: fn(SpatialHitV2) -> SpatialNodeKeyV2 = SpatialHitV2::owner;
    let _: fn(SpatialHitV2) -> u32 = SpatialHitV2::item_ordinal;
    let _: fn(SpatialHitV2) -> SpatialCoverageV2 = SpatialHitV2::coverage;
    let _: fn(SpatialHitV2) -> Option<SpatialClipKeyV2> = SpatialHitV2::clip;
    let _: fn(SpatialHitV2) -> SpatialInputPolicyV2 = SpatialHitV2::input_policy;

    let _: fn(
        SpatialNodeKeyV2,
        u32,
        SpatialShapeKeyV2,
        SpatialFillRuleV2,
        Option<SpatialClipKeyV2>,
    ) -> SpatialSemanticGeometryV2 = SpatialSemanticGeometryV2::new;
    let _: fn(SpatialSemanticGeometryV2) -> SpatialNodeKeyV2 = SpatialSemanticGeometryV2::owner;
    let _: fn(SpatialSemanticGeometryV2) -> u32 = SpatialSemanticGeometryV2::item_ordinal;
    let _: fn(SpatialSemanticGeometryV2) -> SpatialShapeKeyV2 = SpatialSemanticGeometryV2::shape;
    let _: fn(SpatialSemanticGeometryV2) -> SpatialFillRuleV2 =
        SpatialSemanticGeometryV2::fill_rule;
    let _: fn(SpatialSemanticGeometryV2) -> Option<SpatialClipKeyV2> =
        SpatialSemanticGeometryV2::clip;

    assert_input_signatures(&());
}

fn assert_input_signatures<'a>(_: &'a ()) {
    let _: ResourceInputConstructor<'a> = SpatialResourceInputV2::new;
    let _: fn(SpatialResourceInputV2<'a>) -> &'a [SpatialGradientStopV2] =
        SpatialResourceInputV2::gradient_stops;
    let _: fn(SpatialResourceInputV2<'a>) -> &'a [SpatialBrushV2] = SpatialResourceInputV2::brushes;
    let _: fn(SpatialResourceInputV2<'a>) -> &'a [SpatialImageV2] = SpatialResourceInputV2::images;

    let _: ItemInputConstructor<'a> = SpatialItemInputV2::new;
    let _: fn(SpatialItemInputV2<'a>) -> &'a [SpatialPaintV2] = SpatialItemInputV2::paint_items;
    let _: fn(SpatialItemInputV2<'a>) -> &'a [SpatialHitV2] = SpatialItemInputV2::hit_items;
    let _: fn(SpatialItemInputV2<'a>) -> &'a [SpatialSemanticGeometryV2] =
        SpatialItemInputV2::semantic_items;
}
