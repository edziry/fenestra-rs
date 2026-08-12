use std::ptr;
use std::sync::Arc;

use fenestra_ui_ir::prototype::{InvalidationClass, InvalidationSet, PropertyValue};
use fenestra_ui_runtime::prototype::{
    CapacityKind, MutationRecordView, TransactionErrorKind, UiRuntime,
};
use fenestra_ui_spatial::prototype::SpatialViewportV2;

use crate::spatial_support::engine::{EnginePlan, EngineSpy, EngineState};
use crate::spatial_support::none::{RootOnlyProgram, VALUE, style as none_style};
use crate::spatial_support::program::{MappingPlan, ProgramSpy, ProgramState, SourcePlan};
use crate::spatial_support::{VIEWPORT, limits, styled_program};
use crate::support::headless::{WIDTH, construction, exact_style, runtime_capacity};
use crate::support::headless_spec::{HeadlessSpecBuilder, surface};

fn spatial_runtime(
    capacity: fenestra_ui_runtime::prototype::RuntimeCapacity,
) -> (UiRuntime, Arc<ProgramState>, Arc<EngineState>) {
    let (program, program_state) = ProgramSpy::new(SourcePlan::Canonical, MappingPlan::Canonical);
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Reference);
    let runtime = UiRuntime::new_spatial_with_layout_engine(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        limits(),
        capacity,
        Box::new(engine),
    )
    .expect("spatial runtime should initialize");
    (runtime, program_state, engine_state)
}

fn resize_invalidation() -> InvalidationSet {
    [
        InvalidationClass::Layout,
        InvalidationClass::Semantics,
        InvalidationClass::HitTest,
        InvalidationClass::Paint,
        InvalidationClass::Composition,
    ]
    .into_iter()
    .fold(InvalidationSet::NONE, |set, class| {
        set.union(InvalidationSet::from_class(class))
    })
}

#[test]
fn spatial_resize_rebuilds_once_and_publishes_one_exact_record() {
    let (mut runtime, program, engine) = spatial_runtime(runtime_capacity());
    let before = runtime.committed();
    let before_spatial = before.spatial().expect("spatial state should exist");
    let resized = SpatialViewportV2::new(120, 80);
    let mut transaction = runtime.begin_transaction();
    transaction
        .resize_spatial(resized)
        .expect("spatial resize should stage");

    let receipt = runtime
        .commit(transaction)
        .expect("spatial resize should publish");
    let after = runtime.committed();
    let after_spatial = after
        .spatial()
        .expect("spatial state should remain available");
    let mut mutations = receipt.mutations();

    assert_eq!(program.calls(), 2);
    assert_eq!(engine.calls(), 2);
    assert_eq!(receipt.invalidation(), resize_invalidation());
    assert!(!receipt.invalidation().contains(InvalidationClass::Surface));
    assert!(
        !receipt
            .invalidation()
            .contains(InvalidationClass::Structure)
    );
    assert_eq!(mutations.len(), 1);
    let Some(MutationRecordView::SpatialViewportChanged(change)) = mutations.next() else {
        panic!("resize should emit one spatial viewport change");
    };
    assert_eq!(change.old_viewport(), VIEWPORT);
    assert_eq!(change.new_viewport(), resized);
    assert_eq!(before_spatial.snapshot().viewport(), VIEWPORT);
    assert_eq!(after_spatial.snapshot().viewport(), resized);
    assert!(!ptr::eq(
        before_spatial.snapshot(),
        after_spatial.snapshot()
    ));
    assert!(!before.shares_state_with(&after));
    assert_eq!(after.generation().get(), before.generation().get() + 1);
    drop(receipt);
    drop(before);
    drop(after);

    let current = runtime.committed();
    let mut property = runtime.begin_transaction();
    property
        .set_property(current.root(), WIDTH, PropertyValue::ScalarI32(83))
        .expect("property-only rebuild should stage");
    drop(
        runtime
            .commit(property)
            .expect("property-only rebuild should publish"),
    );
    assert_eq!(
        program
            .facts()
            .last()
            .expect("property rebuild fact")
            .viewport,
        resized
    );
    assert_eq!(program.calls(), 3);
    assert_eq!(engine.calls(), 3);
}

