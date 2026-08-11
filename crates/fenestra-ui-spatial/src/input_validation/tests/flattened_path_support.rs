use std::error::Error;

use super::fixture::RawInputFixture;
use super::local_transform_support::VIEWPORT;
use super::validated_semantic_support;
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::SpatialPathVerbFieldV2;
use crate::geometry_kernel::{GeometryK2Error, flatten_path_k2, validate_path_k1};
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::model::{SpatialPointV2, SpatialScalarV2};
use crate::path::{SpatialPathV2, SpatialPathVerbV2};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

pub(super) const DEPTH_16_NONFLAT_HEIGHT: i64 = 257 * 4_294_967_296;

pub(super) fn fixture(paths: Vec<SpatialPathV2>, verbs: Vec<SpatialPathVerbV2>) -> RawInputFixture {
    validated_semantic_support::fixture(Vec::new()).with_paths(paths, verbs)
}

pub(super) const fn path(key: u32, start: u32, length: u32) -> SpatialPathV2 {
    SpatialPathV2::new(
        crate::geometry_key::SpatialPathKeyV2::new(key),
        start,
        length,
    )
}

pub(super) const fn point(x: i64, y: i64) -> SpatialPointV2 {
    SpatialPointV2::new(SpatialScalarV2::new(x), SpatialScalarV2::new(y))
}

pub(super) const fn move_to(x: i64, y: i64) -> SpatialPathVerbV2 {
    SpatialPathVerbV2::MoveTo { to: point(x, y) }
}

pub(super) const fn line_to(x: i64, y: i64) -> SpatialPathVerbV2 {
    SpatialPathVerbV2::LineTo { to: point(x, y) }
}

pub(super) const fn quadratic(height: i64) -> [SpatialPathVerbV2; 2] {
    [
        move_to(-height, 0),
        SpatialPathVerbV2::QuadraticTo {
            control: point(0, height),
            to: point(height, 0),
        },
    ]
}

pub(super) fn limits(per_path: usize, total: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        match kind {
            SpatialLimitKindV2::FlattenedSegmentsPerPath => *value = per_path,
            SpatialLimitKindV2::FlattenedSegmentsTotal => *value = total,
            _ => {}
        }
    }
    SpatialLimitsV2::new(values)
}

pub(super) fn deferred_limits(per_path: usize, total: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        match kind {
            SpatialLimitKindV2::FlattenedSegmentsPerPath => *value = per_path,
            SpatialLimitKindV2::FlattenedSegmentsTotal => *value = total,
            SpatialLimitKindV2::DependencyVertices | SpatialLimitKindV2::DependencyEdges => {
                *value = 0;
            }
            _ => {}
        }
    }
    SpatialLimitsV2::new(values)
}

pub(super) fn validate(
    fixture: &RawInputFixture,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    prepare_flattened_paths!(fixture, VIEWPORT, limits).map(|_| ())
}

pub(super) fn map_error(error: GeometryK2Error) -> SpatialResolveErrorV2 {
    super::map_path_k2_error_stage(error)
}

pub(super) fn kernel_error(
    path: u32,
    verbs: &[SpatialPathVerbV2],
    accepted_total: usize,
    maximum_per_path: usize,
    maximum_total: usize,
) -> GeometryK2Error {
    let validated = match validate_path_k1(path, verbs, 0, usize::MAX) {
        Ok(validated) => validated,
        Err(error) => panic!("expected K1 path success, got {error:?}"),
    };
    match flatten_path_k2(
        path,
        validated,
        accepted_total,
        maximum_per_path,
        maximum_total,
    ) {
        Ok(_) => panic!("expected K2 path failure"),
        Err(error) => error,
    }
}

pub(super) fn expect_valid<T>(result: Result<T, SpatialResolveErrorV2>) -> T {
    match result {
        Ok(proof) => proof,
        Err(error) => panic!("expected flattened-path success, got {error:?}"),
    }
}

pub(super) fn expect_limit<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialLimitKindV2,
    path: u32,
    verb: u32,
    observed: u128,
    maximum: u128,
) {
    let error = match result {
        Ok(_) => panic!("expected flattened-path limit failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), SpatialResolveErrorKindV2::LimitExceeded(kind));
    assert_eq!(error.location(), path_verb_location(path, verb));
    assert_eq!(error.observed(), Some(observed));
    assert_eq!(error.maximum(), Some(maximum));
    assert_eq!(error.to_string(), "spatial-resolve-error(limit-exceeded)");
    assert_eq!(
        format!("{error:?}"),
        "SpatialResolveErrorV2(spatial-resolve-error(limit-exceeded))"
    );
    assert!(Error::source(&error).is_none());
}

pub(super) fn expect_nonflat<T>(result: Result<T, SpatialResolveErrorV2>, path: u32, verb: u32) {
    let error = match result {
        Ok(_) => panic!("expected non-flat K2 failure"),
        Err(error) => error,
    };
    assert_eq!(
        error.kind(),
        SpatialResolveErrorKindV2::Content(SpatialContentErrorKindV2::NonFlatAtMaximumDepth)
    );
    assert_eq!(error.location(), path_verb_location(path, verb));
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
    assert_eq!(error.to_string(), "spatial-resolve-error(content)");
    assert_eq!(
        format!("{error:?}"),
        "SpatialResolveErrorV2(spatial-resolve-error(content))"
    );
    assert!(Error::source(&error).is_none());
}

pub(super) const fn path_verb_location(path: u32, verb: u32) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::PathVerb {
        path,
        verb,
        field: SpatialPathVerbFieldV2::Kind,
    }
}
