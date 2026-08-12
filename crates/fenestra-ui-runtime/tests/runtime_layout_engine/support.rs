use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use fenestra_ui_ir::prototype::ValidatedStyleProgram;
use fenestra_ui_layout::prototype::{
    LayoutAxisV1, LayoutDimensionV1, LayoutEngineErrorKindV1, LayoutEngineErrorV1, LayoutEngineV1,
    LayoutErrorLocationV1, LayoutNodeKeyV1, LayoutNodeV1, LayoutOutputV1, LayoutPaddingV1,
    LayoutRecordV1, LayoutRectV1, LayoutStyleV1, LayoutViewportV1, ReferenceStackEngineV1,
    ValidatedLayoutInputV1,
};
use fenestra_ui_runtime::prototype::{
    HeadlessProjectionSpec, HeadlessSurface, RuntimeCapacity, RuntimeInitializationError, UiRuntime,
};

use crate::support::headless::{exact_style, runtime_capacity};
use crate::support::headless_spec::{HeadlessSpecBuilder, surface};

pub const DISTINCT_BOUNDS: [LayoutRectV1; 5] = [
    LayoutRectV1::new(5, 4, 90, 70),
    LayoutRectV1::new(8, 7, 70, 50),
    LayoutRectV1::new(10, 10, 20, 8),
    LayoutRectV1::new(12, 22, 25, 9),
    LayoutRectV1::new(40, 22, 25, 9),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedLayoutInputV1 {
    pub viewport: LayoutViewportV1,
    pub nodes: Vec<LayoutNodeV1>,
}

#[derive(Default)]
pub struct SpyState {
    calls: AtomicUsize,
    inputs: Mutex<Vec<CapturedLayoutInputV1>>,
}

impl SpyState {
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    fn observe(&self, input: ValidatedLayoutInputV1<'_>) -> usize {
        let call_index = self.calls.fetch_add(1, Ordering::SeqCst);
        self.inputs
            .lock()
            .expect("spy capture should not be poisoned")
            .push(CapturedLayoutInputV1 {
                viewport: input.viewport(),
                nodes: input.nodes().to_vec(),
            });
        call_index
    }
}

pub struct SpyEngineV1 {
    state: Arc<SpyState>,
}

impl SpyEngineV1 {
    pub fn new(state: Arc<SpyState>) -> Self {
        Self { state }
    }
}

impl LayoutEngineV1 for SpyEngineV1 {
    fn compute(
        &self,
        input: ValidatedLayoutInputV1<'_>,
    ) -> Result<LayoutOutputV1, LayoutEngineErrorV1> {
        self.state.observe(input);
        ReferenceStackEngineV1::new().compute(input)
    }
}

pub struct DistinctBoundsEngineV1;

impl LayoutEngineV1 for DistinctBoundsEngineV1 {
    fn compute(
        &self,
        input: ValidatedLayoutInputV1<'_>,
    ) -> Result<LayoutOutputV1, LayoutEngineErrorV1> {
        if input.nodes().len() != DISTINCT_BOUNDS.len() {
            return Err(invariant_engine_error());
        }
        Ok(LayoutOutputV1::new(
            input
                .nodes()
                .iter()
                .zip(DISTINCT_BOUNDS)
                .map(|(node, bounds)| LayoutRecordV1::new(node.key(), bounds))
                .collect(),
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EngineFaultV1 {
    Engine(LayoutEngineErrorKindV1),
    RecordCount,
    Key,
    Negative,
    FarEdge,
}

pub struct ScriptedFaultEngineV1 {
    state: Arc<SpyState>,
    successful_calls: usize,
    fault: EngineFaultV1,
}

impl ScriptedFaultEngineV1 {
    pub fn new(state: Arc<SpyState>, successful_calls: usize, fault: EngineFaultV1) -> Self {
        Self {
            state,
            successful_calls,
            fault,
        }
    }
}

impl LayoutEngineV1 for ScriptedFaultEngineV1 {
    fn compute(
        &self,
        input: ValidatedLayoutInputV1<'_>,
    ) -> Result<LayoutOutputV1, LayoutEngineErrorV1> {
        let call_index = self.state.observe(input);
        if call_index < self.successful_calls {
            ReferenceStackEngineV1::new().compute(input)
        } else {
            fault_output(self.fault, input)
        }
    }
}

#[allow(clippy::result_large_err)]
pub fn try_runtime_with_engine(
    style: ValidatedStyleProgram,
    spec: HeadlessProjectionSpec,
    runtime_surface: HeadlessSurface,
    capacity: RuntimeCapacity,
    engine: Box<dyn LayoutEngineV1>,
) -> Result<UiRuntime, RuntimeInitializationError> {
    UiRuntime::new_headless_with_layout_engine(style, spec, runtime_surface, capacity, engine)
}

pub fn exact_runtime_with_engine(engine: Box<dyn LayoutEngineV1>) -> UiRuntime {
    try_exact_runtime_with_engine(engine).expect("injected headless runtime should initialize")
}

#[allow(clippy::result_large_err)]
pub fn try_exact_runtime_with_engine(
    engine: Box<dyn LayoutEngineV1>,
) -> Result<UiRuntime, RuntimeInitializationError> {
    try_runtime_with_engine(
        exact_style(),
        HeadlessSpecBuilder::new().build(),
        surface(),
        runtime_capacity(),
        engine,
    )
}

pub fn initialize_with_spy(style: ValidatedStyleProgram) -> (UiRuntime, Arc<SpyState>) {
    let state = Arc::new(SpyState::default());
    let runtime = try_runtime_with_engine(
        style,
        HeadlessSpecBuilder::new().build(),
        surface(),
        runtime_capacity(),
        Box::new(SpyEngineV1::new(Arc::clone(&state))),
    )
    .expect("injected headless runtime should initialize");
    (runtime, state)
}

pub fn only_input(state: &SpyState) -> CapturedLayoutInputV1 {
    assert_eq!(state.calls(), 1);
    let inputs = state
        .inputs
        .lock()
        .expect("spy capture should not be poisoned");
    assert_eq!(inputs.len(), 1);
    inputs[0].clone()
}

pub const fn dimension(preferred: i32, maximum: i32) -> LayoutDimensionV1 {
    LayoutDimensionV1::new(0, preferred, maximum)
}

pub const fn node(
    key: u32,
    parent: Option<u32>,
    width: LayoutDimensionV1,
    height: LayoutDimensionV1,
) -> LayoutNodeV1 {
    LayoutNodeV1::new(
        LayoutNodeKeyV1::new(key),
        match parent {
            Some(parent) => Some(LayoutNodeKeyV1::new(parent)),
            None => None,
        },
        LayoutStyleV1::new(
            LayoutAxisV1::Column,
            width,
            height,
            LayoutPaddingV1::new(0, 0, 0, 0),
            0,
        ),
    )
}

fn fault_output(
    fault: EngineFaultV1,
    input: ValidatedLayoutInputV1<'_>,
) -> Result<LayoutOutputV1, LayoutEngineErrorV1> {
    if let EngineFaultV1::Engine(kind) = fault {
        return Err(LayoutEngineErrorV1::new(kind, LayoutErrorLocationV1::Input));
    }
    let mut records = input
        .nodes()
        .iter()
        .map(|node| LayoutRecordV1::new(node.key(), LayoutRectV1::new(0, 0, 0, 0)))
        .collect::<Vec<_>>();
    match fault {
        EngineFaultV1::RecordCount => {
            let _ = records.pop();
        }
        EngineFaultV1::Key => {
            let record = records.first_mut().ok_or_else(invariant_engine_error)?;
            *record = LayoutRecordV1::new(
                LayoutNodeKeyV1::new(u32::MAX),
                LayoutRectV1::new(0, 0, 0, 0),
            );
        }
        EngineFaultV1::Negative => {
            let record = records.first_mut().ok_or_else(invariant_engine_error)?;
            *record = LayoutRecordV1::new(record.key(), LayoutRectV1::new(-1, 0, 0, 0));
        }
        EngineFaultV1::FarEdge => {
            let record = records.first_mut().ok_or_else(invariant_engine_error)?;
            *record = LayoutRecordV1::new(record.key(), LayoutRectV1::new(i32::MAX, 0, 1, 0));
        }
        EngineFaultV1::Engine(_) => return Err(invariant_engine_error()),
    }
    Ok(LayoutOutputV1::new(records))
}

const fn invariant_engine_error() -> LayoutEngineErrorV1 {
    LayoutEngineErrorV1::new(
        LayoutEngineErrorKindV1::InvariantViolation,
        LayoutErrorLocationV1::Input,
    )
}
