use std::panic::panic_any;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use fenestra_ui_runtime::prototype::NodeId;
use fenestra_ui_spatial::prototype::{SpatialOwnedInputV2, SpatialViewportV2};

use crate::{RuntimeSpatialBuildViewV2, RuntimeSpatialInputV2, RuntimeSpatialProgramV2};

use super::facts::{BuildFacts, ForeignIds, LogicalNodes};
use super::input::{canonical_source, free_source};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProgramMarker;

#[derive(Default)]
pub struct ProgramState {
    calls: AtomicUsize,
    facts: Mutex<Vec<BuildFacts>>,
}

impl ProgramState {
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    pub fn only_facts(&self) -> BuildFacts {
        let facts = self
            .facts
            .lock()
            .expect("program facts should be available");
        assert_eq!(facts.len(), 1);
        facts[0].clone()
    }

    fn record(&self, facts: BuildFacts) {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.facts
            .lock()
            .expect("program facts should be available")
            .push(facts);
    }
}

pub enum SourcePlan {
    Canonical,
    Free,
    Exact(Arc<SpatialOwnedInputV2>),
    Panic,
}

#[derive(Clone, Copy)]
pub enum MappingPlan {
    Canonical,
    Free,
    Empty,
    ForeignOnly(NodeId),
    MissingSecond(NodeId),
    DuplicateSecond,
}

pub struct ProgramSpy {
    state: Arc<ProgramState>,
    source: SourcePlan,
    mapping: MappingPlan,
    foreign: Option<ForeignIds>,
    drops: Option<Arc<AtomicUsize>>,
}

impl ProgramSpy {
    pub fn new(source: SourcePlan, mapping: MappingPlan) -> (Self, Arc<ProgramState>) {
        Self::with_options(source, mapping, None, None)
    }

    pub fn with_foreign(
        source: SourcePlan,
        mapping: MappingPlan,
        foreign: ForeignIds,
    ) -> (Self, Arc<ProgramState>) {
        Self::with_options(source, mapping, Some(foreign), None)
    }

    pub fn with_drop_probe(
        source: SourcePlan,
        mapping: MappingPlan,
        drops: Arc<AtomicUsize>,
    ) -> (Self, Arc<ProgramState>) {
        Self::with_options(source, mapping, None, Some(drops))
    }

    fn with_options(
        source: SourcePlan,
        mapping: MappingPlan,
        foreign: Option<ForeignIds>,
        drops: Option<Arc<AtomicUsize>>,
    ) -> (Self, Arc<ProgramState>) {
        let state = Arc::new(ProgramState::default());
        (
            Self {
                state: Arc::clone(&state),
                source,
                mapping,
                foreign,
                drops,
            },
            state,
        )
    }
}

impl RuntimeSpatialProgramV2 for ProgramSpy {
    fn build(
        &self,
        runtime: RuntimeSpatialBuildViewV2<'_>,
        viewport: SpatialViewportV2,
    ) -> RuntimeSpatialInputV2 {
        let facts = BuildFacts::capture(runtime, viewport, self.foreign);
        let nodes = facts.nodes;
        self.state.record(facts);

        let source = match &self.source {
            SourcePlan::Canonical => canonical_source(viewport),
            SourcePlan::Free => free_source(viewport),
            SourcePlan::Exact(source) => Arc::clone(source),
            SourcePlan::Panic => panic_any(ProgramMarker),
        };
        RuntimeSpatialInputV2::new(source, mapping(self.mapping, nodes))
    }
}

impl Drop for ProgramSpy {
    fn drop(&mut self) {
        if let Some(drops) = &self.drops {
            drops.fetch_add(1, Ordering::SeqCst);
        }
    }
}

fn mapping(plan: MappingPlan, nodes: LogicalNodes) -> Box<[NodeId]> {
    match plan {
        MappingPlan::Canonical => vec![
            nodes.second_item,
            nodes.container,
            nodes.first_item,
            nodes.control,
        ],
        MappingPlan::Free => vec![nodes.container],
        MappingPlan::Empty => Vec::new(),
        MappingPlan::ForeignOnly(foreign) => vec![foreign],
        MappingPlan::MissingSecond(foreign) => {
            vec![nodes.control, foreign, nodes.control]
        }
        MappingPlan::DuplicateSecond => {
            vec![nodes.control, nodes.control, nodes.first_item]
        }
    }
    .into_boxed_slice()
}
