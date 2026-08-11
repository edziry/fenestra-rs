use fenestra_ui_layout::prototype::{
    LayoutAxisV1, LayoutEngineErrorKindV1, LayoutOutputErrorKindV1, LayoutOutputFieldV1,
    LayoutPaddingV1,
};
use fenestra_ui_spatial::prototype::{
    SpatialArithmeticOperationV2, SpatialAxisV2, SpatialContainerV2, SpatialCoverageV2,
    SpatialDependencyErrorKindV2, SpatialErrorLocationV2, SpatialExtentV2, SpatialFillRuleV2,
    SpatialGeometryInputV2, SpatialGradientStopV2, SpatialHitV2, SpatialImageKeyV2, SpatialImageV2,
    SpatialInputErrorKindV2, SpatialInputPolicyV2, SpatialItemInputV2, SpatialLimitKindV2,
    SpatialNodeKeyV2, SpatialNodeV2, SpatialPlacementV2, SpatialPointV2, SpatialResourceInputV2,
    SpatialRgba8V2, SpatialScalarV2, SpatialShapeKeyV2, SpatialTopologyInputV2,
    SpatialTransformErrorKindV2, SpatialViewportV2,
};
use fenestra_ui_spatial::prototype::{
    SpatialBrushFieldV2, SpatialClipErrorV2, SpatialClipFieldV2, SpatialColorChannelV2,
    SpatialContentErrorKindV2, SpatialContentReferenceV2, SpatialGradientErrorV2,
    SpatialGradientStopFieldV2, SpatialHitFieldV2, SpatialImageErrorV2, SpatialImageFieldV2,
    SpatialInputV2, SpatialKeyedContentTableV2, SpatialLayoutErrorKindV2,
    SpatialOrderedItemTableV2, SpatialOutputErrorKindV2, SpatialOutputFieldV2,
    SpatialOutputTableV2, SpatialPaintFieldV2, SpatialPathFieldV2, SpatialPathGrammarErrorV2,
    SpatialPathVerbFieldV2, SpatialPayloadTableV2, SpatialPolygonPointFieldV2,
    SpatialResolveErrorKindV2, SpatialResolveErrorV2, SpatialSemanticFieldV2, SpatialShapeErrorV2,
    SpatialShapeFieldV2, SpatialStrokeErrorV2,
};

#[path = "validation_contract/mod.rs"]
mod contract;
