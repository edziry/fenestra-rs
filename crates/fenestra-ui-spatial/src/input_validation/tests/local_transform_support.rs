use std::error::Error;

use fenestra_ui_layout::prototype::{LayoutAxisV1, LayoutDimensionV1, LayoutPaddingV1};

use super::fixture::RawInputFixture;
use super::island_support::{fixture, island_limits, node_with, root_with};
use crate::error::SpatialErrorLocationV2;
use crate::limits::SpatialLimitsV2;
use crate::model::{
    Affine2V2, SpatialAnchorComponentV2, SpatialAnchorTargetV2, SpatialAnchorV2,
    SpatialLocalTransformV2, SpatialNodeKeyV2, SpatialOffsetV2, SpatialPointV2, SpatialScalarV2,
    SpatialViewportV2,
};
use crate::numeric_error::SpatialTransformErrorKindV2;
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};
use crate::topology::{
    SpatialContainerV2, SpatialFreePlacementV2, SpatialLayoutPlacementV2, SpatialNodeV2,
    SpatialPlacementV2,
};
use crate::vocabulary::{SpatialNodeFieldV2, SpatialTransformScalarFieldV2};

pub(super) const VIEWPORT: SpatialViewportV2 = SpatialViewportV2::new(20, 20);

#[derive(Clone, Copy)]
pub(super) enum Placement {
    Layout,
    Free,
}

pub(super) fn input(nodes: Vec<SpatialNodeV2>) -> RawInputFixture {
    fixture(nodes)
}

pub(super) fn root() -> SpatialNodeV2 {
    root_with(valid_container())
}

pub(super) fn root_with_container(container: SpatialContainerV2) -> SpatialNodeV2 {
    root_with(container)
}

pub(super) fn node(
    placement: Placement,
    key: u32,
    parent: u32,
    transform: SpatialLocalTransformV2,
) -> SpatialNodeV2 {
    match placement {
        Placement::Layout => layout_node(key, parent, fixed(10), fixed(10), transform),
        Placement::Free => free_node(key, parent, 10, 10, transform),
    }
}

pub(super) fn layout_node(
    key: u32,
    parent: u32,
    width: LayoutDimensionV1,
    height: LayoutDimensionV1,
    transform: SpatialLocalTransformV2,
) -> SpatialNodeV2 {
    node_with(
        key,
        parent,
        SpatialPlacementV2::Layout(SpatialLayoutPlacementV2::new(width, height, transform)),
        valid_container(),
    )
}

pub(super) fn free_node(
    key: u32,
    parent: u32,
    width: i32,
    height: i32,
    transform: SpatialLocalTransformV2,
) -> SpatialNodeV2 {
    let anchor = SpatialAnchorV2::new(
        SpatialAnchorComponentV2::Start,
        SpatialAnchorComponentV2::End,
    );
    node_with(
        key,
        parent,
        SpatialPlacementV2::Free(SpatialFreePlacementV2::new(
            width,
            height,
            anchor,
            SpatialAnchorTargetV2::Node(SpatialNodeKeyV2::new(u32::MAX)),
            anchor,
            SpatialOffsetV2::new(scalar(0), scalar(0)),
            transform,
        )),
        valid_container(),
    )
}

pub(super) const fn valid_container() -> SpatialContainerV2 {
    SpatialContainerV2::new(LayoutAxisV1::Column, LayoutPaddingV1::new(0, 0, 0, 0), 0)
}

pub(super) const fn container(
    axis: LayoutAxisV1,
    padding: LayoutPaddingV1,
    gap: i32,
) -> SpatialContainerV2 {
    SpatialContainerV2::new(axis, padding, gap)
}

pub(super) const fn fixed(value: i32) -> LayoutDimensionV1 {
    LayoutDimensionV1::new(value, value, value)
}

pub(super) fn limits(islands: usize, per_island: usize, total: usize) -> SpatialLimitsV2 {
    island_limits(islands, per_island, total)
}

pub(super) const fn scalar(raw: i64) -> SpatialScalarV2 {
    SpatialScalarV2::new(raw)
}

pub(super) const fn identity_values() -> [i64; 8] {
    [
        SpatialScalarV2::SCALE,
        0,
        0,
        SpatialScalarV2::SCALE,
        0,
        0,
        0,
        0,
    ]
}

pub(super) fn identity() -> SpatialLocalTransformV2 {
    transform(identity_values())
}

pub(super) fn transform(values: [i64; 8]) -> SpatialLocalTransformV2 {
    SpatialLocalTransformV2::new(
        Affine2V2::new(
            scalar(values[0]),
            scalar(values[1]),
            scalar(values[2]),
            scalar(values[3]),
            scalar(values[4]),
            scalar(values[5]),
        ),
        SpatialPointV2::new(scalar(values[6]), scalar(values[7])),
    )
}

pub(super) fn set_field(values: &mut [i64; 8], field: SpatialTransformScalarFieldV2, raw: i64) {
    values[field_index(field)] = raw;
}

pub(super) const fn node_field(field: SpatialTransformScalarFieldV2) -> SpatialNodeFieldV2 {
    match field {
        SpatialTransformScalarFieldV2::AffineA => SpatialNodeFieldV2::AffineA,
        SpatialTransformScalarFieldV2::AffineB => SpatialNodeFieldV2::AffineB,
        SpatialTransformScalarFieldV2::AffineC => SpatialNodeFieldV2::AffineC,
        SpatialTransformScalarFieldV2::AffineD => SpatialNodeFieldV2::AffineD,
        SpatialTransformScalarFieldV2::AffineTx => SpatialNodeFieldV2::AffineTx,
        SpatialTransformScalarFieldV2::AffineTy => SpatialNodeFieldV2::AffineTy,
        SpatialTransformScalarFieldV2::TransformOriginX => SpatialNodeFieldV2::TransformOriginX,
        SpatialTransformScalarFieldV2::TransformOriginY => SpatialNodeFieldV2::TransformOriginY,
    }
}

pub(super) fn validate(
    fixture: &RawInputFixture,
    viewport: SpatialViewportV2,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    prepare_local_transforms!(fixture, viewport, limits).map(|_| ())
}

pub(super) fn expect_valid<T>(result: Result<T, SpatialResolveErrorV2>) -> T {
    match result {
        Ok(proof) => proof,
        Err(error) => panic!("expected local-transform success, got {error:?}"),
    }
}

pub(super) fn expect_transform(
    result: Result<(), SpatialResolveErrorV2>,
    kind: SpatialTransformErrorKindV2,
    location: SpatialErrorLocationV2,
) {
    let error = match result {
        Ok(()) => panic!("expected local-transform failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), SpatialResolveErrorKindV2::Transform(kind));
    assert_eq!(error.location(), location);
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
    assert_eq!(error.to_string(), "spatial-resolve-error(transform)");
    assert_eq!(
        format!("{error:?}"),
        "SpatialResolveErrorV2(spatial-resolve-error(transform))"
    );
    assert!(Error::source(&error).is_none());
}

fn field_index(field: SpatialTransformScalarFieldV2) -> usize {
    SpatialTransformScalarFieldV2::ALL
        .iter()
        .position(|candidate| *candidate == field)
        .expect("the field comes from the closed transform vocabulary")
}
