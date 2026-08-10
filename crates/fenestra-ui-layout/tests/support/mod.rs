#![allow(dead_code)]

use std::sync::atomic::{AtomicUsize, Ordering};

use fenestra_ui_layout::prototype::{
    LayoutAxisV1, LayoutDimensionV1, LayoutEngineErrorV1, LayoutEngineV1, LayoutErrorKindV1,
    LayoutErrorLocationV1, LayoutExtentV1, LayoutInputErrorKindV1, LayoutInputV1, LayoutLimitsV1,
    LayoutNodeKeyV1, LayoutNodeV1, LayoutOutputV1, LayoutPaddingV1, LayoutRecordV1, LayoutRectV1,
    LayoutStyleV1, LayoutViewportV1, ValidatedLayoutInputV1, compute_layout_v1,
};

pub const VIEWPORT: LayoutViewportV1 = LayoutViewportV1::new(100, 80);
pub const GENEROUS_LIMITS: LayoutLimitsV1 = LayoutLimitsV1::new(64, 64, 64);

pub struct CountingEngine {
    calls: AtomicUsize,
}

impl CountingEngine {
    pub const fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
        }
    }

    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl LayoutEngineV1 for CountingEngine {
    fn compute(
        &self,
        input: ValidatedLayoutInputV1<'_>,
    ) -> Result<LayoutOutputV1, LayoutEngineErrorV1> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(LayoutOutputV1::new(
            input
                .nodes()
                .iter()
                .map(|node| LayoutRecordV1::new(node.key(), LayoutRectV1::new(0, 0, 0, 0)))
                .collect(),
        ))
    }
}

pub const fn dimension(minimum: i32, preferred: i32, maximum: i32) -> LayoutDimensionV1 {
    LayoutDimensionV1::new(minimum, preferred, maximum)
}

pub const fn padding(left: i32, right: i32, top: i32, bottom: i32) -> LayoutPaddingV1 {
    LayoutPaddingV1::new(left, right, top, bottom)
}

pub const fn style(
    width: LayoutDimensionV1,
    height: LayoutDimensionV1,
    padding: LayoutPaddingV1,
    gap: i32,
) -> LayoutStyleV1 {
    LayoutStyleV1::new(LayoutAxisV1::Column, width, height, padding, gap)
}

pub const fn node_with(
    key: u32,
    parent: Option<u32>,
    width: LayoutDimensionV1,
    height: LayoutDimensionV1,
    padding: LayoutPaddingV1,
    gap: i32,
) -> LayoutNodeV1 {
    LayoutNodeV1::new(
        LayoutNodeKeyV1::new(key),
        match parent {
            Some(parent) => Some(LayoutNodeKeyV1::new(parent)),
            None => None,
        },
        style(width, height, padding, gap),
    )
}

pub const fn node(key: u32, parent: Option<u32>) -> LayoutNodeV1 {
    node_with(
        key,
        parent,
        dimension(0, 20, 20),
        dimension(0, 10, 10),
        padding(0, 0, 0, 0),
        0,
    )
}

pub const fn root() -> LayoutNodeV1 {
    node(0, None)
}

pub fn assert_invalid(
    nodes: &[LayoutNodeV1],
    viewport: LayoutViewportV1,
    limits: LayoutLimitsV1,
    expected_kind: LayoutInputErrorKindV1,
    expected_location: LayoutErrorLocationV1,
) {
    let engine = CountingEngine::new();
    let result = compute_layout_v1(&engine, LayoutInputV1::new(viewport, nodes), limits);

    assert_eq!(
        engine.calls(),
        0,
        "invalid core input invoked the layout engine"
    );
    let error = result.expect_err("invalid core input should fail before engine invocation");
    assert_eq!(error.kind(), LayoutErrorKindV1::Input(expected_kind));
    assert_eq!(error.location(), expected_location);
}

pub fn assert_valid(nodes: &[LayoutNodeV1], viewport: LayoutViewportV1, limits: LayoutLimitsV1) {
    let engine = CountingEngine::new();
    compute_layout_v1(&engine, LayoutInputV1::new(viewport, nodes), limits)
        .expect("valid core input should reach the engine");
    assert_eq!(engine.calls(), 1);
}

pub const fn input_node(index: u32) -> LayoutErrorLocationV1 {
    LayoutErrorLocationV1::InputNode { index }
}

pub const fn negative_viewport(extent: LayoutExtentV1) -> LayoutInputErrorKindV1 {
    LayoutInputErrorKindV1::NegativeViewport(extent)
}
