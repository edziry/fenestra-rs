//! Remaining raw placement-input validation.

use super::topology::{TopologyLimitsProof, input_error, node_input_error, trusted_node_ordinal};
use crate::error::{SpatialErrorLocationV2, SpatialInputErrorKindV2};
use crate::resolve_error::SpatialResolveErrorV2;
use crate::topology::SpatialPlacementV2;
use crate::vocabulary::{SpatialAxisV2, SpatialExtentV2, SpatialNodeFieldV2};

pub(super) struct PlacementInputProof<'a> {
    topology_limits: TopologyLimitsProof<'a>,
}

pub(super) fn prepare_placement_input(
    topology_limits: TopologyLimitsProof<'_>,
) -> Result<PlacementInputProof<'_>, SpatialResolveErrorV2> {
    let topology = topology_limits.input().topology();
    let nodes = topology.nodes();

    for (index, node) in nodes.iter().copied().enumerate().skip(1) {
        if matches!(node.placement(), SpatialPlacementV2::Root) {
            return Err(node_input_error(
                SpatialInputErrorKindV2::RootPlacementOnNonRoot,
                trusted_node_ordinal(index),
                SpatialNodeFieldV2::Placement,
            ));
        }
    }

    let viewport = topology.viewport();
    if viewport.width() < 0 {
        return Err(input_error(
            SpatialInputErrorKindV2::NegativeViewport(SpatialExtentV2::Width),
            SpatialErrorLocationV2::Viewport {
                extent: SpatialExtentV2::Width,
            },
        ));
    }
    if viewport.height() < 0 {
        return Err(input_error(
            SpatialInputErrorKindV2::NegativeViewport(SpatialExtentV2::Height),
            SpatialErrorLocationV2::Viewport {
                extent: SpatialExtentV2::Height,
            },
        ));
    }

    for (index, node) in nodes.iter().copied().enumerate() {
        let SpatialPlacementV2::Free(free) = node.placement() else {
            continue;
        };
        let ordinal = trusted_node_ordinal(index);
        if free.width() < 0 {
            return Err(node_input_error(
                SpatialInputErrorKindV2::NegativeFreeExtent(SpatialExtentV2::Width),
                ordinal,
                SpatialNodeFieldV2::FreeWidth,
            ));
        }
        if free.height() < 0 {
            return Err(node_input_error(
                SpatialInputErrorKindV2::NegativeFreeExtent(SpatialExtentV2::Height),
                ordinal,
                SpatialNodeFieldV2::FreeHeight,
            ));
        }
        if !free.offset().x().is_in_domain() {
            return Err(node_input_error(
                SpatialInputErrorKindV2::FreeOffsetOutOfDomain(SpatialAxisV2::X),
                ordinal,
                SpatialNodeFieldV2::FreeOffsetX,
            ));
        }
        if !free.offset().y().is_in_domain() {
            return Err(node_input_error(
                SpatialInputErrorKindV2::FreeOffsetOutOfDomain(SpatialAxisV2::Y),
                ordinal,
                SpatialNodeFieldV2::FreeOffsetY,
            ));
        }
    }

    Ok(PlacementInputProof { topology_limits })
}
