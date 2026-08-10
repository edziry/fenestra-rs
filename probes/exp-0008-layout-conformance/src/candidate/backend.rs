use fenestra_ui_layout::prototype::{LayoutErrorLocationV1, LayoutNodeV1, ValidatedLayoutInputV1};
use taffy::prelude::NodeId;

use super::conversion::CandidateRawRecordV1;
use super::error::invariant_error_v1;
use super::style::{map_taffy_available_space_v1, map_taffy_style_v1, new_taffy_tree_v1};

pub(crate) fn solve_taffy_layout_v1<F>(
    input: ValidatedLayoutInputV1<'_>,
    on_backend_entry: F,
) -> Result<Vec<CandidateRawRecordV1>, fenestra_ui_layout::prototype::LayoutEngineErrorV1>
where
    F: FnOnce(),
{
    on_backend_entry();
    let nodes = input.nodes();
    let children = collect_child_ordinals_v1(nodes);
    let mut tree = new_taffy_tree_v1();
    let mut candidate_ids = vec![None; nodes.len()];

    for index in (0..nodes.len()).rev() {
        let mut child_ids = Vec::with_capacity(children[index].len());
        for child_index in children[index].iter().copied() {
            let Some(child_id) = candidate_ids[child_index] else {
                return Err(invariant_error_v1(input_node_location(child_index)));
            };
            child_ids.push(child_id);
        }
        let candidate_id = tree
            .new_with_children(map_taffy_style_v1(nodes[index].style()), &child_ids)
            .map_err(|_| invariant_error_v1(input_node_location(index)))?;
        candidate_ids[index] = Some(candidate_id);
    }

    let Some(root) = candidate_ids.first().copied().flatten() else {
        return Err(invariant_error_v1(LayoutErrorLocationV1::Input));
    };
    tree.compute_layout(root, map_taffy_available_space_v1(input.viewport()))
        .map_err(|_| invariant_error_v1(LayoutErrorLocationV1::Input))?;

    collect_raw_records_v1(&tree, nodes, &candidate_ids)
}

fn collect_child_ordinals_v1(nodes: &[LayoutNodeV1]) -> Vec<Vec<usize>> {
    let mut children = vec![Vec::new(); nodes.len()];
    for (index, node) in nodes.iter().copied().enumerate().skip(1) {
        if let Some(parent) = node.parent() {
            children[parent.get() as usize].push(index);
        }
    }
    children
}

fn collect_raw_records_v1(
    tree: &taffy::prelude::TaffyTree<()>,
    nodes: &[LayoutNodeV1],
    candidate_ids: &[Option<NodeId>],
) -> Result<Vec<CandidateRawRecordV1>, fenestra_ui_layout::prototype::LayoutEngineErrorV1> {
    let mut records = Vec::with_capacity(nodes.len());
    for (index, node) in nodes.iter().copied().enumerate() {
        let Some(candidate_id) = candidate_ids[index] else {
            return Err(invariant_error_v1(output_record_location(index)));
        };
        let layout = tree
            .layout(candidate_id)
            .map_err(|_| invariant_error_v1(output_record_location(index)))?;
        records.push(CandidateRawRecordV1::new(
            node.key(),
            layout.location.x,
            layout.location.y,
            layout.size.width,
            layout.size.height,
        ));
    }
    Ok(records)
}

fn input_node_location(index: usize) -> LayoutErrorLocationV1 {
    match u32::try_from(index) {
        Ok(index) => LayoutErrorLocationV1::InputNode { index },
        Err(_) => LayoutErrorLocationV1::Input,
    }
}

fn output_record_location(index: usize) -> LayoutErrorLocationV1 {
    match u32::try_from(index) {
        Ok(index) => LayoutErrorLocationV1::OutputRecord { index },
        Err(_) => LayoutErrorLocationV1::Output,
    }
}
