use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};

use fenestra_ui_ir::prototype::{
    SUPPORTED_STYLE_FORMAT, SchemaNamespace, SchemaRevision, StyleProgram, StyleValidationLimits,
    validate_style,
};
use fenestra_ui_layout::prototype::{
    LayoutAxisV1, LayoutDimensionV1, LayoutEngineErrorV1, LayoutEngineV1, LayoutOutputV1,
    LayoutPaddingV1, ReferenceStackEngineV1, ValidatedLayoutInputV1,
};
use fenestra_ui_spatial::prototype::{
    Affine2V2, SpatialContainerV2, SpatialLayoutPlacementV2, SpatialLimitsV2,
    SpatialLocalTransformV2, SpatialNodeKeyV2, SpatialNodeV2, SpatialOwnedInputV2,
    SpatialPlacementV2, SpatialPointV2, SpatialScalarV2, SpatialViewportV2,
};

use super::{
    CommitTestHook, PROPERTY, RuntimeCapacity, TransactionErrorKind, UiRuntime,
    changed_transaction, construction,
};
use crate::runtime::error::CapacityKind;
use crate::runtime::spatial::{
    RuntimeSpatialBuildViewV2, RuntimeSpatialInputV2, RuntimeSpatialProgramV2,
};

const VIEWPORT: SpatialViewportV2 = SpatialViewportV2::new(80, 60);
const NAMESPACE: SchemaNamespace = SchemaNamespace::new(99);
const REVISION: SchemaRevision = SchemaRevision::new(1);

#[derive(Default)]
struct ProgramState {
    calls: AtomicUsize,
    sources: Mutex<Vec<Weak<SpatialOwnedInputV2>>>,
}

struct TrackingProgram {
    state: Arc<ProgramState>,
}

impl RuntimeSpatialProgramV2 for TrackingProgram {
    fn build(
        &self,
        runtime: RuntimeSpatialBuildViewV2<'_>,
        viewport: SpatialViewportV2,
    ) -> RuntimeSpatialInputV2 {
        self.state.calls.fetch_add(1, Ordering::SeqCst);
        let root = runtime.root();
        let member = runtime
            .children(root)
            .and_then(|children| children.first())
            .copied()
            .expect("spatial unit fixture should contain one member");
        let source = source(viewport);
        self.state
            .sources
            .lock()
            .expect("spatial source observations should be available")
            .push(Arc::downgrade(&source));
        RuntimeSpatialInputV2::new(source, vec![member].into_boxed_slice())
    }
}

struct TrackingEngine {
    calls: Arc<AtomicUsize>,
}

impl LayoutEngineV1 for TrackingEngine {
    fn compute(
        &self,
        input: ValidatedLayoutInputV1<'_>,
    ) -> Result<LayoutOutputV1, LayoutEngineErrorV1> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        ReferenceStackEngineV1::new().compute(input)
    }
}

fn runtime(retained_generations: usize) -> (UiRuntime, Arc<ProgramState>, Arc<AtomicUsize>) {
    let style = validate_style(
        &construction(),
        StyleProgram::new(
            SUPPORTED_STYLE_FORMAT,
            NAMESPACE,
            REVISION,
            Vec::new(),
            fenestra_ui_ir::prototype::SourceSpan::synthetic(),
        ),
        StyleValidationLimits::new(0),
    )
    .expect("spatial unit style should validate");
    let program_state = Arc::new(ProgramState::default());
    let engine_calls = Arc::new(AtomicUsize::new(0));
    let runtime = UiRuntime::new_spatial_with_layout_engine(
        style,
        Box::new(TrackingProgram {
            state: Arc::clone(&program_state),
        }),
        VIEWPORT,
        SpatialLimitsV2::new([usize::MAX; 30]),
        RuntimeCapacity::new(8, 8, 8, 8, 8, retained_generations),
        Box::new(TrackingEngine {
            calls: Arc::clone(&engine_calls),
        }),
    )
    .expect("spatial unit runtime should initialize");
    (runtime, program_state, engine_calls)
}

#[test]
fn injected_draft_corruption_precedes_spatial_rebuild_and_preserves_publication() {
    let hooks = [
        CommitTestHook::CorruptPropertiesBeforeValidation,
        CommitTestHook::CorruptTreeBeforeValidation,
        CommitTestHook::CorruptFragmentBeforeValidation,
    ];
    for hook in hooks {
        let (mut runtime, program, engine) = runtime(4);
        let before = runtime.committed();
        let before_spatial = before.spatial().expect("spatial publication should exist");
        let mapped = before_spatial
            .logical_node(SpatialNodeKeyV2::new(1))
            .expect("spatial key should be mapped");
        let transaction = changed_transaction(&runtime, 10);

        let error = runtime
            .commit_with_test_hook(transaction, hook)
            .expect_err("corrupt logical draft should fail");

        assert_eq!(error.kind(), TransactionErrorKind::InvariantViolation);
        assert_eq!(error.operation_index(), None);
        assert_eq!(program.calls.load(Ordering::SeqCst), 1);
        assert_eq!(engine.load(Ordering::SeqCst), 1);
        assert_unchanged(&before, &runtime, before_spatial.snapshot(), mapped);
    }
}

