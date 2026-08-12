//! Free-placement target-scope validation.

use super::{dependency_error, trusted_ordinal};
use crate::error::{SpatialDependencyErrorKindV2, SpatialErrorLocationV2};
use crate::model::SpatialAnchorTargetV2;
use crate::resolve_error::SpatialResolveErrorV2;
use crate::topology::{SpatialNodeV2, SpatialPlacementV2};
use crate::vocabulary::SpatialNodeFieldV2;

pub(super) fn validate_targets(nodes: &[SpatialNodeV2]) -> Result<(), SpatialResolveErrorV2> {
    for (index, node) in nodes.iter().copied().enumerate().skip(1) {
        let SpatialPlacementV2::Free(free) = node.placement() else {
            continue;
        };
        let SpatialAnchorTargetV2::Node(target) = free.target() else {
            continue;
        };
        let target = target.get();
        let kind = if target == 0 {
            Some(SpatialDependencyErrorKindV2::SentinelNodeTarget)
        } else if u128::from(target) >= nodes.len() as u128 {
            Some(SpatialDependencyErrorKindV2::MissingTarget)
        } else {
            None
        };
        if let Some(kind) = kind {
            return Err(dependency_error(kind, target_location(index)));
        }
    }

    for (index, node) in nodes.iter().copied().enumerate().skip(1) {
        let SpatialPlacementV2::Free(free) = node.placement() else {
            continue;
        };
        let SpatialAnchorTargetV2::Node(target) = free.target() else {
            continue;
        };
        if target.get() == trusted_ordinal(index) {
            return Err(dependency_error(
                SpatialDependencyErrorKindV2::SelfTarget,
                target_location(index),
            ));
        }
    }

    Ok(())
}

fn target_location(index: usize) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::NodeField {
        index: trusted_ordinal(index),
        field: SpatialNodeFieldV2::TargetKey,
    }
}
