//! Root and authored-preorder validation.

use super::{DirectCountProof, make_resolve_error};
use crate::error::{SpatialErrorLocationV2, SpatialInputErrorKindV2};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};
use crate::topology::SpatialPlacementV2;
use crate::vocabulary::SpatialNodeFieldV2;

pub(super) struct TopologyProof<'a> {
    direct_counts: DirectCountProof<'a>,
    depths: Vec<usize>,
    child_counts: Vec<usize>,
}

#[cfg(test)]
impl TopologyProof<'_> {
    pub(super) fn depths(&self) -> &[usize] {
        &self.depths
    }

    pub(super) fn child_counts(&self) -> &[usize] {
        &self.child_counts
    }
}

pub(super) fn prepare_topology(
    direct_counts: DirectCountProof<'_>,
) -> Result<TopologyProof<'_>, SpatialResolveErrorV2> {
    let nodes = direct_counts.input.topology().nodes();
    let Some(root) = nodes.first().copied() else {
        return Err(input_error(
            SpatialInputErrorKindV2::EmptyInput,
            SpatialErrorLocationV2::Input,
        ));
    };

    if root.key().get() != 0 {
        return Err(node_input_error(
            SpatialInputErrorKindV2::InvalidRootKey,
            0,
            SpatialNodeFieldV2::Key,
        ));
    }
    if root.parent().is_some() {
        return Err(node_input_error(
            SpatialInputErrorKindV2::RootHasParent,
            0,
            SpatialNodeFieldV2::Parent,
        ));
    }
    if !matches!(root.placement(), SpatialPlacementV2::Root) {
        return Err(node_input_error(
            SpatialInputErrorKindV2::InvalidRootPlacement,
            0,
            SpatialNodeFieldV2::Placement,
        ));
    }

    let mut active_ancestors = Vec::with_capacity(nodes.len());
    active_ancestors.push(0);
    let mut depths = Vec::with_capacity(nodes.len());
    depths.push(1);
    let mut child_counts = vec![0; nodes.len()];

    for (index, node) in nodes.iter().copied().enumerate().skip(1) {
        let ordinal = u32::try_from(index).expect("phase one validated the node row capacity");
        if node.key().get() != ordinal {
            return Err(node_input_error(
                SpatialInputErrorKindV2::NonDenseNodeKey,
                ordinal,
                SpatialNodeFieldV2::Key,
            ));
        }

        let Some(parent) = node.parent() else {
            return Err(node_input_error(
                SpatialInputErrorKindV2::MissingSpatialParent,
                ordinal,
                SpatialNodeFieldV2::Parent,
            ));
        };
        let Ok(parent_index) = usize::try_from(parent.get()) else {
            return Err(node_input_error(
                SpatialInputErrorKindV2::MissingSpatialParent,
                ordinal,
                SpatialNodeFieldV2::Parent,
            ));
        };
        let Some(parent_node) = nodes.get(parent_index) else {
            return Err(node_input_error(
                SpatialInputErrorKindV2::MissingSpatialParent,
                ordinal,
                SpatialNodeFieldV2::Parent,
            ));
        };
        if parent_node.key() != parent {
            return Err(node_input_error(
                SpatialInputErrorKindV2::MissingSpatialParent,
                ordinal,
                SpatialNodeFieldV2::Parent,
            ));
        }
        if parent_index >= index {
            return Err(node_input_error(
                SpatialInputErrorKindV2::ForwardSpatialParent,
                ordinal,
                SpatialNodeFieldV2::Parent,
            ));
        }
        if !activate_parent(&mut active_ancestors, parent.get()) {
            return Err(node_input_error(
                SpatialInputErrorKindV2::InvalidPreorder,
                ordinal,
                SpatialNodeFieldV2::Parent,
            ));
        }

        depths.push(active_ancestors.len() + 1);
        child_counts[parent_index] += 1;
        active_ancestors.push(ordinal);
    }

    Ok(TopologyProof {
        direct_counts,
        depths,
        child_counts,
    })
}

fn activate_parent(active_ancestors: &mut Vec<u32>, parent: u32) -> bool {
    loop {
        match active_ancestors.last().copied() {
            Some(active) if active == parent => return true,
            Some(_) => {
                let _ = active_ancestors.pop();
            }
            None => return false,
        }
    }
}

fn node_input_error(
    kind: SpatialInputErrorKindV2,
    index: u32,
    field: SpatialNodeFieldV2,
) -> SpatialResolveErrorV2 {
    input_error(kind, SpatialErrorLocationV2::NodeField { index, field })
}

fn input_error(
    kind: SpatialInputErrorKindV2,
    location: SpatialErrorLocationV2,
) -> SpatialResolveErrorV2 {
    make_resolve_error(SpatialResolveErrorKindV2::Input(kind), location)
}
