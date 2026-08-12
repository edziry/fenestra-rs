use std::error::Error;

use fenestra_ui_layout::prototype::{LayoutDimensionV1, LayoutOutputV1};

use super::fixture::RawInputFixture;
pub(super) use super::placement_execution_support::{
    ScriptedLayoutEngine, VIEWPORT, limits, logical, output, placement,
};
use crate::error::SpatialErrorLocationV2;
use crate::model::{
    Affine2V2, SpatialAnchorComponentV2, SpatialAnchorTargetV2, SpatialAnchorV2,
    SpatialLocalTransformV2, SpatialNodeKeyV2, SpatialOffsetV2, SpatialPointV2, SpatialScalarV2,
};
use crate::numeric_error::{SpatialArithmeticOperationV2, SpatialTransformErrorKindV2};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};
use crate::topology::{
    SpatialFreePlacementV2, SpatialLayoutPlacementV2, SpatialNodeV2, SpatialPlacementV2,
};
use crate::vocabulary::{SpatialAffineComponentV2, SpatialTransformStageV2};

pub(super) const SCALE: i64 = SpatialScalarV2::SCALE;
pub(super) const MAXIMUM: i64 = SpatialScalarV2::MAX_RAW;
pub(super) type WorldTransformFact = (u32, [i64; 6]);

pub(super) fn fixture(nodes: Vec<SpatialNodeV2>) -> RawInputFixture {
    super::placement_execution_support::fixture(nodes)
}

pub(super) fn root() -> SpatialNodeV2 {
    super::placement_execution_support::root()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn free(
    key: u32,
    parent: u32,
    target: SpatialAnchorTargetV2,
    offset_x: i64,
    offset_y: i64,
    width: i32,
    height: i32,
    transform: SpatialLocalTransformV2,
) -> SpatialNodeV2 {
    let start = SpatialAnchorComponentV2::Start;
    free_anchored(
        key, parent, target, start, start, offset_x, offset_y, width, height, transform,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn free_anchored(
    key: u32,
    parent: u32,
    target: SpatialAnchorTargetV2,
    target_x: SpatialAnchorComponentV2,
    target_y: SpatialAnchorComponentV2,
    offset_x: i64,
    offset_y: i64,
    width: i32,
    height: i32,
    transform: SpatialLocalTransformV2,
) -> SpatialNodeV2 {
    let start = SpatialAnchorComponentV2::Start;
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(key),
        Some(SpatialNodeKeyV2::new(parent)),
        SpatialPlacementV2::Free(SpatialFreePlacementV2::new(
            width,
            height,
            SpatialAnchorV2::new(start, start),
            target,
            SpatialAnchorV2::new(target_x, target_y),
            SpatialOffsetV2::new(scalar(offset_x), scalar(offset_y)),
            transform,
        )),
        super::local_transform_support::valid_container(),
    )
}

pub(super) fn layout(
    key: u32,
    parent: u32,
    width: i32,
    height: i32,
    transform: SpatialLocalTransformV2,
) -> SpatialNodeV2 {
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(key),
        Some(SpatialNodeKeyV2::new(parent)),
        SpatialPlacementV2::Layout(SpatialLayoutPlacementV2::new(
            fixed(width),
            fixed(height),
            transform,
        )),
        super::local_transform_support::valid_container(),
    )
}

pub(super) const fn node_target(key: u32) -> SpatialAnchorTargetV2 {
    SpatialAnchorTargetV2::Node(SpatialNodeKeyV2::new(key))
}

pub(super) const fn transform(
    values: [i64; 6],
    origin_x: i64,
    origin_y: i64,
) -> SpatialLocalTransformV2 {
    SpatialLocalTransformV2::new(
        Affine2V2::new(
            scalar(values[0]),
            scalar(values[1]),
            scalar(values[2]),
            scalar(values[3]),
            scalar(values[4]),
            scalar(values[5]),
        ),
        SpatialPointV2::new(scalar(origin_x), scalar(origin_y)),
    )
}

pub(super) const fn identity() -> SpatialLocalTransformV2 {
    transform([SCALE, 0, 0, SCALE, 0, 0], 0, 0)
}

pub(super) const fn scalar(raw: i64) -> SpatialScalarV2 {
    SpatialScalarV2::new(raw)
}

pub(super) const fn world(index: u32, values: [i64; 6]) -> WorldTransformFact {
    (index, values)
}

pub(super) fn expect_valid<T>(result: Result<T, SpatialResolveErrorV2>) -> T {
    match result {
        Ok(proof) => proof,
        Err(error) => panic!("expected world-transform success, got {error:?}"),
    }
}

pub(super) fn expect_arithmetic<T>(
    result: Result<T, SpatialResolveErrorV2>,
    node: u32,
    stage: SpatialTransformStageV2,
    component: SpatialAffineComponentV2,
) {
    expect_error(
        result,
        SpatialResolveErrorKindV2::Arithmetic(SpatialArithmeticOperationV2::Affine {
            stage,
            component,
        }),
        node,
        "arithmetic",
    );
}

pub(super) fn expect_singular<T>(
    result: Result<T, SpatialResolveErrorV2>,
    node: u32,
    stage: SpatialTransformStageV2,
) {
    expect_error(
        result,
        SpatialResolveErrorKindV2::Transform(
            SpatialTransformErrorKindV2::ComposedTransformSingular(stage),
        ),
        node,
        "transform",
    );
}

pub(super) fn expect_predecessor_arithmetic<T>(
    result: Result<T, SpatialResolveErrorV2>,
    node: u32,
    operation: SpatialArithmeticOperationV2,
) {
    expect_error(
        result,
        SpatialResolveErrorKindV2::Arithmetic(operation),
        node,
        "arithmetic",
    );
}

fn expect_error<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialResolveErrorKindV2,
    node: u32,
    label: &str,
) {
    let error = match result {
        Ok(_) => panic!("expected world-transform failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), kind);
    assert_eq!(
        error.location(),
        SpatialErrorLocationV2::Node { index: node }
    );
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
    assert_eq!(error.to_string(), format!("spatial-resolve-error({label})"));
    assert_eq!(
        format!("{error:?}"),
        format!("SpatialResolveErrorV2(spatial-resolve-error({label}))")
    );
    assert!(Error::source(&error).is_none());
}

const fn fixed(value: i32) -> LayoutDimensionV1 {
    LayoutDimensionV1::new(value, value, value)
}

pub(super) fn successful_output(
    records: &[(u32, i32, i32, i32, i32)],
) -> Result<LayoutOutputV1, fenestra_ui_layout::prototype::LayoutEngineErrorV1> {
    Ok(output(records))
}
