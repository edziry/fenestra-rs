//! Shared direct-count preflight for raw and materialized spatial inputs.

use crate::error::SpatialErrorLocationV2;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::resolve_error::SpatialResolveErrorV2;

const U32_ROW_CAPACITY: u128 = u32::MAX as u128 + 1;

/// Validates widened direct table counts before spatial input allocation.
#[must_use = "direct-count preflight errors must be handled"]
pub fn preflight_spatial_direct_counts_v2(
    observed: [u128; 12],
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    for (kind, observed) in SpatialLimitKindV2::DIRECT_ALL.into_iter().zip(observed) {
        let caller_maximum = limits.limit(kind) as u128;
        let maximum = match kind {
            SpatialLimitKindV2::Nodes
            | SpatialLimitKindV2::Shapes
            | SpatialLimitKindV2::Brushes
            | SpatialLimitKindV2::Clips
            | SpatialLimitKindV2::PaintItems
            | SpatialLimitKindV2::HitItems
            | SpatialLimitKindV2::SemanticItems
            | SpatialLimitKindV2::Paths
            | SpatialLimitKindV2::Images => caller_maximum.min(U32_ROW_CAPACITY),
            SpatialLimitKindV2::PathVerbsTotal
            | SpatialLimitKindV2::PolygonPointsTotal
            | SpatialLimitKindV2::GradientStopsTotal => caller_maximum,
            _ => unreachable!("non-direct spatial limit in direct-count preflight"),
        };

        if observed > maximum {
            return Err(SpatialResolveErrorV2::limit_exceeded(
                kind,
                SpatialErrorLocationV2::Input,
                observed,
                maximum,
            ));
        }
    }

    Ok(())
}
