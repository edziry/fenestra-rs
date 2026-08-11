use std::error::Error;

use super::fixture::RawInputFixture;
use super::local_transform_support::{VIEWPORT, input, root};
use super::path_structure_support;
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_key::SpatialPathKeyV2;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::model::{SpatialPointV2, SpatialScalarV2};
use crate::path::{SpatialPathV2, SpatialPathVerbV2};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

pub(super) fn fixture(paths: Vec<SpatialPathV2>, verbs: Vec<SpatialPathVerbV2>) -> RawInputFixture {
    input(vec![root()]).with_paths(paths, verbs)
}

pub(super) const fn path(key: u32, start: u32, length: u32) -> SpatialPathV2 {
    SpatialPathV2::new(SpatialPathKeyV2::new(key), start, length)
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

pub(super) fn limits(per_path: usize, subpaths: usize) -> SpatialLimitsV2 {
    path_structure_support::limits(per_path, subpaths)
}

pub(super) fn permissive_limits() -> SpatialLimitsV2 {
    path_structure_support::permissive_limits()
}

pub(super) fn flattening_poison_limits() -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        if matches!(
            kind,
            SpatialLimitKindV2::FlattenedSegmentsPerPath
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
    prepare_validated_paths!(fixture, VIEWPORT, limits).map(|_| ())
}

pub(super) fn expect_valid<T>(result: Result<T, SpatialResolveErrorV2>) -> T {
    match result {
        Ok(proof) => proof,
        Err(error) => panic!("expected validated-path success, got {error:?}"),
    }
}

pub(super) fn expect_content<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialContentErrorKindV2,
    location: SpatialErrorLocationV2,
) {
    let error = match result {
        Ok(_) => panic!("expected validated-path content failure"),
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

pub(super) fn expect_limit<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialLimitKindV2,
    location: SpatialErrorLocationV2,
    observed: u128,
    maximum: u128,
) {
    let error = match result {
        Ok(_) => panic!("expected validated-path limit failure"),
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
