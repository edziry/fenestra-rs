use std::error::Error;

use super::fixture::RawInputFixture;
use super::local_transform_support::VIEWPORT;
use super::validated_semantic_support;
use crate::aabb::SpatialAabbV2;
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_kernel::GeometryK3Error;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::model::SpatialScalarV2;
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};
use crate::vocabulary::SpatialAxisV2;

pub(super) fn fixture(
    shapes: Vec<crate::shape::SpatialShapeV2>,
    points: Vec<crate::model::SpatialPointV2>,
    paints: Vec<crate::paint::SpatialPaintV2>,
    hits: Vec<crate::content_item::SpatialHitV2>,
) -> RawInputFixture {
    validated_semantic_support::fixture_with_items(paints, hits, Vec::new())
        .with_paths(Vec::new(), Vec::new())
        .with_shapes(shapes, points)
        .with_clips(Vec::new())
}

pub(super) const fn scalar(raw: i64) -> SpatialScalarV2 {
    SpatialScalarV2::new(raw)
}

pub(super) fn aabb(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> SpatialAabbV2 {
    match SpatialAabbV2::from_edges(scalar(min_x), scalar(min_y), scalar(max_x), scalar(max_y)) {
        Some(bounds) => bounds,
        None => panic!("expected test bounds to be canonical and ordered"),
    }
}

pub(super) const fn limits() -> SpatialLimitsV2 {
    SpatialLimitsV2::new([usize::MAX; SpatialLimitKindV2::ALL.len()])
}

pub(super) fn deferred_limits() -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        match kind {
            SpatialLimitKindV2::FlattenedSegmentsPerPath
            | SpatialLimitKindV2::FlattenedSegmentsTotal => *value = 2,
            SpatialLimitKindV2::DependencyVertices | SpatialLimitKindV2::DependencyEdges => {
                *value = 0;
            }
            _ => {}
        }
    }
    SpatialLimitsV2::new(values)
}

pub(super) fn validate(fixture: &RawInputFixture) -> Result<(), SpatialResolveErrorV2> {
    prepare_local_bounds!(fixture, VIEWPORT, limits()).map(|_| ())
}

pub(super) fn map_error(error: GeometryK3Error) -> SpatialResolveErrorV2 {
    super::map_geometry_k3_error_stage(error)
}

pub(super) fn expect_valid<T>(result: Result<T, SpatialResolveErrorV2>) -> T {
    match result {
        Ok(proof) => proof,
        Err(error) => panic!("expected local-bounds success, got {error:?}"),
    }
}

pub(super) fn expect_bounds_error<T>(
    result: Result<T, SpatialResolveErrorV2>,
    axis: SpatialAxisV2,
    location: SpatialErrorLocationV2,
) {
    let error = match result {
        Ok(_) => panic!("expected local-bounds failure"),
        Err(error) => error,
    };
    assert_eq!(
        error.kind(),
        SpatialResolveErrorKindV2::Content(SpatialContentErrorKindV2::LocalBoundsOutOfDomain(axis))
    );
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
