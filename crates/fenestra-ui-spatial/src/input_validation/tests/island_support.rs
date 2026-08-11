use std::error::Error;

use fenestra_ui_layout::prototype::{LayoutAxisV1, LayoutDimensionV1, LayoutPaddingV1};

use super::fixture::RawInputFixture;
use crate::error::{SpatialErrorLocationV2, SpatialInputErrorKindV2};
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::model::{
    Affine2V2, SpatialAnchorComponentV2, SpatialAnchorTargetV2, SpatialAnchorV2,
    SpatialLocalTransformV2, SpatialNodeKeyV2, SpatialOffsetV2, SpatialPointV2, SpatialScalarV2,
    SpatialViewportV2,
};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};
use crate::topology::{
    SpatialContainerV2, SpatialFreePlacementV2, SpatialLayoutPlacementV2, SpatialNodeV2,
    SpatialPlacementV2,
};

pub(super) fn fixture(nodes: Vec<SpatialNodeV2>) -> RawInputFixture {
    RawInputFixture::with_nodes(nodes)
}

pub(super) const fn zero_viewport() -> SpatialViewportV2 {
    SpatialViewportV2::new(0, 0)
}

pub(super) fn root() -> SpatialNodeV2 {
    root_with(deferred_container())
}

pub(super) fn root_with(container: SpatialContainerV2) -> SpatialNodeV2 {
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(0),
        None,
        SpatialPlacementV2::Root,
        container,
    )
}

pub(super) fn free(key: u32, parent: u32) -> SpatialNodeV2 {
    free_with(key, parent, 0, 0, deferred_container())
}

pub(super) fn free_with(
    key: u32,
    parent: u32,
    width: i32,
    height: i32,
    container: SpatialContainerV2,
) -> SpatialNodeV2 {
    node_with(key, parent, free_placement(width, height), container)
}

pub(super) fn layout(key: u32, parent: u32) -> SpatialNodeV2 {
    layout_with(
        key,
        parent,
        LayoutDimensionV1::new(-1, -2, -3),
        LayoutDimensionV1::new(-4, -5, -6),
        deferred_container(),
    )
}

pub(super) fn layout_with(
    key: u32,
    parent: u32,
    width: LayoutDimensionV1,
    height: LayoutDimensionV1,
    container: SpatialContainerV2,
) -> SpatialNodeV2 {
    node_with(
        key,
        parent,
        SpatialPlacementV2::Layout(SpatialLayoutPlacementV2::new(
            width,
            height,
            deferred_transform(),
        )),
        container,
    )
}

pub(super) fn node(key: u32, parent: u32, placement: SpatialPlacementV2) -> SpatialNodeV2 {
    node_with(key, parent, placement, deferred_container())
}

pub(super) fn node_with(
    key: u32,
    parent: u32,
    placement: SpatialPlacementV2,
    container: SpatialContainerV2,
) -> SpatialNodeV2 {
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(key),
        Some(SpatialNodeKeyV2::new(parent)),
        placement,
        container,
    )
}

pub(super) fn island_limits(islands: usize, per_island: usize, total: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        match kind {
            SpatialLimitKindV2::Islands => *value = islands,
            SpatialLimitKindV2::LayoutInputRecordsPerIsland => *value = per_island,
            SpatialLimitKindV2::LayoutInputRecordsTotal => *value = total,
            _ => {}
        }
    }
    SpatialLimitsV2::new(values)
}

pub(super) fn expect_plan<T>(result: Result<T, SpatialResolveErrorV2>) -> T {
    match result {
        Ok(proof) => proof,
        Err(error) => panic!("expected island planning success, got {error:?}"),
    }
}

pub(super) fn expect_limit<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialLimitKindV2,
    location: SpatialErrorLocationV2,
    observed: u128,
    maximum: u128,
) {
    let error = match result {
        Ok(_) => panic!("expected island capacity failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), SpatialResolveErrorKindV2::LimitExceeded(kind));
    assert_eq!(error.location(), location);
    assert_eq!(error.observed(), Some(observed));
    assert_eq!(error.maximum(), Some(maximum));
    assert_eq!(error.to_string(), "spatial-resolve-error(limit-exceeded)");
    assert_eq!(
        format!("{error:?}"),
        "SpatialResolveErrorV2(spatial-resolve-error(limit-exceeded))"
    );
    assert!(Error::source(&error).is_none());
}

pub(super) fn expect_input(
    result: Result<(), SpatialResolveErrorV2>,
    kind: SpatialInputErrorKindV2,
    location: SpatialErrorLocationV2,
) {
    let error = match result {
        Ok(()) => panic!("expected earlier input failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), SpatialResolveErrorKindV2::Input(kind));
    assert_eq!(error.location(), location);
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
}

fn free_placement(width: i32, height: i32) -> SpatialPlacementV2 {
    let anchor = SpatialAnchorV2::new(
        SpatialAnchorComponentV2::Start,
        SpatialAnchorComponentV2::End,
    );
    SpatialPlacementV2::Free(SpatialFreePlacementV2::new(
        width,
        height,
        anchor,
        SpatialAnchorTargetV2::Node(SpatialNodeKeyV2::new(u32::MAX)),
        anchor,
        SpatialOffsetV2::new(SpatialScalarV2::new(0), SpatialScalarV2::new(0)),
        deferred_transform(),
    ))
}

fn deferred_container() -> SpatialContainerV2 {
    SpatialContainerV2::new(
        LayoutAxisV1::Column,
        LayoutPaddingV1::new(-1, -2, -3, -4),
        -5,
    )
}

fn deferred_transform() -> SpatialLocalTransformV2 {
    let scalar = SpatialScalarV2::new(i64::MAX);
    SpatialLocalTransformV2::new(
        Affine2V2::new(scalar, scalar, scalar, scalar, scalar, scalar),
        SpatialPointV2::new(scalar, scalar),
    )
}
