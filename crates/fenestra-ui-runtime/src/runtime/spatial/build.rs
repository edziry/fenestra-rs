use std::sync::Arc;

use fenestra_ui_layout::prototype::LayoutEngineV1;
use fenestra_ui_spatial::prototype::{
    SpatialLimitsV2, SpatialNodeKeyV2, SpatialOwnedInputV2, SpatialViewportV2, resolve_spatial_v2,
};

use super::error::RuntimeSpatialErrorV2;
use super::types::{RuntimeSpatialInputV2, SpatialPublication};
use crate::runtime::state::RuntimeState;

const SPATIAL_KEY_CAPACITY: u128 = u32::MAX as u128 + 1;

pub(super) fn build_publication(
    state: &RuntimeState,
    input: RuntimeSpatialInputV2,
    viewport: SpatialViewportV2,
    limits: SpatialLimitsV2,
    layout_engine: &dyn LayoutEngineV1,
) -> Result<SpatialPublication, RuntimeSpatialErrorV2> {
    let RuntimeSpatialInputV2 {
        source,
        logical_nodes,
    } = input;
    let topology = source.as_input().topology();
    if topology.viewport() != viewport {
        return Err(RuntimeSpatialErrorV2::ViewportMismatch);
    }

    let node_count = topology.nodes().len();
    if logical_nodes.len() != node_count.saturating_sub(1) {
        return Err(RuntimeSpatialErrorV2::MappingLengthMismatch);
    }
    if spatial_node_count_exceeds_key_capacity(node_count as u128) {
        return resolve_oversized(layout_engine, source, limits);
    }

    validate_mapping(state, &logical_nodes)?;
    let snapshot = resolve_spatial_v2(layout_engine, source, limits)
        .map_err(RuntimeSpatialErrorV2::Resolve)?;
    Ok(SpatialPublication {
        snapshot: Arc::new(snapshot),
        logical_nodes,
    })
}

fn validate_mapping(
    state: &RuntimeState,
    logical_nodes: &[crate::logical_tree::NodeId],
) -> Result<(), RuntimeSpatialErrorV2> {
    for (index, node) in logical_nodes.iter().copied().enumerate() {
        let key = SpatialNodeKeyV2::new(
            u32::try_from(index + 1).expect("representable spatial mapping key"),
        );
        if state.tree.value(node).is_none() {
            return Err(RuntimeSpatialErrorV2::MissingLogicalNode { key });
        }
        if logical_nodes[..index].contains(&node) {
            return Err(RuntimeSpatialErrorV2::DuplicateLogicalNode { key });
        }
    }
    Ok(())
}

fn resolve_oversized(
    layout_engine: &dyn LayoutEngineV1,
    source: Arc<SpatialOwnedInputV2>,
    limits: SpatialLimitsV2,
) -> Result<SpatialPublication, RuntimeSpatialErrorV2> {
    match resolve_spatial_v2(layout_engine, source, limits) {
        Err(error) => Err(RuntimeSpatialErrorV2::Resolve(error)),
        Ok(_) => unreachable!("oversized topology must fail direct-count validation"),
    }
}

const fn spatial_node_count_exceeds_key_capacity(node_count: u128) -> bool {
    node_count > SPATIAL_KEY_CAPACITY
}

#[cfg(test)]
mod tests {
    use super::spatial_node_count_exceeds_key_capacity;

    #[test]
    fn widened_spatial_node_count_boundary_precedes_key_formation() {
        let maximum = u32::MAX as u128 + 1;
        assert!(!spatial_node_count_exceeds_key_capacity(maximum));
        assert!(spatial_node_count_exceeds_key_capacity(maximum + 1));
    }
}
