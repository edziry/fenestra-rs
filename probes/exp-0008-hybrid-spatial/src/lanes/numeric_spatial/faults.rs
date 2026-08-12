use super::candidates::{euclid_detects, fixed_detects, kurbo_detects};
use super::oracle;
use super::types::{NumericFaultKindV2, NumericFaultV2};

pub(crate) fn numeric_faults_v2() -> Vec<NumericFaultV2> {
    [
        NumericFaultKindV2::BelowMinimum,
        NumericFaultKindV2::AboveMaximum,
        NumericFaultKindV2::CompositionOverflow,
        NumericFaultKindV2::SingularInverse,
        NumericFaultKindV2::NonFiniteCandidate,
    ]
    .into_iter()
    .map(|kind| NumericFaultV2 {
        kind,
        detected_by_literal: oracle::detects(kind),
        detected_by_euclid: euclid_detects(kind),
        detected_by_kurbo: kurbo_detects(kind),
        detected_by_fixed: fixed_detects(kind),
    })
    .collect()
}
