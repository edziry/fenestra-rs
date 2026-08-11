use fenestra_ui_layout::prototype::{
    LayoutAxisV1, LayoutConstraintFieldV1, LayoutDimensionV1, LayoutExtentV1, LayoutPaddingSideV1,
    LayoutPaddingV1,
};
use fenestra_ui_spatial::prototype::{
    Affine2V2, REGISTERED_SPATIAL_LIMITS_V2, SpatialAnchorComponentV2, SpatialAnchorTargetKindV2,
    SpatialAnchorTargetV2, SpatialAnchorV2, SpatialAxisV2, SpatialContainerErrorKindV2,
    SpatialContainerV2, SpatialDependencyErrorKindV2, SpatialErrorLocationV2, SpatialExtentV2,
    SpatialFreePlacementV2, SpatialInputErrorKindV2, SpatialLayoutDimensionErrorKindV2,
    SpatialLayoutPlacementV2, SpatialLimitKindV2, SpatialLimitsV2, SpatialLocalTransformV2,
    SpatialNodeFieldV2, SpatialNodeKeyV2, SpatialNodeV2, SpatialOffsetV2, SpatialPlacementKindV2,
    SpatialPlacementV2, SpatialPointV2, SpatialScalarV2, SpatialTopologyInputV2, SpatialViewportV2,
};

mod contract {
    mod limits;
    mod traits;
    mod values;
    mod vocabulary;
}
