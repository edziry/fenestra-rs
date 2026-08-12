use std::panic::panic_any;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};

use fenestra_ui_runtime::prototype::NodeId;
use fenestra_ui_spatial::prototype::{SpatialOwnedInputV2, SpatialViewportV2};

use crate::{RuntimeSpatialBuildViewV2, RuntimeSpatialInputV2, RuntimeSpatialProgramV2};

use super::facts::{BuildFacts, ForeignIds, LogicalNodes};
use super::input::{canonical_source, free_source, malformed_source, three_node_source};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProgramMarker;

#[derive(Default)]
pub struct ProgramState {
    calls: AtomicUsize,
    facts: Mutex<Vec<BuildFacts>>,
    sources: Mutex<Vec<Weak<SpatialOwnedInputV2>>>,
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

    pub fn facts(&self) -> Vec<BuildFacts> {
        self.facts
            .lock()
            .expect("program facts should be available")
            .clone()
    }

    pub fn source_weaks(&self) -> Vec<Weak<SpatialOwnedInputV2>> {
        self.sources
            .lock()
            .expect("program sources should be available")
            .clone()
    }

    fn record(&self, facts: BuildFacts) {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.facts
            .lock()
            .expect("program facts should be available")
            .push(facts);
    }

    fn record_source(&self, source: &Arc<SpatialOwnedInputV2>) {
        self.sources
            .lock()
            .expect("program sources should be available")
            .push(Arc::downgrade(source));
    }
}

pub enum SourcePlan {
    Canonical,
    FreshCanonical,
    Free,
    Exact(Arc<SpatialOwnedInputV2>),
    MalformedCanonicalOnCall(usize),
    MalformedThreeOnCall(usize),
    Panic,
    PanicOnCall(usize),
}

#[derive(Clone, Copy)]
pub enum MappingPlan {
    Canonical,
    Free,
    Empty,
    ForeignOnly(NodeId),
    MissingSecond(NodeId),
    DuplicateSecond,
    ForeignOnlyOnCall(usize, NodeId),
    MissingSecondOnCall(usize, NodeId),
    DuplicateSecondOnCall(usize),
}

pub struct ProgramSpy {
    state: Arc<ProgramState>,
    source: SourcePlan,
    mapping: MappingPlan,
    foreign: Option<ForeignIds>,
    drops: Option<Arc<AtomicUsize>>,
    trace: Option<Arc<Mutex<Vec<&'static str>>>>,
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

    pub fn with_trace(
        source: SourcePlan,
        mapping: MappingPlan,
        trace: Arc<Mutex<Vec<&'static str>>>,
    ) -> (Self, Arc<ProgramState>) {
        let (mut program, state) = Self::with_options(source, mapping, None, None);
        program.trace = Some(trace);
        (program, state)
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
                trace: None,
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
        if let Some(trace) = &self.trace {
            trace
                .lock()
                .expect("callback trace should be available")
                .push("program");
        }
        let nodes = facts.nodes;
        self.state.record(facts);
        let call = self.state.calls();

        if matches!(&self.source, SourcePlan::PanicOnCall(panic_call) if call == *panic_call) {
            panic_any(ProgramMarker);
        }

        let source = match &self.source {
            SourcePlan::Canonical => canonical_source(viewport),
            SourcePlan::FreshCanonical => {
                let source = canonical_source(viewport);
                self.state.record_source(&source);
                source
            }
            SourcePlan::Free => free_source(viewport),
            SourcePlan::Exact(source) => Arc::clone(source),
            SourcePlan::MalformedCanonicalOnCall(malformed_call) if call == *malformed_call => {
                malformed_source(viewport)
            }
            SourcePlan::MalformedCanonicalOnCall(_) => canonical_source(viewport),
            SourcePlan::MalformedThreeOnCall(malformed_call) if call == *malformed_call => {
                three_node_source(viewport, true)
            }
            SourcePlan::MalformedThreeOnCall(_) => canonical_source(viewport),
            SourcePlan::Panic => panic_any(ProgramMarker),
            SourcePlan::PanicOnCall(_) => canonical_source(viewport),
        };
        RuntimeSpatialInputV2::new(source, mapping(self.mapping, nodes, call))
    }
}

impl Drop for ProgramSpy {
    fn drop(&mut self) {
        if let Some(drops) = &self.drops {
            drops.fetch_add(1, Ordering::SeqCst);
        }
    }
}

fn mapping(plan: MappingPlan, nodes: LogicalNodes, call: usize) -> Box<[NodeId]> {
    match plan {
        MappingPlan::Canonical => canonical_mapping(nodes),
        MappingPlan::Free => vec![nodes.container],
        MappingPlan::Empty => Vec::new(),
        MappingPlan::ForeignOnly(foreign) => vec![foreign],
        MappingPlan::MissingSecond(foreign) => {
            vec![nodes.control, foreign, nodes.control]
        }
        MappingPlan::DuplicateSecond => {
            vec![nodes.control, nodes.control, nodes.first_item]
        }
        MappingPlan::ForeignOnlyOnCall(fault_call, foreign) if call == fault_call => {
            vec![foreign]
        }
        MappingPlan::MissingSecondOnCall(fault_call, foreign) if call == fault_call => {
            vec![nodes.control, foreign, nodes.control]
        }
        MappingPlan::DuplicateSecondOnCall(fault_call) if call == fault_call => {
            vec![nodes.control, nodes.control, nodes.first_item]
        }
        MappingPlan::ForeignOnlyOnCall(_, _)
        | MappingPlan::MissingSecondOnCall(_, _)
        | MappingPlan::DuplicateSecondOnCall(_) => canonical_mapping(nodes),
    }
    .into_boxed_slice()
}

fn canonical_mapping(nodes: LogicalNodes) -> Vec<NodeId> {
    vec![
        nodes.second_item,
        nodes.container,
        nodes.first_item,
        nodes.control,
    ]
}
