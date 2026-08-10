use crate::error::{LayoutErrorLocationV1, LayoutInputErrorKindV1};
use crate::limits::{LayoutLimitKindV1, LayoutLimitsV1};
use crate::model::{LayoutDimensionV1, LayoutInputV1, LayoutNodeV1};
use crate::vocabulary::{LayoutConstraintFieldV1, LayoutExtentV1, LayoutPaddingSideV1};

type InputValidationFailureV1 = (LayoutInputErrorKindV1, LayoutErrorLocationV1);

struct TopologyFactsV1 {
    depths: Vec<usize>,
    child_counts: Vec<usize>,
}

pub(crate) fn validate_input_v1(
    input: LayoutInputV1<'_>,
    limits: LayoutLimitsV1,
) -> Result<(), InputValidationFailureV1> {
    let nodes = input.nodes();
    validate_node_count(nodes, limits)?;
    let Some(root) = nodes.first().copied() else {
        return Err(input_failure(LayoutInputErrorKindV1::EmptyInput));
    };
    if root.key().get() != 0 {
        return Err(node_failure(LayoutInputErrorKindV1::InvalidRootKey, 0));
    }
    if root.parent().is_some() {
        return Err(node_failure(LayoutInputErrorKindV1::RootHasParent, 0));
    }

    let topology = validate_topology(nodes)?;
    validate_depths(&topology.depths, limits)?;
    validate_child_counts(&topology.child_counts, limits)?;
    validate_viewport(input)?;
    validate_constraints(nodes)?;
    validate_negative_padding(nodes)?;
    validate_padding_fit(nodes)?;
    validate_gaps(nodes)
}

fn validate_node_count(
    nodes: &[LayoutNodeV1],
    limits: LayoutLimitsV1,
) -> Result<(), InputValidationFailureV1> {
    if nodes.len() > limits.limit(LayoutLimitKindV1::Nodes)
        || nodes.len() > maximum_keyed_node_count()
    {
        return Err(input_failure(LayoutInputErrorKindV1::LimitExceeded(
            LayoutLimitKindV1::Nodes,
        )));
    }
    Ok(())
}

fn maximum_keyed_node_count() -> usize {
    let Ok(maximum_index) = usize::try_from(u32::MAX) else {
        return usize::MAX;
    };
    match maximum_index.checked_add(1) {
        Some(count) => count,
        None => usize::MAX,
    }
}

