use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;
use fenestra_ui_layout::prototype::{
    LayoutAxisV1, LayoutDimensionV1, LayoutNodeV1, LayoutPaddingV1, LayoutStyleV1,
    ValidatedLayoutInputV1,
};

pub enum EngineResponse {
    Echo,
    Output(LayoutOutputV1),
    Error(LayoutEngineErrorV1),
}

pub struct ScriptedEngine {
    calls: AtomicUsize,
    expected_viewport: LayoutViewportV1,
    expected_nodes: Vec<LayoutNodeV1>,
    response: EngineResponse,
}

impl ScriptedEngine {
    pub fn new(
        expected_viewport: LayoutViewportV1,
        expected_nodes: Vec<LayoutNodeV1>,
        response: EngineResponse,
    ) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            expected_viewport,
            expected_nodes,
            response,
        }
    }

    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl LayoutEngineV1 for ScriptedEngine {
    fn compute(
        &self,
        input: ValidatedLayoutInputV1<'_>,
    ) -> Result<LayoutOutputV1, LayoutEngineErrorV1> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(input.viewport(), self.expected_viewport);
        assert_eq!(input.nodes(), self.expected_nodes);
        match &self.response {
            EngineResponse::Echo => Ok(output_for(input.nodes())),
            EngineResponse::Output(output) => Ok(output.clone()),
            EngineResponse::Error(error) => Err(*error),
        }
    }
}

pub const fn generous_limits() -> LayoutLimitsV1 {
    LayoutLimitsV1::new(64, 64, 64)
}

pub fn two_nodes() -> Vec<LayoutNodeV1> {
    vec![node(0, None, 0), node(1, Some(0), 0)]
}

pub fn negative_gap_nodes() -> Vec<LayoutNodeV1> {
    vec![node(0, None, -1)]
}

pub fn invalid_preorder_nodes() -> Vec<LayoutNodeV1> {
    vec![
        node(0, None, 0),
        node(1, Some(0), 0),
        node(2, Some(1), 0),
        node(3, Some(0), 0),
        node(4, Some(1), 0),
    ]
}

pub fn thirty_three_nodes() -> Vec<LayoutNodeV1> {
    let mut nodes = Vec::with_capacity(33);
    nodes.push(node(0, None, 0));
    for pair in 0..16_u32 {
        let child = pair * 2 + 1;
        nodes.push(node(child, Some(0), 0));
        nodes.push(node(child + 1, Some(child), 0));
    }
    nodes
}

pub fn output_for(nodes: &[LayoutNodeV1]) -> LayoutOutputV1 {
    LayoutOutputV1::new(
        nodes
            .iter()
            .map(|node| LayoutRecordV1::new(node.key(), LayoutRectV1::new(0, 0, 0, 0)))
            .collect(),
    )
}

pub const fn node(key: u32, parent: Option<u32>, gap: i32) -> LayoutNodeV1 {
    LayoutNodeV1::new(
        LayoutNodeKeyV1::new(key),
        match parent {
            Some(parent) => Some(LayoutNodeKeyV1::new(parent)),
            None => None,
        },
        LayoutStyleV1::new(
            LayoutAxisV1::Column,
            LayoutDimensionV1::new(0, 10, 10),
            LayoutDimensionV1::new(0, 10, 10),
            LayoutPaddingV1::new(0, 0, 0, 0),
            gap,
        ),
    )
}
