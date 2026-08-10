use std::collections::HashSet;

use fenestra_ui_layout::prototype::{
    LayoutAxisV1, LayoutDimensionV1, LayoutEngineErrorKindV1, LayoutErrorKindV1, LayoutErrorV1,
    LayoutInputV1, LayoutLimitsV1, LayoutNodeKeyV1, LayoutNodeV1, LayoutPaddingV1, LayoutStyleV1,
    LayoutViewportV1, compute_layout_v1,
};

use super::build::scalar_property;
use super::types::{HeadlessProjectionFailure, HeadlessRuntimeConfig};
use super::{HeadlessProjectionErrorKind, HeadlessRect, HeadlessSurface};
use crate::logical_tree::NodeId;
use crate::runtime::state::RuntimeState;

pub(super) struct LayoutPlacement {
    pub(super) node: NodeId,
    pub(super) parent_index: Option<usize>,
    pub(super) bounds: HeadlessRect,
}

struct PendingNode {
    node: NodeId,
    parent_index: Option<usize>,
    maximum_width: Option<i32>,
}

struct MappedLayoutInput {
    runtime_nodes: Vec<NodeId>,
    parent_indices: Vec<Option<usize>>,
    layout_nodes: Vec<LayoutNodeV1>,
}

pub(super) fn compute_layout(
    state: &RuntimeState,
    config: &HeadlessRuntimeConfig,
    surface: HeadlessSurface,
) -> Result<Vec<LayoutPlacement>, HeadlessProjectionFailure> {
    let mapped = map_layout_input(state, config)?;
    let node_count = mapped.layout_nodes.len();
    let input = LayoutInputV1::new(
        LayoutViewportV1::new(surface.width(), surface.height()),
        &mapped.layout_nodes,
    );
    let output = compute_layout_v1(
        config.layout_engine.as_ref(),
        input,
        LayoutLimitsV1::new(node_count, node_count, node_count),
    )
    .map_err(layout_failure)?;
    let records = output.records();
    if records.len() != mapped.runtime_nodes.len()
        || mapped.parent_indices.len() != mapped.runtime_nodes.len()
    {
        return Err(invariant());
    }

    Ok(mapped
        .runtime_nodes
        .into_iter()
        .zip(mapped.parent_indices)
        .zip(records.iter().copied())
        .map(|((node, parent_index), record)| {
            let bounds = record.bounds();
            LayoutPlacement {
                node,
                parent_index,
                bounds: HeadlessRect::new(bounds.x(), bounds.y(), bounds.width(), bounds.height()),
            }
        })
        .collect())
}

fn map_layout_input(
    state: &RuntimeState,
    config: &HeadlessRuntimeConfig,
) -> Result<MappedLayoutInput, HeadlessProjectionFailure> {
    let root = state.tree.root().ok_or_else(invariant)?;
    let node_count = state.tree.len();
    let mut runtime_nodes = Vec::with_capacity(node_count);
    let mut parent_indices = Vec::with_capacity(node_count);
    let mut layout_nodes = Vec::with_capacity(node_count);
    let mut seen = HashSet::with_capacity(node_count);
    let mut pending = vec![PendingNode {
        node: root,
        parent_index: None,
        maximum_width: None,
    }];

    while let Some(pending_node) = pending.pop() {
        if !seen.insert(pending_node.node) {
            return Err(invariant());
        }
        let index = layout_nodes.len();
        let key = layout_key(index)?;
        let expected_parent = match pending_node.parent_index {
            Some(parent_index) => runtime_nodes.get(parent_index).copied(),
            None => None,
        };
        if state.tree.parent(pending_node.node) != expected_parent {
            return Err(invariant());
        }
        let stored = state.tree.value(pending_node.node).ok_or_else(invariant)?;
        let width = scalar_property(stored, config.spec.width())?;
        let height = scalar_property(stored, config.spec.height())?;
        let maximum_width = pending_node.maximum_width.unwrap_or(width);
        let resolved_width = width.min(maximum_width);
        let parent_key = match pending_node.parent_index {
            Some(parent_index) => Some(layout_key(parent_index)?),
            None => None,
        };
        layout_nodes.push(LayoutNodeV1::new(
            key,
            parent_key,
            LayoutStyleV1::new(
                LayoutAxisV1::Column,
                LayoutDimensionV1::new(0, width, maximum_width),
                LayoutDimensionV1::new(0, height, height),
                LayoutPaddingV1::new(0, 0, 0, 0),
                0,
            ),
        ));
        runtime_nodes.push(pending_node.node);
        parent_indices.push(pending_node.parent_index);

        let children = state
            .tree
            .children(pending_node.node)
            .ok_or_else(invariant)?;
        for child in children.iter().rev() {
            pending.push(PendingNode {
                node: *child,
                parent_index: Some(index),
                maximum_width: Some(resolved_width),
            });
        }
    }

    if seen.len() != node_count {
        return Err(invariant());
    }
    Ok(MappedLayoutInput {
        runtime_nodes,
        parent_indices,
        layout_nodes,
    })
}

fn layout_key(index: usize) -> Result<LayoutNodeKeyV1, HeadlessProjectionFailure> {
    u32::try_from(index)
        .map(LayoutNodeKeyV1::new)
        .map_err(|_| invariant())
}

const fn layout_failure(error: LayoutErrorV1) -> HeadlessProjectionFailure {
    match error.kind() {
        LayoutErrorKindV1::Engine(LayoutEngineErrorKindV1::ArithmeticExhausted { .. }) => {
            HeadlessProjectionFailure::new(HeadlessProjectionErrorKind::ArithmeticExhausted)
        }
        _ => invariant(),
    }
}

const fn invariant() -> HeadlessProjectionFailure {
    HeadlessProjectionFailure::new(HeadlessProjectionErrorKind::InvariantViolation)
}
