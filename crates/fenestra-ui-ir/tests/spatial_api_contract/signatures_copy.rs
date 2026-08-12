use crate::*;
use fenestra_ui_ir::prototype::{InputPolicy, PropertyId, SourceSpan};

type Integer = SpatialFieldV2<SpatialBindingV2<i32>>;
type Fixed = SpatialFieldV2<SpatialBindingV2<i64>>;
type Color = SpatialFieldV2<SpatialBindingV2<[u8; 4]>>;
type Policy = SpatialFieldV2<SpatialBindingV2<InputPolicy>>;

#[test]
fn version_symbol_field_and_binding_signatures_are_exact() {
    let _: fn(u32) -> SpatialFormatVersion = SpatialFormatVersion::new;
    let _: fn(SpatialFormatVersion) -> u32 = SpatialFormatVersion::get;
    let _: fn(u32) -> SpatialNodeSymbolV2 = SpatialNodeSymbolV2::new;
    let _: fn(SpatialNodeSymbolV2) -> u32 = SpatialNodeSymbolV2::get;
    let _: fn(u32) -> SpatialShapeSymbolV2 = SpatialShapeSymbolV2::new;
    let _: fn(SpatialShapeSymbolV2) -> u32 = SpatialShapeSymbolV2::get;
    let _: fn(u32) -> SpatialBrushSymbolV2 = SpatialBrushSymbolV2::new;
    let _: fn(SpatialBrushSymbolV2) -> u32 = SpatialBrushSymbolV2::get;
    let _: fn(u32) -> SpatialClipSymbolV2 = SpatialClipSymbolV2::new;
    let _: fn(SpatialClipSymbolV2) -> u32 = SpatialClipSymbolV2::get;
    let _: fn(u32) -> SpatialImageSymbolV2 = SpatialImageSymbolV2::new;
    let _: fn(SpatialImageSymbolV2) -> u32 = SpatialImageSymbolV2::get;

    let _: fn(i32, SourceSpan) -> SpatialFieldV2<i32> = SpatialFieldV2::new;
    let _: for<'a> fn(&'a SpatialFieldV2<i32>) -> &'a i32 = SpatialFieldV2::value;
    let _: fn(&SpatialFieldV2<i32>) -> SourceSpan = SpatialFieldV2::span;

    let _: SpatialBindingV2<i32> = SpatialBindingV2::Literal(0);
    let _: SpatialBindingV2<i32> = SpatialBindingV2::Property(PropertyId::new(0));
}

#[test]
fn point_padding_dimension_and_transform_signatures_are_exact() {
    let _: fn(Fixed, Fixed) -> SpatialPointRecipeV2 = SpatialPointRecipeV2::new;
    let _: fn(SpatialPointRecipeV2) -> Fixed = SpatialPointRecipeV2::x;
    let _: fn(SpatialPointRecipeV2) -> Fixed = SpatialPointRecipeV2::y;

    let _: fn(Integer, Integer, Integer, Integer) -> SpatialPaddingRecipeV2 =
        SpatialPaddingRecipeV2::new;
    let _: fn(SpatialPaddingRecipeV2) -> Integer = SpatialPaddingRecipeV2::left;
    let _: fn(SpatialPaddingRecipeV2) -> Integer = SpatialPaddingRecipeV2::right;
    let _: fn(SpatialPaddingRecipeV2) -> Integer = SpatialPaddingRecipeV2::top;
    let _: fn(SpatialPaddingRecipeV2) -> Integer = SpatialPaddingRecipeV2::bottom;

    let _: fn(Integer, Integer, Integer) -> SpatialDimensionRecipeV2 =
        SpatialDimensionRecipeV2::new;
    let _: fn(SpatialDimensionRecipeV2) -> Integer = SpatialDimensionRecipeV2::minimum;
    let _: fn(SpatialDimensionRecipeV2) -> Integer = SpatialDimensionRecipeV2::preferred;
    let _: fn(SpatialDimensionRecipeV2) -> Integer = SpatialDimensionRecipeV2::maximum;

    let _: fn(
        Fixed,
        Fixed,
        Fixed,
        Fixed,
        Fixed,
        Fixed,
        SpatialPointRecipeV2,
    ) -> SpatialTransformRecipeV2 = SpatialTransformRecipeV2::new;
    let _: fn(SpatialTransformRecipeV2) -> Fixed = SpatialTransformRecipeV2::a;
    let _: fn(SpatialTransformRecipeV2) -> Fixed = SpatialTransformRecipeV2::b;
    let _: fn(SpatialTransformRecipeV2) -> Fixed = SpatialTransformRecipeV2::c;
    let _: fn(SpatialTransformRecipeV2) -> Fixed = SpatialTransformRecipeV2::d;
    let _: fn(SpatialTransformRecipeV2) -> Fixed = SpatialTransformRecipeV2::tx;
    let _: fn(SpatialTransformRecipeV2) -> Fixed = SpatialTransformRecipeV2::ty;
    let _: fn(SpatialTransformRecipeV2) -> SpatialPointRecipeV2 = SpatialTransformRecipeV2::origin;
}

