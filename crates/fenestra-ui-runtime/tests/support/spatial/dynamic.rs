use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::NodeId;
use fenestra_ui_spatial::prototype::SpatialViewportV2;

use crate::support::headless::{COLOR, HEIGHT, WIDTH};
use crate::{RuntimeSpatialBuildViewV2, RuntimeSpatialInputV2, RuntimeSpatialProgramV2};

use super::input::layout_source;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DynamicBuildFact {
    pub viewport: SpatialViewportV2,
    pub nodes: Vec<NodeId>,
    pub widths: Vec<i32>,
    pub heights: Vec<i32>,
    pub colors: Vec<[u8; 4]>,
}

#[derive(Default)]
pub struct DynamicProgramState {
    calls: AtomicUsize,
    facts: Mutex<Vec<DynamicBuildFact>>,
}

impl DynamicProgramState {
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    pub fn facts(&self) -> Vec<DynamicBuildFact> {
        self.facts
            .lock()
            .expect("dynamic program facts should be available")
            .clone()
    }
}

pub struct DynamicProgram {
    state: Arc<DynamicProgramState>,
}

impl DynamicProgram {
    pub fn new() -> (Self, Arc<DynamicProgramState>) {
        let state = Arc::new(DynamicProgramState::default());
        (
            Self {
                state: Arc::clone(&state),
            },
            state,
        )
    }
}

impl RuntimeSpatialProgramV2 for DynamicProgram {
    fn build(
        &self,
        runtime: RuntimeSpatialBuildViewV2<'_>,
        viewport: SpatialViewportV2,
    ) -> RuntimeSpatialInputV2 {
        let mut nodes = Vec::new();
        collect_descendants(runtime, runtime.root(), &mut nodes);
        nodes.reverse();
        let widths = nodes
            .iter()
            .map(|&node| scalar(runtime, node, WIDTH, 1))
            .collect::<Vec<_>>();
        let heights = nodes
            .iter()
            .map(|&node| scalar(runtime, node, HEIGHT, 1))
            .collect::<Vec<_>>();
        let colors = nodes
            .iter()
            .map(|&node| rgba(runtime, node, COLOR))
            .collect::<Vec<_>>();
        let dimensions = widths
            .iter()
            .copied()
            .zip(heights.iter().copied())
            .collect::<Vec<_>>();
        self.state.calls.fetch_add(1, Ordering::SeqCst);
        self.state
            .facts
            .lock()
            .expect("dynamic program facts should be available")
            .push(DynamicBuildFact {
                viewport,
                nodes: nodes.clone(),
                widths,
                heights,
                colors,
            });
        RuntimeSpatialInputV2::new(
            layout_source(viewport, &dimensions),
            nodes.into_boxed_slice(),
        )
    }
}

fn collect_descendants(
    runtime: RuntimeSpatialBuildViewV2<'_>,
    parent: NodeId,
    output: &mut Vec<NodeId>,
) {
    for &child in runtime
        .children(parent)
        .expect("dynamic fixture parent should be live")
    {
        output.push(child);
        collect_descendants(runtime, child, output);
    }
}

fn scalar(
    runtime: RuntimeSpatialBuildViewV2<'_>,
    node: NodeId,
    property: fenestra_ui_ir::prototype::PropertyId,
    fallback: i32,
) -> i32 {
    match runtime.property(node, property) {
        Some(PropertyValue::ScalarI32(value)) => *value,
        Some(_) => panic!("dynamic fixture scalar should retain its type"),
        None => fallback,
    }
}

fn rgba(
    runtime: RuntimeSpatialBuildViewV2<'_>,
    node: NodeId,
    property: fenestra_ui_ir::prototype::PropertyId,
) -> [u8; 4] {
    match runtime.property(node, property) {
        Some(PropertyValue::Rgba8(value)) => *value,
        Some(_) => panic!("dynamic fixture color should retain its type"),
        None => [0; 4],
    }
}
