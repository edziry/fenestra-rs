use fenestra_ui_ir::prototype::{
    SUPPORTED_SPATIAL_FORMAT, SpatialAnchorComponentV2, SpatialAnchorTargetRecipeV2, SpatialAxisV2,
    SpatialBindingV2, SpatialBrushContentV2, SpatialBrushDeclarationV2, SpatialBrushSymbolV2,
    SpatialClipAddressV2, SpatialClipDeclarationV2, SpatialClipSymbolV2, SpatialContainerRecipeV2,
    SpatialCoverageRecipeV2, SpatialDimensionRecipeV2, SpatialFieldV2, SpatialFillRuleV2,
    SpatialFormatVersion, SpatialFreePlacementRecipeV2, SpatialGradientStopV2, SpatialHitRecipeV2,
    SpatialImageDeclarationV2, SpatialImageSymbolV2, SpatialLayoutPlacementRecipeV2,
    SpatialNodeDeclarationV2, SpatialNodeParentV2, SpatialNodeSymbolV2, SpatialPaddingRecipeV2,
    SpatialPaintRecipeV2, SpatialPathVerbRecipeV2, SpatialPlacementRecipeV2, SpatialPointRecipeV2,
    SpatialPolygonPointV2, SpatialProgramV2, SpatialSemanticRecipeV2, SpatialShapeDeclarationV2,
    SpatialShapeGeometryV2, SpatialShapeSymbolV2, SpatialTransformRecipeV2,
    SpatialValidationLimitsV2, SpatialViewportContainerV2, ValidatedSpatialProgramV2,
    validate_spatial,
};

#[path = "spatial_api_contract/mod.rs"]
mod contract;