fn validate_topology(nodes: &[LayoutNodeV1]) -> Result<TopologyFactsV1, InputValidationFailureV1> {
    let mut active_ancestors = Vec::with_capacity(nodes.len());
    active_ancestors.push(0u32);
    let mut depths = Vec::with_capacity(nodes.len());
    depths.push(1usize);
    let mut child_counts = vec![0usize; nodes.len()];

    for (index, node) in nodes.iter().copied().enumerate().skip(1) {
        let ordinal = node_ordinal(index)?;
        if node.key().get() != ordinal {
            return Err(node_failure(LayoutInputErrorKindV1::NonDenseKey, index));
        }

        let Some(parent) = node.parent() else {
            return Err(node_failure(LayoutInputErrorKindV1::MissingParent, index));
        };
        let Ok(parent_index) = usize::try_from(parent.get()) else {
            return Err(node_failure(LayoutInputErrorKindV1::MissingParent, index));
        };
        let Some(parent_node) = nodes.get(parent_index) else {
            return Err(node_failure(LayoutInputErrorKindV1::MissingParent, index));
        };
        if parent_node.key() != parent {
            return Err(node_failure(LayoutInputErrorKindV1::MissingParent, index));
        }
        if parent_index >= index {
            return Err(node_failure(LayoutInputErrorKindV1::ForwardParent, index));
        }
        if !activate_parent(&mut active_ancestors, parent.get()) {
            return Err(node_failure(LayoutInputErrorKindV1::InvalidPreorder, index));
        }

        let Some(depth) = active_ancestors.len().checked_add(1) else {
            return Err(node_failure(
                LayoutInputErrorKindV1::LimitExceeded(LayoutLimitKindV1::Depth),
                index,
            ));
        };
        let Some(child_count) = child_counts.get_mut(parent_index) else {
            return Err(node_failure(LayoutInputErrorKindV1::MissingParent, index));
        };
        let Some(next_count) = child_count.checked_add(1) else {
            return Err(node_failure(
                LayoutInputErrorKindV1::LimitExceeded(LayoutLimitKindV1::ChildrenPerNode),
                parent_index,
            ));
        };
        *child_count = next_count;
        depths.push(depth);
        active_ancestors.push(ordinal);
    }

    Ok(TopologyFactsV1 {
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

fn validate_depths(
    depths: &[usize],
    limits: LayoutLimitsV1,
) -> Result<(), InputValidationFailureV1> {
    for (index, depth) in depths.iter().copied().enumerate() {
        if depth > limits.limit(LayoutLimitKindV1::Depth) {
            return Err(node_failure(
                LayoutInputErrorKindV1::LimitExceeded(LayoutLimitKindV1::Depth),
                index,
            ));
        }
    }
    Ok(())
}

fn validate_child_counts(
    child_counts: &[usize],
    limits: LayoutLimitsV1,
) -> Result<(), InputValidationFailureV1> {
    for (index, children) in child_counts.iter().copied().enumerate() {
        if children > limits.limit(LayoutLimitKindV1::ChildrenPerNode) {
            return Err(node_failure(
                LayoutInputErrorKindV1::LimitExceeded(LayoutLimitKindV1::ChildrenPerNode),
                index,
            ));
        }
    }
    Ok(())
}

fn validate_viewport(input: LayoutInputV1<'_>) -> Result<(), InputValidationFailureV1> {
    let viewport = input.viewport();
    if viewport.width() < 0 {
        return Err((
            LayoutInputErrorKindV1::NegativeViewport(LayoutExtentV1::Width),
            LayoutErrorLocationV1::Viewport,
        ));
    }
    if viewport.height() < 0 {
        return Err((
            LayoutInputErrorKindV1::NegativeViewport(LayoutExtentV1::Height),
            LayoutErrorLocationV1::Viewport,
        ));
    }
    Ok(())
}

fn validate_constraints(nodes: &[LayoutNodeV1]) -> Result<(), InputValidationFailureV1> {
    for (index, node) in nodes.iter().copied().enumerate() {
        validate_dimension(node.style().width(), LayoutExtentV1::Width, index)?;
        validate_dimension(node.style().height(), LayoutExtentV1::Height, index)?;
    }
    Ok(())
}

fn validate_dimension(
    dimension: LayoutDimensionV1,
    extent: LayoutExtentV1,
    index: usize,
) -> Result<(), InputValidationFailureV1> {
    for (field, value) in [
        (LayoutConstraintFieldV1::Minimum, dimension.minimum()),
        (LayoutConstraintFieldV1::Preferred, dimension.preferred()),
        (LayoutConstraintFieldV1::Maximum, dimension.maximum()),
    ] {
        if value < 0 {
            return Err(node_failure(
                LayoutInputErrorKindV1::NegativeConstraint { extent, field },
                index,
            ));
        }
    }
    if dimension.minimum() > dimension.maximum() {
        return Err(node_failure(
            LayoutInputErrorKindV1::InvertedConstraint(extent),
            index,
        ));
    }
    Ok(())
}

fn validate_negative_padding(nodes: &[LayoutNodeV1]) -> Result<(), InputValidationFailureV1> {
    for (index, node) in nodes.iter().copied().enumerate() {
        let padding = node.style().padding();
        for (side, value) in [
            (LayoutPaddingSideV1::Left, padding.left()),
            (LayoutPaddingSideV1::Right, padding.right()),
            (LayoutPaddingSideV1::Top, padding.top()),
            (LayoutPaddingSideV1::Bottom, padding.bottom()),
        ] {
            if value < 0 {
                return Err(node_failure(
                    LayoutInputErrorKindV1::NegativePadding(side),
                    index,
                ));
            }
        }
    }
    Ok(())
}

fn validate_padding_fit(nodes: &[LayoutNodeV1]) -> Result<(), InputValidationFailureV1> {
    for (index, node) in nodes.iter().copied().enumerate() {
        let style = node.style();
        let padding = style.padding();
        let horizontal = i64::from(padding.left()) + i64::from(padding.right());
        if horizontal > i64::from(style.width().resolved()) {
            return Err(node_failure(
                LayoutInputErrorKindV1::PaddingExceedsExtent(LayoutExtentV1::Width),
                index,
            ));
        }
        let vertical = i64::from(padding.top()) + i64::from(padding.bottom());
        if vertical > i64::from(style.height().resolved()) {
            return Err(node_failure(
                LayoutInputErrorKindV1::PaddingExceedsExtent(LayoutExtentV1::Height),
                index,
            ));
        }
    }
    Ok(())
}

fn validate_gaps(nodes: &[LayoutNodeV1]) -> Result<(), InputValidationFailureV1> {
    for (index, node) in nodes.iter().copied().enumerate() {
        if node.style().gap() < 0 {
            return Err(node_failure(LayoutInputErrorKindV1::NegativeGap, index));
        }
    }
    Ok(())
}

const fn input_failure(kind: LayoutInputErrorKindV1) -> InputValidationFailureV1 {
    (kind, LayoutErrorLocationV1::Input)
}

fn node_failure(kind: LayoutInputErrorKindV1, index: usize) -> InputValidationFailureV1 {
    match u32::try_from(index) {
        Ok(index) => (kind, LayoutErrorLocationV1::InputNode { index }),
        Err(_) => input_failure(LayoutInputErrorKindV1::LimitExceeded(
            LayoutLimitKindV1::Nodes,
        )),
    }
}

fn node_ordinal(index: usize) -> Result<u32, InputValidationFailureV1> {
    u32::try_from(index).map_err(|_| {
        input_failure(LayoutInputErrorKindV1::LimitExceeded(
            LayoutLimitKindV1::Nodes,
        ))
    })
}
