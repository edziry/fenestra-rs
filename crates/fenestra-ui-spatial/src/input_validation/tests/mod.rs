use std::error::Error;

use super::{make_resolve_error, prepare_direct_counts, prepare_topology, validate_direct_count};
use crate::error::SpatialErrorLocationV2;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

const DIRECT_COUNT: usize = SpatialLimitKindV2::DIRECT_ALL.len();
const U32_ROW_CAPACITY: u128 = u32::MAX as u128 + 1;

const GLOBALLY_INDEXED_DIRECT_INDICES: [usize; 9] = [0, 1, 2, 3, 4, 5, 6, 7, 11];
const PAYLOAD_DIRECT_INDICES: [usize; 3] = [8, 9, 10];

mod counts;
mod errors;
mod fixture;
mod input;
mod placement;
mod topology;
mod topology_limits;

fn limits_with_direct(maxima: [usize; DIRECT_COUNT]) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    values[..DIRECT_COUNT].copy_from_slice(&maxima);
    SpatialLimitsV2::new(values)
}

fn expect_valid<T>(result: Result<T, SpatialResolveErrorV2>) {
    if let Err(error) = result {
        panic!("expected direct-count validation success, got {error:?}");
    }
}

fn expect_limit<T>(
    result: Result<T, SpatialResolveErrorV2>,
    limit: SpatialLimitKindV2,
    observed: u128,
    maximum: u128,
) {
    let error = match result {
        Ok(_) => panic!("expected direct-count limit failure"),
        Err(error) => error,
    };

    assert_eq!(
        error.kind(),
        SpatialResolveErrorKindV2::LimitExceeded(limit)
    );
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