#[test]
fn container_and_placement_signatures_are_exact() {
    let _: fn(
        SpatialAxisV2,
        SpatialFieldV2<i32>,
        SpatialFieldV2<i32>,
        SpatialFieldV2<i32>,
        SpatialFieldV2<i32>,
        SpatialFieldV2<i32>,
        SourceSpan,
    ) -> SpatialViewportContainerV2 = SpatialViewportContainerV2::new;
    let _: fn(SpatialViewportContainerV2) -> SpatialAxisV2 = SpatialViewportContainerV2::axis;
    let _: fn(SpatialViewportContainerV2) -> SpatialFieldV2<i32> = SpatialViewportContainerV2::left;
    let _: fn(SpatialViewportContainerV2) -> SpatialFieldV2<i32> =
        SpatialViewportContainerV2::right;
    let _: fn(SpatialViewportContainerV2) -> SpatialFieldV2<i32> = SpatialViewportContainerV2::top;
    let _: fn(SpatialViewportContainerV2) -> SpatialFieldV2<i32> =
        SpatialViewportContainerV2::bottom;
    let _: fn(SpatialViewportContainerV2) -> SpatialFieldV2<i32> = SpatialViewportContainerV2::gap;
    let _: fn(SpatialViewportContainerV2) -> SourceSpan = SpatialViewportContainerV2::span;

    let _: fn(SpatialAxisV2, SpatialPaddingRecipeV2, Integer) -> SpatialContainerRecipeV2 =
        SpatialContainerRecipeV2::new;
    let _: fn(SpatialContainerRecipeV2) -> SpatialAxisV2 = SpatialContainerRecipeV2::axis;
    let _: fn(SpatialContainerRecipeV2) -> SpatialPaddingRecipeV2 =
        SpatialContainerRecipeV2::padding;
    let _: fn(SpatialContainerRecipeV2) -> Integer = SpatialContainerRecipeV2::gap;

    let _: fn(
        SpatialDimensionRecipeV2,
        SpatialDimensionRecipeV2,
        SpatialTransformRecipeV2,
    ) -> SpatialLayoutPlacementRecipeV2 = SpatialLayoutPlacementRecipeV2::new;
    let _: fn(SpatialLayoutPlacementRecipeV2) -> SpatialDimensionRecipeV2 =
        SpatialLayoutPlacementRecipeV2::width;
    let _: fn(SpatialLayoutPlacementRecipeV2) -> SpatialDimensionRecipeV2 =
        SpatialLayoutPlacementRecipeV2::height;
    let _: fn(SpatialLayoutPlacementRecipeV2) -> SpatialTransformRecipeV2 =
        SpatialLayoutPlacementRecipeV2::transform;

    assert_free_placement_signatures();
}

#[test]
fn copy_content_record_signatures_are_exact() {
    let _: fn(
        SpatialFieldV2<SpatialNodeSymbolV2>,
        SpatialFieldV2<SpatialClipSymbolV2>,
    ) -> SpatialClipAddressV2 = SpatialClipAddressV2::new;
    let _: fn(SpatialClipAddressV2) -> SpatialFieldV2<SpatialNodeSymbolV2> =
        SpatialClipAddressV2::owner;
    let _: fn(SpatialClipAddressV2) -> SpatialFieldV2<SpatialClipSymbolV2> =
        SpatialClipAddressV2::clip;

    let _: fn(SpatialPointRecipeV2, SourceSpan) -> SpatialPolygonPointV2 =
        SpatialPolygonPointV2::new;
    let _: fn(SpatialPolygonPointV2) -> SpatialPointRecipeV2 = SpatialPolygonPointV2::point;
    let _: fn(SpatialPolygonPointV2) -> SourceSpan = SpatialPolygonPointV2::span;

    let _: fn(SpatialFieldV2<u16>, Color, SourceSpan) -> SpatialGradientStopV2 =
        SpatialGradientStopV2::new;
    let _: fn(SpatialGradientStopV2) -> SpatialFieldV2<u16> = SpatialGradientStopV2::offset;
    let _: fn(SpatialGradientStopV2) -> Color = SpatialGradientStopV2::color;
    let _: fn(SpatialGradientStopV2) -> SourceSpan = SpatialGradientStopV2::span;

    assert_clip_hit_semantic_signatures();
}

