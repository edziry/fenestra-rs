use std::panic::panic_any;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use fenestra_ui_layout::prototype::{
    LayoutEngineErrorKindV1, LayoutEngineErrorV1, LayoutEngineV1, LayoutErrorLocationV1,
    LayoutNodeKeyV1, LayoutOutputV1, LayoutRecordV1, LayoutRectV1, ReferenceStackEngineV1,
    ValidatedLayoutInputV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EngineMarker;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayoutInputFact {
    pub viewport: (i32, i32),
    pub nodes: Vec<(u32, Option<u32>, i32, i32)>,
}

#[derive(Default)]
pub struct EngineState {
    calls: AtomicUsize,
    inputs: Mutex<Vec<LayoutInputFact>>,
    trace: Option<Arc<Mutex<Vec<&'static str>>>>,
}

impl EngineState {
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    pub fn only_input(&self) -> LayoutInputFact {
        let inputs = self
            .inputs
            .lock()
            .expect("engine inputs should be available");
        assert_eq!(inputs.len(), 1);
        inputs[0].clone()
    }

    fn record(&self, input: ValidatedLayoutInputV1<'_>) {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if let Some(trace) = &self.trace {
            trace
                .lock()
                .expect("callback trace should be available")
                .push("engine");
        }
        self.inputs
            .lock()
            .expect("engine inputs should be available")
            .push(LayoutInputFact {
                viewport: (input.viewport().width(), input.viewport().height()),
                nodes: input
                    .nodes()
                    .iter()
                    .copied()
                    .map(|node| {
                        (
                            node.key().get(),
                            node.parent().map(LayoutNodeKeyV1::get),
                            node.style().width().preferred(),
                            node.style().height().preferred(),
                        )
                    })
                    .collect(),
            });
    }
}

#[derive(Clone, Copy)]
pub enum EnginePlan {
    Reference,
    Distinct,
    Reject,
    RejectOnCall(usize),
    Panic,
    PanicOnCall(usize),
}

pub struct EngineSpy {
    state: Arc<EngineState>,
    plan: EnginePlan,
    drops: Option<Arc<AtomicUsize>>,
}

impl EngineSpy {
    pub fn new(plan: EnginePlan) -> (Self, Arc<EngineState>) {
        Self::with_drop_probe(plan, None)
    }

    pub fn with_drops(plan: EnginePlan, drops: Arc<AtomicUsize>) -> (Self, Arc<EngineState>) {
        Self::with_drop_probe(plan, Some(drops))
    }

    pub fn with_trace(
        plan: EnginePlan,
        trace: Arc<Mutex<Vec<&'static str>>>,
    ) -> (Self, Arc<EngineState>) {
        let state = Arc::new(EngineState {
            calls: AtomicUsize::new(0),
            inputs: Mutex::new(Vec::new()),
            trace: Some(trace),
        });
        (
            Self {
                state: Arc::clone(&state),
                plan,
                drops: None,
            },
            state,
        )
    }

    fn with_drop_probe(
        plan: EnginePlan,
        drops: Option<Arc<AtomicUsize>>,
    ) -> (Self, Arc<EngineState>) {
        let state = Arc::new(EngineState::default());
        (
            Self {
                state: Arc::clone(&state),
                plan,
                drops,
            },
            state,
        )
    }
}

impl LayoutEngineV1 for EngineSpy {
    fn compute(
        &self,
        input: ValidatedLayoutInputV1<'_>,
    ) -> Result<LayoutOutputV1, LayoutEngineErrorV1> {
        self.state.record(input);
        match self.plan {
            EnginePlan::Reference => ReferenceStackEngineV1::new().compute(input),
            EnginePlan::Distinct => Ok(distinct_output(input)),
            EnginePlan::Reject => Err(rejection()),
            EnginePlan::RejectOnCall(call) if self.state.calls() == call => Err(rejection()),
            EnginePlan::RejectOnCall(_) => ReferenceStackEngineV1::new().compute(input),
            EnginePlan::Panic => panic_any(EngineMarker),
            EnginePlan::PanicOnCall(call) if self.state.calls() == call => panic_any(EngineMarker),
            EnginePlan::PanicOnCall(_) => ReferenceStackEngineV1::new().compute(input),
        }
    }
}

fn rejection() -> LayoutEngineErrorV1 {
    LayoutEngineErrorV1::new(
        LayoutEngineErrorKindV1::RejectedInput,
        LayoutErrorLocationV1::InputNode { index: 2 },
    )
}

impl Drop for EngineSpy {
    fn drop(&mut self) {
        if let Some(drops) = &self.drops {
            drops.fetch_add(1, Ordering::SeqCst);
        }
    }
}

fn distinct_output(input: ValidatedLayoutInputV1<'_>) -> LayoutOutputV1 {
    let viewport = input.viewport();
    LayoutOutputV1::new(
        input
            .nodes()
            .iter()
            .copied()
            .enumerate()
            .map(|(index, node)| {
                let bounds = if index == 0 {
                    LayoutRectV1::new(0, 0, viewport.width(), viewport.height())
                } else {
                    LayoutRectV1::new(
                        i32::try_from(index * 3).expect("fixture x should fit"),
                        i32::try_from(index * 4).expect("fixture y should fit"),
                        node.style().width().preferred(),
                        node.style().height().preferred(),
                    )
                };
                LayoutRecordV1::new(node.key(), bounds)
            })
            .collect(),
    )
}
