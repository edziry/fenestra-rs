use crate::engine::ValidatedLayoutInputV1;
use crate::error::{LayoutEngineErrorKindV1, LayoutEngineErrorV1, LayoutErrorLocationV1};
use crate::model::{LayoutNodeV1, LayoutOutputV1, LayoutRecordV1, LayoutRectV1};
use crate::vocabulary::{LayoutArithmeticOperationV1, LayoutAxisV1, LayoutExtentV1};

pub(crate) fn compute_reference_stack_v1(
    input: ValidatedLayoutInputV1<'_>,
) -> Result<LayoutOutputV1, LayoutEngineErrorV1> {
    let nodes = input.nodes();
    let Some(root) = nodes.first().copied() else {
        return Err(invariant_failure(LayoutErrorLocationV1::Input));
    };

    let root_location = input_node_location(0);
    let root_width = root.style().width().resolved();
    let root_height = root.style().height().resolved();
    checked_add(
        0,
        root_width,
        LayoutArithmeticOperationV1::FarEdge,
        LayoutExtentV1::Width,
        root_location,
    )?;
    checked_add(
        0,
        root_height,
        LayoutArithmeticOperationV1::FarEdge,
        LayoutExtentV1::Height,
        root_location,
    )?;

    let children = index_children(nodes)?;
    let mut placements = vec![None; nodes.len()];
    let Some(root_placement) = placements.first_mut() else {
        return Err(invariant_failure(LayoutErrorLocationV1::Input));
    };
    *root_placement = Some(LayoutRectV1::new(0, 0, root_width, root_height));

    for (parent_index, direct_children) in children.iter().enumerate() {
        if direct_children.is_empty() {
            continue;
        }
        place_children(nodes, &mut placements, parent_index, direct_children)?;
    }

    materialize_output(nodes, placements)
}

fn index_children(nodes: &[LayoutNodeV1]) -> Result<Vec<Vec<usize>>, LayoutEngineErrorV1> {
    let mut children = Vec::with_capacity(nodes.len());
    children.resize_with(nodes.len(), Vec::new);

    for (index, node) in nodes.iter().copied().enumerate().skip(1) {
        let location = input_node_location(index);
        let Some(parent) = node.parent() else {
            return Err(invariant_failure(location));
        };
        let Ok(parent_index) = usize::try_from(parent.get()) else {
            return Err(invariant_failure(location));
        };
        let Some(direct_children) = children.get_mut(parent_index) else {
            return Err(invariant_failure(location));
        };
        direct_children.push(index);
    }

    Ok(children)
}

fn place_children(
    nodes: &[LayoutNodeV1],
    placements: &mut [Option<LayoutRectV1>],
    parent_index: usize,
    direct_children: &[usize],
) -> Result<(), LayoutEngineErrorV1> {
    let Some(parent) = nodes.get(parent_index).copied() else {
        return Err(invariant_failure(LayoutErrorLocationV1::Input));
    };
    let parent_location = input_node_location(parent_index);
    let Some(parent_bounds) = placements.get(parent_index).copied().flatten() else {
        return Err(invariant_failure(parent_location));
    };
    let style = parent.style();
    let padding = style.padding();
    let content_x = checked_add(
        parent_bounds.x(),
        padding.left(),
        LayoutArithmeticOperationV1::ContentOrigin,
        LayoutExtentV1::Width,
        parent_location,
    )?;
    let content_y = checked_add(
        parent_bounds.y(),
        padding.top(),
        LayoutArithmeticOperationV1::ContentOrigin,
        LayoutExtentV1::Height,
        parent_location,
    )?;
    let mut cursor = match style.axis() {
        LayoutAxisV1::Row => content_x,
        LayoutAxisV1::Column => content_y,
    };

    for (child_position, child_index) in direct_children.iter().copied().enumerate() {
        let Some(child) = nodes.get(child_index).copied() else {
            return Err(invariant_failure(parent_location));
        };
        let child_location = input_node_location(child_index);
        let (x, y) = child_origin(style.axis(), cursor, content_x, content_y);
        let width = child.style().width().resolved();
        let height = child.style().height().resolved();
        let far_x = checked_add(
            x,
            width,
            LayoutArithmeticOperationV1::FarEdge,
            LayoutExtentV1::Width,
            child_location,
        )?;
        let far_y = checked_add(
            y,
            height,
            LayoutArithmeticOperationV1::FarEdge,
            LayoutExtentV1::Height,
            child_location,
        )?;
        let bounds = LayoutRectV1::new(x, y, width, height);
        let Some(child_placement) = placements.get_mut(child_index) else {
            return Err(invariant_failure(child_location));
        };
        if child_placement.replace(bounds).is_some() {
            return Err(invariant_failure(child_location));
        }

        cursor = match style.axis() {
            LayoutAxisV1::Row => far_x,
            LayoutAxisV1::Column => far_y,
        };
        if child_position + 1 < direct_children.len() {
            cursor = checked_add(
                cursor,
                style.gap(),
                LayoutArithmeticOperationV1::GapAdvance,
                main_extent(style.axis()),
                parent_location,
            )?;
        }
    }

    Ok(())
}

const fn child_origin(
    axis: LayoutAxisV1,
    cursor: i32,
    content_x: i32,
    content_y: i32,
) -> (i32, i32) {
    match axis {
        LayoutAxisV1::Row => (cursor, content_y),
        LayoutAxisV1::Column => (content_x, cursor),
    }
}

const fn main_extent(axis: LayoutAxisV1) -> LayoutExtentV1 {
    match axis {
        LayoutAxisV1::Row => LayoutExtentV1::Width,
        LayoutAxisV1::Column => LayoutExtentV1::Height,
    }
}

fn materialize_output(
    nodes: &[LayoutNodeV1],
    placements: Vec<Option<LayoutRectV1>>,
) -> Result<LayoutOutputV1, LayoutEngineErrorV1> {
    let mut records = Vec::with_capacity(nodes.len());
    for (index, (node, placement)) in nodes.iter().copied().zip(placements).enumerate() {
        let Some(bounds) = placement else {
            return Err(invariant_failure(input_node_location(index)));
        };
        records.push(LayoutRecordV1::new(node.key(), bounds));
    }
    Ok(LayoutOutputV1::new(records))
}

fn checked_add(
    left: i32,
    right: i32,
    operation: LayoutArithmeticOperationV1,
    extent: LayoutExtentV1,
    location: LayoutErrorLocationV1,
) -> Result<i32, LayoutEngineErrorV1> {
    left.checked_add(right).ok_or_else(|| {
        LayoutEngineErrorV1::new(
            LayoutEngineErrorKindV1::ArithmeticExhausted { operation, extent },
            location,
        )
    })
}

fn input_node_location(index: usize) -> LayoutErrorLocationV1 {
    match u32::try_from(index) {
        Ok(index) => LayoutErrorLocationV1::InputNode { index },
        Err(_) => LayoutErrorLocationV1::Input,
    }
}

const fn invariant_failure(location: LayoutErrorLocationV1) -> LayoutEngineErrorV1 {
    LayoutEngineErrorV1::new(LayoutEngineErrorKindV1::InvariantViolation, location)
}
