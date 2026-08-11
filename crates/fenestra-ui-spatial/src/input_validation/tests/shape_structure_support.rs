use std::error::Error;

use super::fixture::RawInputFixture;
use super::local_transform_support::{VIEWPORT, free_node, identity, input, root};
use super::validated_path_support::permissive_limits;
use crate::content_diagnostic::{
    SpatialContentReferenceV2, SpatialKeyedContentTableV2, SpatialPayloadTableV2,
};
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_key::{SpatialPathKeyV2, SpatialShapeKeyV2};
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::model::{SpatialNodeKeyV2, SpatialPointV2, SpatialScalarV2};
use crate::path::{SpatialPathV2, SpatialPathVerbV2};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};
use crate::shape::{SpatialShapeGeometryV2, SpatialShapeV2};

pub(super) fn fixture(shapes: Vec<SpatialShapeV2>, points: Vec<SpatialPointV2>) -> RawInputFixture {
    fixture_with_paths(shapes, points, Vec::new(), Vec::new())
}

pub(super) fn fixture_with_paths(
    shapes: Vec<SpatialShapeV2>,
    points: Vec<SpatialPointV2>,
    paths: Vec<SpatialPathV2>,
    verbs: Vec<SpatialPathVerbV2>,
) -> RawInputFixture {
    let transform = identity();
    input(vec![
        root(),
        free_node(1, 0, 10, 10, transform),
        free_node(2, 0, 10, 10, transform),
    ])
    .with_paths(paths, verbs)
    .with_shapes(shapes, points)
}

pub(super) const fn shape(
    key: u32,
    owner: u32,
    geometry: SpatialShapeGeometryV2,
) -> SpatialShapeV2 {
    SpatialShapeV2::new(
        SpatialShapeKeyV2::new(key),
        SpatialNodeKeyV2::new(owner),
        geometry,
    )
}

pub(super) const fn rect(key: u32, owner: u32) -> SpatialShapeV2 {
    shape(
        key,
        owner,
        SpatialShapeGeometryV2::Rect {
            origin: point(0, 0),
            width: scalar(1),
            height: scalar(1),
        },
    )
}

pub(super) const fn circle(key: u32, owner: u32) -> SpatialShapeV2 {
    shape(
        key,
        owner,
        SpatialShapeGeometryV2::Circle {
            center: point(0, 0),
            radius: scalar(1),
        },
    )
}

pub(super) const fn polygon(key: u32, owner: u32, start: u32, length: u32) -> SpatialShapeV2 {
    shape(
        key,
        owner,
        SpatialShapeGeometryV2::Polygon {
            point_start: start,
            point_length: length,
        },
    )
}

pub(super) const fn path_shape(key: u32, owner: u32, path: u32) -> SpatialShapeV2 {
    shape(
        key,
        owner,
        SpatialShapeGeometryV2::Path {
            path: SpatialPathKeyV2::new(path),
        },
    )
}

pub(super) const fn point(x: i64, y: i64) -> SpatialPointV2 {
    SpatialPointV2::new(scalar(x), scalar(y))
}

pub(super) const fn scalar(raw: i64) -> SpatialScalarV2 {
    SpatialScalarV2::new(raw)
}

pub(super) fn shape_k1_poison_limits() -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        if matches!(
            kind,
            SpatialLimitKindV2::PolygonPointsPerShape
                | SpatialLimitKindV2::FlattenedSegmentsPerPath
                | SpatialLimitKindV2::FlattenedSegmentsTotal
        ) {
            *value = 0;
        }
    }
    SpatialLimitsV2::new(values)
}

pub(super) fn validate(
    fixture: &RawInputFixture,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    prepare_shape_structure!(fixture, VIEWPORT, limits).map(|_| ())
}

pub(super) fn expect_valid<T>(result: Result<T, SpatialResolveErrorV2>) -> T {
    match result {
        Ok(proof) => proof,
        Err(error) => panic!("expected shape-structure success, got {error:?}"),
    }
}

pub(super) fn expect_non_dense<T>(
    result: Result<T, SpatialResolveErrorV2>,
    location: SpatialErrorLocationV2,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::NonDenseKey(SpatialKeyedContentTableV2::Shape),
        location,
    );
}

pub(super) fn expect_invalid_range<T>(
    result: Result<T, SpatialResolveErrorV2>,
    location: SpatialErrorLocationV2,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::InvalidRange(SpatialPayloadTableV2::PolygonPoint),
        location,
    );
}

pub(super) fn expect_reference<T>(
    result: Result<T, SpatialResolveErrorV2>,
    reference: SpatialContentReferenceV2,
    location: SpatialErrorLocationV2,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::InvalidReference(reference),
        location,
    );
}

pub(super) fn expect_content<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialContentErrorKindV2,
    location: SpatialErrorLocationV2,
) {
    let error = match result {
        Ok(_) => panic!("expected shape-structure content failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), SpatialResolveErrorKindV2::Content(kind));
    assert_eq!(error.location(), location);
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
    assert_eq!(error.to_string(), "spatial-resolve-error(content)");
    assert_eq!(
        format!("{error:?}"),
        "SpatialResolveErrorV2(spatial-resolve-error(content))"
    );
    assert!(Error::source(&error).is_none());
}

pub(super) fn limits() -> SpatialLimitsV2 {
    permissive_limits()
}
