use std::error::Error;

use super::fixture::RawInputFixture;
use super::local_transform_support::{VIEWPORT, input, root};
use crate::content_diagnostic::{SpatialKeyedContentTableV2, SpatialPayloadTableV2};
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_key::SpatialPathKeyV2;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::path::{SpatialPathV2, SpatialPathVerbV2};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

pub(super) fn fixture(paths: Vec<SpatialPathV2>, verbs: Vec<SpatialPathVerbV2>) -> RawInputFixture {
    input(vec![root()]).with_paths(paths, verbs)
}

pub(super) const fn path(key: u32, start: u32, length: u32) -> SpatialPathV2 {
    SpatialPathV2::new(SpatialPathKeyV2::new(key), start, length)
}

pub(super) fn closes(count: usize) -> Vec<SpatialPathVerbV2> {
    vec![SpatialPathVerbV2::Close; count]
}

pub(super) fn limits(per_path: usize, subpaths: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        match kind {
            SpatialLimitKindV2::PathVerbsPerPath => *value = per_path,
            SpatialLimitKindV2::PathSubpathsTotal => *value = subpaths,
            _ => {}
        }
    }
    SpatialLimitsV2::new(values)
}

pub(super) fn permissive_limits() -> SpatialLimitsV2 {
    limits(usize::MAX, usize::MAX)
}

pub(super) fn validate(
    fixture: &RawInputFixture,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    prepare_path_structure!(fixture, VIEWPORT, limits).map(|_| ())
}

pub(super) fn expect_valid<T>(result: Result<T, SpatialResolveErrorV2>) -> T {
    match result {
        Ok(proof) => proof,
        Err(error) => panic!("expected path-structure success, got {error:?}"),
    }
}

pub(super) fn expect_non_dense<T>(
    result: Result<T, SpatialResolveErrorV2>,
    location: SpatialErrorLocationV2,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::NonDenseKey(SpatialKeyedContentTableV2::Path),
        location,
    );
}

pub(super) fn expect_invalid_range<T>(
    result: Result<T, SpatialResolveErrorV2>,
    location: SpatialErrorLocationV2,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::InvalidRange(SpatialPayloadTableV2::PathVerb),
        location,
    );
}

fn expect_content<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialContentErrorKindV2,
    location: SpatialErrorLocationV2,
) {
    let error = match result {
        Ok(_) => panic!("expected path-structure failure"),
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
