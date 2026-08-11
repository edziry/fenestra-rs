use std::error::Error;

use super::fixture::RawInputFixture;
use super::local_transform_support::VIEWPORT;
pub(super) use super::shape_structure_support::{
    circle, expect_content, fixture, fixture_with_paths, path_shape, point, polygon, rect, scalar,
    shape,
};
use crate::error::SpatialErrorLocationV2;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::model::{SpatialPointV2, SpatialScalarV2};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};
use crate::shape::{SpatialShapeGeometryV2, SpatialShapeV2};

pub(super) const fn rect_values(
    key: u32,
    owner: u32,
    x: i64,
    y: i64,
    width: i64,
    height: i64,
) -> SpatialShapeV2 {
    shape(
        key,
        owner,
        SpatialShapeGeometryV2::Rect {
            origin: point(x, y),
            width: scalar(width),
            height: scalar(height),
        },
    )
}

pub(super) const fn circle_values(
    key: u32,
    owner: u32,
    x: i64,
    y: i64,
    radius: i64,
) -> SpatialShapeV2 {
    shape(
        key,
        owner,
        SpatialShapeGeometryV2::Circle {
            center: point(x, y),
            radius: scalar(radius),
        },
    )
}

pub(super) fn limits(polygon_points_per_shape: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        if kind == SpatialLimitKindV2::PolygonPointsPerShape {
            *value = polygon_points_per_shape;
        }
    }
    SpatialLimitsV2::new(values)
}

pub(super) fn deferred_k2_limits(polygon_points_per_shape: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        match kind {
            SpatialLimitKindV2::PolygonPointsPerShape => *value = polygon_points_per_shape,
            SpatialLimitKindV2::FlattenedSegmentsPerPath
            | SpatialLimitKindV2::FlattenedSegmentsTotal => *value = 0,
            _ => {}
        }
    }
    SpatialLimitsV2::new(values)
}

pub(super) fn validate(
    fixture: &RawInputFixture,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    prepare_validated_shapes!(fixture, VIEWPORT, limits).map(|_| ())
}

pub(super) fn expect_valid<T>(result: Result<T, SpatialResolveErrorV2>) -> T {
    match result {
        Ok(proof) => proof,
        Err(error) => panic!("expected validated-shape success, got {error:?}"),
    }
}

pub(super) fn expect_limit<T>(
    result: Result<T, SpatialResolveErrorV2>,
    location: SpatialErrorLocationV2,
    observed: u128,
    maximum: u128,
) {
    let error = match result {
        Ok(_) => panic!("expected validated-shape limit failure"),
        Err(error) => error,
    };
    assert_eq!(
        error.kind(),
        SpatialResolveErrorKindV2::LimitExceeded(SpatialLimitKindV2::PolygonPointsPerShape)
    );
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

pub(super) const fn triangle(offset: i64) -> [SpatialPointV2; 3] {
    [point(offset, 0), point(offset + 2, 0), point(offset, 2)]
}

pub(super) const fn outside_low() -> i64 {
    SpatialScalarV2::MIN_RAW - 1
}

pub(super) const fn outside_high() -> i64 {
    SpatialScalarV2::MAX_RAW + 1
}
