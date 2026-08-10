use std::sync::atomic::{AtomicUsize, Ordering};

use fenestra_ui_layout::prototype::{
    LayoutAxisV1, LayoutDimensionV1, LayoutEngineErrorV1, LayoutEngineV1, LayoutErrorKindV1,
    LayoutErrorLocationV1, LayoutErrorV1, LayoutInputV1, LayoutLimitsV1, LayoutNodeKeyV1,
    LayoutNodeV1, LayoutOutputErrorKindV1, LayoutOutputV1, LayoutPaddingV1, LayoutRecordV1,
    LayoutRectV1, LayoutStyleV1, LayoutViewportV1, ValidatedLayoutInputV1, compute_layout_v1,
};

const VIEWPORT: LayoutViewportV1 = LayoutViewportV1::new(100, 80);
const LIMITS: LayoutLimitsV1 = LayoutLimitsV1::new(2, 2, 1);
const DIMENSION: LayoutDimensionV1 = LayoutDimensionV1::new(0, 20, 20);
const PADDING: LayoutPaddingV1 = LayoutPaddingV1::new(0, 0, 0, 0);
const STYLE: LayoutStyleV1 =
    LayoutStyleV1::new(LayoutAxisV1::Column, DIMENSION, DIMENSION, PADDING, 0);
const NODES: [LayoutNodeV1; 2] = [
    LayoutNodeV1::new(LayoutNodeKeyV1::new(0), None, STYLE),
    LayoutNodeV1::new(
        LayoutNodeKeyV1::new(1),
        Some(LayoutNodeKeyV1::new(0)),
        STYLE,
    ),
];

pub struct FakeOutputEngine {
    calls: AtomicUsize,
    result: Result<LayoutOutputV1, LayoutEngineErrorV1>,
}

impl FakeOutputEngine {
    pub fn with_records(records: Vec<LayoutRecordV1>) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            result: Ok(LayoutOutputV1::new(records)),
        }
    }

    pub const fn with_error(error: LayoutEngineErrorV1) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            result: Err(error),
        }
    }

    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl LayoutEngineV1 for FakeOutputEngine {
    fn compute(
        &self,
        _input: ValidatedLayoutInputV1<'_>,
    ) -> Result<LayoutOutputV1, LayoutEngineErrorV1> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.result.clone()
    }
}

pub fn run(engine: &FakeOutputEngine) -> Result<LayoutOutputV1, LayoutErrorV1> {
    let result = compute_layout_v1(engine, LayoutInputV1::new(VIEWPORT, &NODES), LIMITS);
    assert_eq!(engine.calls(), 1, "valid input must invoke the engine once");
    result
}

pub fn assert_output_error(
    records: Vec<LayoutRecordV1>,
    expected_kind: LayoutOutputErrorKindV1,
    expected_location: LayoutErrorLocationV1,
) -> LayoutErrorV1 {
    let engine = FakeOutputEngine::with_records(records);
    let error = match run(&engine) {
        Err(error) => error,
        Ok(_) => panic!("malformed output crossed the boundary"),
    };
    assert_eq!(error.kind(), LayoutErrorKindV1::Output(expected_kind));
    assert_eq!(error.location(), expected_location);
    error
}

pub const fn rect(x: i32, y: i32, width: i32, height: i32) -> LayoutRectV1 {
    LayoutRectV1::new(x, y, width, height)
}

pub const fn record(key: u32, bounds: LayoutRectV1) -> LayoutRecordV1 {
    LayoutRecordV1::new(LayoutNodeKeyV1::new(key), bounds)
}

pub const fn output_record(index: u32) -> LayoutErrorLocationV1 {
    LayoutErrorLocationV1::OutputRecord { index }
}

pub fn valid_records() -> Vec<LayoutRecordV1> {
    vec![
        record(0, rect(0, 0, 20, 20)),
        record(1, rect(0, 20, 20, 20)),
    ]
}