fn assert_free_placement_signatures() {
    let _: fn(
        Integer,
        Integer,
        [SpatialAnchorComponentV2; 2],
        SpatialAnchorTargetRecipeV2,
        [SpatialAnchorComponentV2; 2],
        SpatialPointRecipeV2,
        SpatialTransformRecipeV2,
    ) -> SpatialFreePlacementRecipeV2 = SpatialFreePlacementRecipeV2::new;
    let _: fn(SpatialFreePlacementRecipeV2) -> Integer = SpatialFreePlacementRecipeV2::width;
    let _: fn(SpatialFreePlacementRecipeV2) -> Integer = SpatialFreePlacementRecipeV2::height;
    let _: fn(SpatialFreePlacementRecipeV2) -> [SpatialAnchorComponentV2; 2] =
        SpatialFreePlacementRecipeV2::self_anchor;
    let _: fn(SpatialFreePlacementRecipeV2) -> SpatialAnchorTargetRecipeV2 =
        SpatialFreePlacementRecipeV2::target;
    let _: fn(SpatialFreePlacementRecipeV2) -> [SpatialAnchorComponentV2; 2] =
        SpatialFreePlacementRecipeV2::target_anchor;
    let _: fn(SpatialFreePlacementRecipeV2) -> SpatialPointRecipeV2 =
        SpatialFreePlacementRecipeV2::offset;
    let _: fn(SpatialFreePlacementRecipeV2) -> SpatialTransformRecipeV2 =
        SpatialFreePlacementRecipeV2::transform;
}

fn assert_clip_hit_semantic_signatures() {
    let _: fn(
        SpatialFieldV2<SpatialClipSymbolV2>,
        Option<SpatialClipAddressV2>,
        SpatialFieldV2<SpatialShapeSymbolV2>,
        SpatialFillRuleV2,
        SourceSpan,
    ) -> SpatialClipDeclarationV2 = SpatialClipDeclarationV2::new;
    let _: fn(SpatialClipDeclarationV2) -> SpatialFieldV2<SpatialClipSymbolV2> =
        SpatialClipDeclarationV2::symbol;
    let _: fn(SpatialClipDeclarationV2) -> Option<SpatialClipAddressV2> =
        SpatialClipDeclarationV2::parent;
    let _: fn(SpatialClipDeclarationV2) -> SpatialFieldV2<SpatialShapeSymbolV2> =
        SpatialClipDeclarationV2::shape;
    let _: fn(SpatialClipDeclarationV2) -> SpatialFillRuleV2 = SpatialClipDeclarationV2::fill_rule;
    let _: fn(SpatialClipDeclarationV2) -> SourceSpan = SpatialClipDeclarationV2::span;

    let _: fn(
        SpatialCoverageRecipeV2,
        Option<SpatialClipAddressV2>,
        Policy,
        SourceSpan,
    ) -> SpatialHitRecipeV2 = SpatialHitRecipeV2::new;
    let _: fn(SpatialHitRecipeV2) -> SpatialCoverageRecipeV2 = SpatialHitRecipeV2::coverage;
    let _: fn(SpatialHitRecipeV2) -> Option<SpatialClipAddressV2> = SpatialHitRecipeV2::clip;
    let _: fn(SpatialHitRecipeV2) -> Policy = SpatialHitRecipeV2::input_policy;
    let _: fn(SpatialHitRecipeV2) -> SourceSpan = SpatialHitRecipeV2::span;

    let _: fn(
        SpatialFieldV2<SpatialShapeSymbolV2>,
        SpatialFillRuleV2,
        Option<SpatialClipAddressV2>,
        SourceSpan,
    ) -> SpatialSemanticRecipeV2 = SpatialSemanticRecipeV2::new;
    let _: fn(SpatialSemanticRecipeV2) -> SpatialFieldV2<SpatialShapeSymbolV2> =
        SpatialSemanticRecipeV2::shape;
    let _: fn(SpatialSemanticRecipeV2) -> SpatialFillRuleV2 = SpatialSemanticRecipeV2::fill_rule;
    let _: fn(SpatialSemanticRecipeV2) -> Option<SpatialClipAddressV2> =
        SpatialSemanticRecipeV2::clip;
    let _: fn(SpatialSemanticRecipeV2) -> SourceSpan = SpatialSemanticRecipeV2::span;
}
