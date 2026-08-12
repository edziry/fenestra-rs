use fenestra_ui_spatial::prototype::{
    SpatialBrushV2, SpatialClipV2, SpatialGradientStopV2, SpatialHitV2, SpatialImageV2,
    SpatialNodeV2, SpatialPaintV2, SpatialPathV2, SpatialPathVerbV2, SpatialPointV2,
    SpatialSemanticGeometryV2, SpatialShapeV2, SpatialViewportV2,
};

use crate::*;

#[allow(clippy::type_complexity)]
type OwnedInputConstructor = fn(
    SpatialViewportV2,
    Box<[SpatialNodeV2]>,
    Box<[SpatialPointV2]>,
    Box<[SpatialPathVerbV2]>,
    Box<[SpatialPathV2]>,
    Box<[SpatialShapeV2]>,
    Box<[SpatialClipV2]>,
    Box<[SpatialGradientStopV2]>,
    Box<[SpatialBrushV2]>,
    Box<[SpatialImageV2]>,
    Box<[SpatialPaintV2]>,
    Box<[SpatialHitV2]>,
    Box<[SpatialSemanticGeometryV2]>,
) -> SpatialOwnedInputV2;

#[test]
fn owned_input_function_signatures_are_exact() {
    let _: OwnedInputConstructor = SpatialOwnedInputV2::new;
    let _: for<'a> fn(&'a SpatialOwnedInputV2) -> SpatialInputV2<'a> =
        SpatialOwnedInputV2::as_input;
}