#[test]
fn spatial_rebuild_decision_is_independent_of_aggregate_invalidation() {
    let (mut runtime, program, engine) = runtime(4);
    let before = runtime.committed();
    let before_spatial = before.spatial().expect("spatial publication should exist");
    let mapped = before_spatial
        .logical_node(SpatialNodeKeyV2::new(1))
        .expect("spatial key should be mapped");
    let transaction = changed_transaction(&runtime, 10);

    let receipt = runtime
        .commit_with_test_hook(transaction, CommitTestHook::EmptyInvalidationBeforeRebuild)
        .expect("effective empty-invalidation commit should publish");

    assert!(!receipt.is_empty());
    assert!(receipt.invalidation().is_empty());
    assert_eq!(receipt.mutations().count(), 1);
    assert_eq!(receipt.generation().get(), 1);
    assert_eq!(program.calls.load(Ordering::SeqCst), 2);
    assert_eq!(engine.load(Ordering::SeqCst), 2);
    let after = runtime.committed();
    assert!(!before.shares_state_with(&after));
    let after_spatial = after.spatial().expect("spatial publication should exist");
    assert!(!std::ptr::eq(
        before_spatial.snapshot(),
        after_spatial.snapshot()
    ));
    assert_eq!(
        after_spatial.logical_node(SpatialNodeKeyV2::new(1)),
        Some(mapped)
    );
    assert_eq!(
        after_spatial.spatial_key(mapped),
        Some(SpatialNodeKeyV2::new(1))
    );
    assert_eq!(
        after.property(before.root(), PROPERTY),
        Some(&fenestra_ui_ir::prototype::PropertyValue::ScalarI32(10))
    );
}

#[test]
fn spatial_rebuild_precedes_generation_exhaustion_and_drops_candidate_source() {
    assert_late_publication_failure(4, TransactionErrorKind::GenerationExhausted);
}

#[test]
fn retained_generation_capacity_still_precedes_generation_exhaustion_after_spatial_rebuild() {
    assert_late_publication_failure(
        0,
        TransactionErrorKind::CapacityExceeded(CapacityKind::RetainedGenerations),
    );
}

fn assert_late_publication_failure(retained_generations: usize, expected: TransactionErrorKind) {
    let (mut runtime, program, engine) = runtime(retained_generations);
    runtime.set_generation_for_test(u64::MAX);
    let before = runtime.committed();
    let before_spatial = before.spatial().expect("spatial publication should exist");
    let mapped = before_spatial
        .logical_node(SpatialNodeKeyV2::new(1))
        .expect("spatial key should be mapped");
    let transaction = changed_transaction(&runtime, 10);

    let error = runtime
        .commit(transaction)
        .expect_err("late publication guard should reject the commit");

    assert_eq!(error.kind(), expected);
    assert_eq!(error.operation_index(), None);
    assert_eq!(program.calls.load(Ordering::SeqCst), 2);
    assert_eq!(engine.load(Ordering::SeqCst), 2);
    let sources = program
        .sources
        .lock()
        .expect("spatial source observations should be available");
    assert_eq!(sources.len(), 2);
    assert!(sources[0].upgrade().is_some());
    assert!(sources[1].upgrade().is_none());
    assert_unchanged(&before, &runtime, before_spatial.snapshot(), mapped);
}

fn assert_unchanged(
    before: &crate::runtime::view::CommittedRuntimeSnapshot,
    runtime: &UiRuntime,
    snapshot: &fenestra_ui_spatial::prototype::SpatialResolvedSnapshotV2,
    mapped: crate::logical_tree::NodeId,
) {
    let after = runtime.committed();
    assert!(before.shares_state_with(&after));
    let after_spatial = after.spatial().expect("spatial publication should remain");
    assert!(std::ptr::eq(snapshot, after_spatial.snapshot()));
    assert_eq!(
        after_spatial.logical_node(SpatialNodeKeyV2::new(1)),
        Some(mapped)
    );
    assert_eq!(
        after_spatial.spatial_key(mapped),
        Some(SpatialNodeKeyV2::new(1))
    );
    assert_eq!(
        after.property(before.root(), PROPERTY),
        Some(&fenestra_ui_ir::prototype::PropertyValue::ScalarI32(0))
    );
}

fn source(viewport: SpatialViewportV2) -> Arc<SpatialOwnedInputV2> {
    Arc::new(SpatialOwnedInputV2::new(
        viewport,
        vec![root_node(), layout_node()].into_boxed_slice(),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
    ))
}

fn root_node() -> SpatialNodeV2 {
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(0),
        None,
        SpatialPlacementV2::Root,
        container(),
    )
}

fn layout_node() -> SpatialNodeV2 {
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(1),
        Some(SpatialNodeKeyV2::new(0)),
        SpatialPlacementV2::Layout(SpatialLayoutPlacementV2::new(
            dimension(12),
            dimension(8),
            identity(),
        )),
        container(),
    )
}

fn dimension(value: i32) -> LayoutDimensionV1 {
    LayoutDimensionV1::new(value, value, value)
}

fn container() -> SpatialContainerV2 {
    SpatialContainerV2::new(LayoutAxisV1::Column, LayoutPaddingV1::new(0, 0, 0, 0), 0)
}

fn identity() -> SpatialLocalTransformV2 {
    let zero = SpatialScalarV2::new(0);
    let one = SpatialScalarV2::new(SpatialScalarV2::SCALE);
    SpatialLocalTransformV2::new(
        Affine2V2::new(one, zero, zero, one, zero, zero),
        SpatialPointV2::new(zero, zero),
    )
}
