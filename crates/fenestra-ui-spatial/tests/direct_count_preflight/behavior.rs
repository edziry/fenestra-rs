use std::error::Error;

use fenestra_ui_spatial::prototype::{
    SpatialErrorLocationV2, SpatialLimitKindV2, SpatialLimitsV2, SpatialResolveErrorKindV2,
    SpatialResolveErrorV2,
};

use super::preflight_spatial_direct_counts_v2;
use super::support::{DIRECT_COUNT, limits_with_direct};

const U32_ROW_CAPACITY: u128 = u32::MAX as u128 + 1;
const CAPPED: [bool; DIRECT_COUNT] = [
    true, true, true, true, true, true, true, true, false, false, false, true,
];

#[test]
fn signature_and_direct_limit_order_are_exact() {
    let _: fn([u128; 12], SpatialLimitsV2) -> Result<(), SpatialResolveErrorV2> =
        preflight_spatial_direct_counts_v2;
    assert_eq!(
        SpatialLimitKindV2::DIRECT_ALL,
        [
            SpatialLimitKindV2::Nodes,
            SpatialLimitKindV2::Shapes,
            SpatialLimitKindV2::Brushes,
            SpatialLimitKindV2::Clips,
            SpatialLimitKindV2::PaintItems,
            SpatialLimitKindV2::HitItems,
            SpatialLimitKindV2::SemanticItems,
            SpatialLimitKindV2::Paths,
            SpatialLimitKindV2::PathVerbsTotal,
            SpatialLimitKindV2::PolygonPointsTotal,
            SpatialLimitKindV2::GradientStopsTotal,
            SpatialLimitKindV2::Images,
        ]
    );
}

#[test]
fn every_caller_maximum_is_inclusive_and_one_over_is_exact() {
    const MAXIMUM: usize = 17;
    let limits = limits_with_direct([MAXIMUM; DIRECT_COUNT]);

    for (index, kind) in SpatialLimitKindV2::DIRECT_ALL.into_iter().enumerate() {
        let mut observed = [0; DIRECT_COUNT];
        observed[index] = MAXIMUM as u128;
        expect_valid(preflight_spatial_direct_counts_v2(observed, limits));

        observed[index] += 1;
        expect_limit(
            preflight_spatial_direct_counts_v2(observed, limits),
            kind,
            MAXIMUM as u128 + 1,
            MAXIMUM as u128,
        );
    }
}

#[test]
fn every_effective_maximum_accepts_equality_and_preserves_widened_evidence() {
    let limits = limits_with_direct([usize::MAX; DIRECT_COUNT]);

    for (index, kind) in SpatialLimitKindV2::DIRECT_ALL.into_iter().enumerate() {
        let maximum = effective_maximum(index, usize::MAX);
        let mut observed = [0; DIRECT_COUNT];
        observed[index] = maximum;
        expect_valid(preflight_spatial_direct_counts_v2(observed, limits));

        observed[index] = maximum + 1;
        expect_limit(
            preflight_spatial_direct_counts_v2(observed, limits),
            kind,
            maximum + 1,
            maximum,
        );

        observed[index] = u128::MAX;
        expect_limit(
            preflight_spatial_direct_counts_v2(observed, limits),
            kind,
            u128::MAX,
            maximum,
        );
    }
}

#[cfg(target_pointer_width = "64")]
#[test]
fn globally_indexed_tables_cap_at_one_past_u32_while_payloads_do_not() {
    for (index, capped) in CAPPED.into_iter().enumerate() {
        let maximum = effective_maximum(index, usize::MAX);
        if capped {
            assert_eq!(maximum, U32_ROW_CAPACITY);
        } else {
            assert_eq!(maximum, usize::MAX as u128);
            assert!(maximum > U32_ROW_CAPACITY);
        }
    }
}

#[test]
fn simultaneous_excesses_report_the_first_direct_kind() {
    let maxima = [3; DIRECT_COUNT];
    let limits = limits_with_direct(maxima);

    for first_excess in 0..DIRECT_COUNT {
        let mut observed = maxima.map(|maximum| maximum as u128);
        for value in &mut observed[first_excess..] {
            *value += 1;
        }
        expect_limit(
            preflight_spatial_direct_counts_v2(observed, limits),
            SpatialLimitKindV2::DIRECT_ALL[first_excess],
            4,
            3,
        );
    }
}

fn effective_maximum(index: usize, caller_maximum: usize) -> u128 {
    let caller_maximum = caller_maximum as u128;
    if CAPPED[index] {
        caller_maximum.min(U32_ROW_CAPACITY)
    } else {
        caller_maximum
    }
}

fn expect_valid(result: Result<(), SpatialResolveErrorV2>) {
    if let Err(error) = result {
        panic!("expected direct-count success, got {error:?}");
    }
}

pub(super) fn expect_limit(
    result: Result<(), SpatialResolveErrorV2>,
    kind: SpatialLimitKindV2,
    observed: u128,
    maximum: u128,
) -> SpatialResolveErrorV2 {
    let error = match result {
        Ok(()) => panic!("expected direct-count limit failure"),
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
    error
}
