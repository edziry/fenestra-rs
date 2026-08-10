use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fenestra_ui_layout::prototype::{
    LayoutAxisV1::Column, LayoutEngineErrorKindV1, LayoutEngineErrorV1, LayoutEngineV1,
    LayoutErrorKindV1, LayoutInputV1, LayoutOutputV1, REGISTERED_LAYOUT_LIMITS_V1,
    ValidatedLayoutInputV1, compute_layout_v1,
};

use crate::candidate::TaffyStackEngineV1;

use super::support::{dimension, fixed, fixed_node, node, padding, viewport};

struct CountingEngine {
    inner: TaffyStackEngineV1,
    calls: AtomicUsize,
}

impl CountingEngine {
    fn new(backend_calls: Arc<AtomicUsize>) -> Self {
        Self {
            inner: TaffyStackEngineV1::new_with_backend_counter(backend_calls),
            calls: AtomicUsize::new(0),
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl LayoutEngineV1 for CountingEngine {
    fn compute(
        &self,
        input: ValidatedLayoutInputV1<'_>,
    ) -> Result<LayoutOutputV1, LayoutEngineErrorV1> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.inner.compute(input)
    }
}

#[test]
fn core_invalid_input_never_enters_the_engine_or_taffy_backend() {
    let backend_calls = Arc::new(AtomicUsize::new(0));
    let engine = CountingEngine::new(Arc::clone(&backend_calls));

    compute_layout_v1(
        &engine,
        LayoutInputV1::new(viewport(0, 0), &[]),
        REGISTERED_LAYOUT_LIMITS_V1,
    )
    .expect_err("empty input must remain a core error");

    assert_eq!(engine.calls(), 0);
    assert_eq!(backend_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn core_valid_candidate_rejections_enter_engine_before_taffy_tree_construction() {
    let root = [fixed_node(0, None, Column, 0, 0)];
    assert_preflight_rejection(viewport(4097, 0), &root);

    let constraint = [node(
        0,
        None,
        Column,
        dimension(0, 0, 4097),
        fixed(0),
        padding(0, 0, 0, 0),
        0,
    )];
    assert_preflight_rejection(viewport(0, 0), &constraint);

    let gap = [
        node(
            0,
            None,
            Column,
            fixed(0),
            fixed(0),
            padding(0, 0, 0, 0),
            4097,
        ),
        fixed_node(1, Some(0), Column, 0, 0),
        fixed_node(2, Some(0), Column, 0, 0),
    ];
    assert_preflight_rejection(viewport(0, 0), &gap);
}

#[test]
fn admitted_input_invokes_exactly_one_call_local_taffy_backend() {
    let backend_calls = Arc::new(AtomicUsize::new(0));
    let engine = CountingEngine::new(Arc::clone(&backend_calls));
    let nodes = [fixed_node(0, None, Column, 4096, 4096)];

    let output = compute_layout_v1(
        &engine,
        LayoutInputV1::new(viewport(4096, 4096), &nodes),
        REGISTERED_LAYOUT_LIMITS_V1,
    )
    .expect("an admitted fixed root must solve");

    assert_eq!(engine.calls(), 1);
    assert_eq!(backend_calls.load(Ordering::SeqCst), 1);
    assert_eq!(output.records()[0].bounds().width(), 4096);
    assert_eq!(output.records()[0].bounds().height(), 4096);
}

fn assert_preflight_rejection(
    viewport: fenestra_ui_layout::prototype::LayoutViewportV1,
    nodes: &[fenestra_ui_layout::prototype::LayoutNodeV1],
) {
    let backend_calls = Arc::new(AtomicUsize::new(0));
    let engine = CountingEngine::new(Arc::clone(&backend_calls));
    let error = compute_layout_v1(
        &engine,
        LayoutInputV1::new(viewport, nodes),
        REGISTERED_LAYOUT_LIMITS_V1,
    )
    .expect_err("one-over candidate scalar must be rejected");

    assert_eq!(
        error.kind(),
        LayoutErrorKindV1::Engine(LayoutEngineErrorKindV1::RejectedInput)
    );
    assert_eq!(engine.calls(), 1);
    assert_eq!(backend_calls.load(Ordering::SeqCst), 0);
}