#[test]
fn spatial_resize_coalesces_only_the_original_and_final_viewports() {
    let (mut runtime, program, engine) = spatial_runtime(runtime_capacity());
    let final_viewport = SpatialViewportV2::new(70, 55);
    let mut transaction = runtime.begin_transaction();
    transaction
        .resize_spatial(SpatialViewportV2::new(-1, 5))
        .expect("invalid intermediate viewport should stage");
    transaction
        .resize_spatial(final_viewport)
        .expect("valid final viewport should stage");

    let receipt = runtime
        .commit(transaction)
        .expect("only the final viewport should be resolved");
    let mut mutations = receipt.mutations();
    let Some(MutationRecordView::SpatialViewportChanged(change)) = mutations.next() else {
        panic!("coalesced resize should retain one record");
    };
    assert!(mutations.next().is_none());
    assert_eq!(change.old_viewport(), VIEWPORT);
    assert_eq!(change.new_viewport(), final_viewport);
    assert_eq!(program.calls(), 2);
    assert_eq!(engine.calls(), 2);
    assert_eq!(program.facts()[1].viewport, final_viewport);
}

#[test]
fn spatial_viewport_round_trip_is_a_true_noop() {
    let (mut runtime, program, engine) = spatial_runtime(runtime_capacity());
    let before = runtime.committed();
    let before_snapshot = before
        .spatial()
        .expect("spatial state should exist")
        .snapshot();
    let mut transaction = runtime.begin_transaction();
    transaction
        .resize_spatial(SpatialViewportV2::new(60, 50))
        .expect("intermediate viewport should stage");
    transaction
        .resize_spatial(VIEWPORT)
        .expect("original viewport should stage");

    let receipt = runtime
        .commit(transaction)
        .expect("viewport round trip should be valid");
    let after = runtime.committed();

    assert!(receipt.is_empty());
    assert!(receipt.invalidation().is_empty());
    assert!(before.shares_state_with(&after));
    assert!(ptr::eq(
        before_snapshot,
        after
            .spatial()
            .expect("spatial state should remain")
            .snapshot()
    ));
    assert_eq!(program.calls(), 1);
    assert_eq!(engine.calls(), 1);
}

#[test]
fn every_logical_and_spatial_true_noop_skips_both_callbacks() {
    let (mut runtime, program, engine) = spatial_runtime(runtime_capacity());
    let before = runtime.committed();
    let root = before.root();

    let empty = runtime.begin_transaction();
    assert!(runtime.commit(empty).expect("empty transaction").is_empty());

    let mut same_value = runtime.begin_transaction();
    same_value
        .set_property(root, WIDTH, PropertyValue::ScalarI32(100))
        .expect("same property should stage");
    assert!(
        runtime
            .commit(same_value)
            .expect("same property should commit")
            .is_empty()
    );

    let mut round_trip = runtime.begin_transaction();
    round_trip
        .set_property(root, WIDTH, PropertyValue::ScalarI32(101))
        .expect("intermediate property should stage");
    round_trip
        .set_property(root, WIDTH, PropertyValue::ScalarI32(100))
        .expect("original property should stage");
    assert!(
        runtime
            .commit(round_trip)
            .expect("property round trip should commit")
            .is_empty()
    );

    let after = runtime.committed();
    assert!(before.shares_state_with(&after));
    assert_eq!(program.calls(), 1);
    assert_eq!(engine.calls(), 1);
}

#[test]
fn every_effective_commit_rebuilds_when_only_style_match_is_invalidated() {
    let (program, state) = RootOnlyProgram::new();
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Panic);
    let mut runtime = UiRuntime::new_spatial_with_layout_engine(
        none_style(),
        Box::new(program),
        VIEWPORT,
        limits(),
        runtime_capacity(),
        Box::new(engine),
    )
    .expect("root-only spatial runtime should initialize");
    let before = runtime.committed();
    let before_snapshot = before
        .spatial()
        .expect("spatial state should exist")
        .snapshot();
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(before.root(), VALUE, PropertyValue::ScalarI32(1))
        .expect("zero-invalidation property should stage");

    let receipt = runtime
        .commit(transaction)
        .expect("effective zero-invalidation change should publish");
    let after = runtime.committed();

    assert!(!receipt.is_empty());
    assert_eq!(
        receipt.invalidation(),
        InvalidationSet::from_class(InvalidationClass::StyleMatch)
    );
    for class in [
        InvalidationClass::Structure,
        InvalidationClass::Intrinsic,
        InvalidationClass::Layout,
        InvalidationClass::Semantics,
        InvalidationClass::HitTest,
        InvalidationClass::Paint,
        InvalidationClass::Composition,
    ] {
        assert!(!receipt.invalidation().contains(class));
    }
    assert_eq!(state.calls(), 2);
    assert_eq!(state.values(), vec![0, 1]);
    assert_eq!(engine_state.calls(), 0);
    assert!(!before.shares_state_with(&after));
    assert!(!ptr::eq(
        before_snapshot,
        after
            .spatial()
            .expect("spatial state should remain")
            .snapshot()
    ));
}

