use std::error::Error;

use fenestra_ui_layout::prototype::{LayoutAxisV1, LayoutDimensionV1, LayoutPaddingV1};

use super::fixture::RawInputFixture;
use crate::error::{SpatialDependencyErrorKindV2, SpatialErrorLocationV2};
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

pub(super) const VIEWPORT: SpatialViewportV2 = SpatialViewportV2::new(20, 20);

pub(super) type DependencyUnitFact = (u32, Option<(u32, u32)>, Vec<u32>, Vec<u32>);

pub(super) fn fixture(nodes: Vec<SpatialNodeV2>) -> RawInputFixture {
    RawInputFixture::with_nodes(nodes)
        .with_paths(Vec::new(), Vec::new())
        .with_shapes(Vec::new(), Vec::new())
        .with_brushes(Vec::new(), Vec::new())
        .with_images(Vec::new())
        .with_clips(Vec::new())
        .with_paint_items(Vec::new())
        .with_hit_items(Vec::new())
        .with_semantic_items(Vec::new())
}

pub(super) fn root() -> SpatialNodeV2 {
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(0),
        None,
        SpatialPlacementV2::Root,
        container(),
    )
}

pub(super) fn layout(key: u32, parent: u32) -> SpatialNodeV2 {
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(key),
        Some(SpatialNodeKeyV2::new(parent)),
        SpatialPlacementV2::Layout(SpatialLayoutPlacementV2::new(
            fixed(10),
            fixed(10),
            identity(),
        )),
        container(),
    )
}

pub(super) fn free(key: u32, parent: u32, target: SpatialAnchorTargetV2) -> SpatialNodeV2 {
    free_with(key, parent, target, offset(0, 0), identity())
}

pub(super) fn free_with(
    key: u32,
    parent: u32,
    target: SpatialAnchorTargetV2,
    offset: SpatialOffsetV2,
    transform: SpatialLocalTransformV2,
) -> SpatialNodeV2 {
    let anchor = SpatialAnchorV2::new(
        SpatialAnchorComponentV2::Start,
        SpatialAnchorComponentV2::End,
    );
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(key),
        Some(SpatialNodeKeyV2::new(parent)),
        SpatialPlacementV2::Free(SpatialFreePlacementV2::new(
            10, 10, anchor, target, anchor, offset, transform,
        )),
        container(),
    )
}

pub(super) const fn node_target(key: u32) -> SpatialAnchorTargetV2 {
    SpatialAnchorTargetV2::Node(SpatialNodeKeyV2::new(key))
}

pub(super) const fn offset(x: i64, y: i64) -> SpatialOffsetV2 {
    SpatialOffsetV2::new(SpatialScalarV2::new(x), SpatialScalarV2::new(y))
}

pub(super) fn identity() -> SpatialLocalTransformV2 {
    let one = SpatialScalarV2::new(SpatialScalarV2::SCALE);
    let zero = SpatialScalarV2::new(0);
    SpatialLocalTransformV2::new(
        Affine2V2::new(one, zero, zero, one, zero, zero),
        SpatialPointV2::new(zero, zero),
    )
}

pub(super) fn dependency_limits(vertices: usize, edges: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        match kind {
            SpatialLimitKindV2::DependencyVertices => *value = vertices,
            SpatialLimitKindV2::DependencyEdges => *value = edges,
            _ => {}
        }
    }
    SpatialLimitsV2::new(values)
}

pub(super) fn validate(
    fixture: &RawInputFixture,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    prepare_dependency_graph!(fixture, VIEWPORT, limits).map(|_| ())
}

pub(super) fn expect_valid<T>(result: Result<T, SpatialResolveErrorV2>) -> T {
    match result {
        Ok(proof) => proof,
        Err(error) => panic!("expected dependency-graph success, got {error:?}"),
    }
}

pub(super) fn expect_dependency<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialDependencyErrorKindV2,
    location: SpatialErrorLocationV2,
) {
    let error = match result {
        Ok(_) => panic!("expected dependency failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), SpatialResolveErrorKindV2::Dependency(kind));
    assert_eq!(error.location(), location);
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
    assert_eq!(error.to_string(), "spatial-resolve-error(dependency)");
    assert_eq!(
        format!("{error:?}"),
        "SpatialResolveErrorV2(spatial-resolve-error(dependency))"
    );
    assert!(Error::source(&error).is_none());
}

pub(super) fn expect_limit<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialLimitKindV2,
    observed: u128,
    maximum: u128,
) {
    let error = match result {
        Ok(_) => panic!("expected dependency capacity failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), SpatialResolveErrorKindV2::LimitExceeded(kind));
    assert_eq!(error.location(), SpatialErrorLocationV2::Input);
    assert_eq!(error.observed(), Some(observed));
    assert_eq!(error.maximum(), Some(maximum));
    assert_eq!(error.to_string(), "spatial-resolve-error(limit-exceeded)");
    assert_eq!(
        format!("{error:?}"),
        "SpatialResolveErrorV2(spatial-resolve-error(limit-exceeded))"
    );
    assert!(Error::source(&error).is_none());
}

fn fixed(value: i32) -> LayoutDimensionV1 {
    LayoutDimensionV1::new(value, value, value)
}

const fn container() -> SpatialContainerV2 {
    SpatialContainerV2::new(LayoutAxisV1::Column, LayoutPaddingV1::new(0, 0, 0, 0), 0)
}
