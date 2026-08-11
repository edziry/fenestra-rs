use std::error::Error;

use super::{
    make_resolve_error, map_layout_preflight_error as map_layout_preflight_error_stage,
    prepare_direct_counts, prepare_island_plan as prepare_island_plan_stage,
    prepare_layout_preflight as prepare_layout_preflight_stage,
    prepare_local_transforms as prepare_local_transforms_stage, prepare_topology,
    validate_direct_count, validate_island_fact as validate_island_fact_stage,
};
use crate::error::SpatialErrorLocationV2;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

const DIRECT_COUNT: usize = SpatialLimitKindV2::DIRECT_ALL.len();
const U32_ROW_CAPACITY: u128 = u32::MAX as u128 + 1;

const GLOBALLY_INDEXED_DIRECT_INDICES: [usize; 9] = [0, 1, 2, 3, 4, 5, 6, 7, 11];
const PAYLOAD_DIRECT_INDICES: [usize; 3] = [8, 9, 10];

macro_rules! prepare_island_plan {
    ($fixture:expr, $limits:expr) => {{
        prepare_island_plan!(
            $fixture,
            $crate::input_validation::tests::island_support::zero_viewport(),
            $limits
        )
    }};
    ($fixture:expr, $viewport:expr, $limits:expr) => {{
        $crate::input_validation::prepare_direct_counts(
            ($fixture).input_with_viewport($viewport),
            $limits,
        )
        .and_then($crate::input_validation::prepare_topology)
        .and_then($crate::input_validation::prepare_topology_limits)
        .and_then($crate::input_validation::prepare_placement_input)
        .and_then($crate::input_validation::tests::prepare_island_plan_stage)
    }};
}

macro_rules! prepare_layout_preflight {
    ($fixture:expr, $viewport:expr, $limits:expr) => {{
        prepare_island_plan!($fixture, $viewport, $limits)
            .and_then($crate::input_validation::tests::prepare_layout_preflight_stage)
    }};
}

macro_rules! map_layout_preflight_error {
    ($plan:expr, $item:expr, $kind:expr, $location:expr) => {{
        $crate::input_validation::tests::map_layout_preflight_error_stage(
            &$plan, $item, $kind, $location,
        )
    }};
}

macro_rules! prepare_local_transforms {
    ($fixture:expr, $viewport:expr, $limits:expr) => {{
        prepare_layout_preflight!($fixture, $viewport, $limits)
            .and_then($crate::input_validation::tests::prepare_local_transforms_stage)
    }};
}

mod counts;
mod errors;
mod fixture;
mod input;
mod island_limits;
mod island_support;
mod islands;
mod layout_preflight;
mod layout_preflight_bridge;
mod layout_preflight_mappings;
mod layout_preflight_support;
mod local_transform_deferral;
mod local_transform_determinants;
mod local_transform_priority;
mod local_transform_scalars;
mod local_transform_support;
mod placement;
mod topology;
mod topology_limits;

fn check_island_fact(
    kind: SpatialLimitKindV2,
    index: Option<u32>,
    observed: u128,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    validate_island_fact_stage(kind, index, observed, limits)
}

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