#[test]
fn spatial_resize_participates_in_the_existing_operation_ceiling() {
    let capacity = runtime_capacity().with_operations(8);
    let (mut runtime, program, engine) = spatial_runtime(capacity);
    let before = runtime.committed();
    let mut transaction = runtime.begin_transaction();
    for _ in 0..8 {
        transaction
            .set_property(before.root(), WIDTH, PropertyValue::ScalarI32(100))
            .expect("operation at the inclusive ceiling should stage");
    }
    let staged = transaction
        .resize_spatial(SpatialViewportV2::new(80, 60))
        .expect_err("ninth operation should poison the transaction");
    let committed = runtime
        .commit(transaction)
        .expect_err("poisoned transaction should fail");

    assert_eq!(staged, committed);
    assert_eq!(staged.operation_index(), Some(8));
    assert_eq!(
        staged.kind(),
        TransactionErrorKind::CapacityExceeded(CapacityKind::Operations)
    );
    assert!(before.shares_state_with(&runtime.committed()));
    assert_eq!(program.calls(), 1);
    assert_eq!(engine.calls(), 1);
}

#[test]
fn projection_modes_reject_the_other_resize_at_its_authored_index() {
    let mut ordinary = UiRuntime::new(construction(), runtime_capacity())
        .expect("ordinary runtime should initialize");
    let ordinary_before = ordinary.committed();
    let mut ordinary_tx = ordinary.begin_transaction();
    ordinary_tx
        .set_property(ordinary_before.root(), WIDTH, PropertyValue::ScalarI32(84))
        .expect("prior property should stage");
    ordinary_tx
        .resize_spatial(VIEWPORT)
        .expect("unavailable resize should fail at commit");
    let error = ordinary
        .commit(ordinary_tx)
        .expect_err("ordinary runtime has no spatial projection");
    assert_eq!(error.kind(), TransactionErrorKind::SpatialUnavailable);
    assert_eq!(error.operation_index(), Some(1));
    assert!(ordinary_before.shares_state_with(&ordinary.committed()));

    let mut headless = UiRuntime::new_headless(
        exact_style(),
        HeadlessSpecBuilder::new().build(),
        surface(),
        runtime_capacity(),
    )
    .expect("headless runtime should initialize");
    let headless_before = headless.committed();
    let mut headless_tx = headless.begin_transaction();
    headless_tx
        .resize_spatial(VIEWPORT)
        .expect("spatial resize should stage");
    let error = headless
        .commit(headless_tx)
        .expect_err("headless runtime has no spatial projection");
    assert_eq!(error.kind(), TransactionErrorKind::SpatialUnavailable);
    assert_eq!(error.operation_index(), Some(0));
    assert!(headless_before.shares_state_with(&headless.committed()));

    let (mut spatial, program, engine) = spatial_runtime(runtime_capacity());
    let spatial_before = spatial.committed();
    let mut spatial_tx = spatial.begin_transaction();
    spatial_tx
        .resize_headless(surface())
        .expect("headless resize should stage");
    let error = spatial
        .commit(spatial_tx)
        .expect_err("spatial runtime has no headless projection");
    assert_eq!(error.kind(), TransactionErrorKind::HeadlessUnavailable);
    assert_eq!(error.operation_index(), Some(0));
    assert!(spatial_before.shares_state_with(&spatial.committed()));
    assert_eq!(program.calls(), 1);
    assert_eq!(engine.calls(), 1);
}
