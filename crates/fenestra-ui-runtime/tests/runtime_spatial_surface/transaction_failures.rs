use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;
use std::sync::Arc;

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_layout::prototype::LayoutEngineErrorKindV1;
use fenestra_ui_runtime::prototype::{CapacityKind, TransactionErrorKind, UiRuntime};
use fenestra_ui_spatial::prototype::{
    SpatialErrorLocationV2, SpatialExtentV2, SpatialInputErrorKindV2, SpatialLayoutErrorKindV2,
    SpatialResolveErrorKindV2, SpatialViewportV2,
};

use crate::RuntimeSpatialErrorV2;
use crate::spatial_support::engine::{EngineMarker, EnginePlan, EngineSpy, EngineState};
use crate::spatial_support::input::canonical_source;
use crate::spatial_support::program::{
    MappingPlan, ProgramMarker, ProgramSpy, ProgramState, SourcePlan,
};
use crate::spatial_support::{VIEWPORT, limits, styled_program};
use crate::support::headless::{WIDTH, construction, runtime_capacity};

fn runtime_with(
    source: SourcePlan,
    engine_plan: EnginePlan,
    capacity: fenestra_ui_runtime::prototype::RuntimeCapacity,
) -> (UiRuntime, Arc<ProgramState>, Arc<EngineState>) {
    let (program, program_state) = ProgramSpy::new(source, MappingPlan::Canonical);
    let (engine, engine_state) = EngineSpy::new(engine_plan);
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

fn change_root(runtime: &UiRuntime, value: i32) -> fenestra_ui_runtime::prototype::UiTransaction {
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(
            runtime.committed().root(),
            WIDTH,
            PropertyValue::ScalarI32(value),
        )
        .expect("property change should stage");
    transaction
}

#[test]
fn stale_and_authored_errors_precede_both_spatial_callbacks() {
    let (mut runtime, program, engine) = runtime_with(
        SourcePlan::Canonical,
        EnginePlan::Reference,
        runtime_capacity(),
    );
    let stale = change_root(&runtime, 81);
    drop(
        runtime
            .commit(change_root(&runtime, 82))
            .expect("winner should publish"),
    );
    let after_winner = runtime.committed();
    let calls = (program.calls(), engine.calls());
    let error = runtime.commit(stale).expect_err("old base should be stale");
    assert_eq!(error.kind(), TransactionErrorKind::StaleBase);
    assert_eq!((program.calls(), engine.calls()), calls);
    assert!(after_winner.shares_state_with(&runtime.committed()));

    let foreign = UiRuntime::new(construction(), runtime_capacity())
        .expect("foreign runtime should initialize")
        .committed()
        .root();
    let mut missing = runtime.begin_transaction();
    missing
        .set_property(foreign, WIDTH, PropertyValue::ScalarI32(99))
        .expect("foreign property should stage");
    let error = runtime
        .commit(missing)
        .expect_err("foreign node should fail during apply");
    assert_eq!(error.kind(), TransactionErrorKind::MissingNode);
    assert_eq!(error.operation_index(), Some(0));
    assert_eq!((program.calls(), engine.calls()), calls);
    assert!(after_winner.shares_state_with(&runtime.committed()));
}

#[test]
fn viewport_wrapper_failure_rolls_back_and_keeps_the_committed_viewport() {
    let original_source = canonical_source(VIEWPORT);
    let (mut runtime, program, engine) = runtime_with(
        SourcePlan::Exact(original_source),
        EnginePlan::Reference,
        runtime_capacity(),
    );
    let before = runtime.committed();
    let before_spatial = before.spatial().expect("spatial state should exist");
    let before_key =
        before_spatial.logical_node(fenestra_ui_spatial::prototype::SpatialNodeKeyV2::new(1));
    let mut transaction = runtime.begin_transaction();
    transaction
        .resize_spatial(SpatialViewportV2::new(120, 80))
        .expect("resize should stage");
    let error = runtime
        .commit(transaction)
        .expect_err("fixed source viewport should mismatch");

    assert_eq!(
        error.kind(),
        TransactionErrorKind::Spatial(RuntimeSpatialErrorV2::ViewportMismatch)
    );
    assert_eq!(error.operation_index(), None);
    assert_eq!(program.calls(), 2);
    assert_eq!(engine.calls(), 1);
    let after_failure = runtime.committed();
    assert!(before.shares_state_with(&after_failure));
    assert!(ptr::eq(
        before_spatial.snapshot(),
        after_failure
            .spatial()
            .expect("spatial state should remain")
            .snapshot()
    ));
    assert_eq!(
        after_failure
            .spatial()
            .expect("spatial state should remain")
            .logical_node(fenestra_ui_spatial::prototype::SpatialNodeKeyV2::new(1)),
        before_key
    );
    drop(after_failure);
    drop(before);

    drop(
        runtime
            .commit(change_root(&runtime, 83))
            .expect("property-only retry should use the committed viewport"),
    );
    assert_eq!(
        program.facts().last().expect("retry fact").viewport,
        VIEWPORT
    );
    assert_eq!(program.calls(), 3);
    assert_eq!(engine.calls(), 2);
}

#[test]
fn resolver_failure_is_preserved_without_resize_operation_attribution() {
    let (mut runtime, program, engine) = runtime_with(
        SourcePlan::Canonical,
        EnginePlan::Reference,
        runtime_capacity(),
    );
    let before = runtime.committed();
    let mut transaction = runtime.begin_transaction();
    transaction
        .resize_spatial(SpatialViewportV2::new(-1, 70))
        .expect("negative viewport should stage for final resolution");
    let error = runtime
        .commit(transaction)
        .expect_err("negative viewport should fail resolution");
    let TransactionErrorKind::Spatial(RuntimeSpatialErrorV2::Resolve(resolve)) = error.kind()
    else {
        panic!("expected one wrapped resolver failure");
    };

    assert_eq!(
        resolve.kind(),
        SpatialResolveErrorKindV2::Input(SpatialInputErrorKindV2::NegativeViewport(
            SpatialExtentV2::Width,
        ))
    );
    assert_eq!(
        resolve.location(),
        SpatialErrorLocationV2::Viewport {
            extent: SpatialExtentV2::Width,
        }
    );
    assert_eq!(error.operation_index(), None);
    assert_eq!(program.calls(), 2);
    assert_eq!(engine.calls(), 1);
    assert!(before.shares_state_with(&runtime.committed()));
    drop(before);
    drop(
        runtime
            .commit(change_root(&runtime, 87))
            .expect("property retry should use the original viewport"),
    );
    assert_eq!(
        program.facts().last().expect("retry fact").viewport,
        VIEWPORT
    );
    assert_eq!(program.calls(), 3);
    assert_eq!(engine.calls(), 2);
}

#[test]
fn late_layout_rejection_is_wrapped_and_drops_only_the_candidate_source() {
    let (mut runtime, program, engine) = runtime_with(
        SourcePlan::FreshCanonical,
        EnginePlan::RejectOnCall(2),
        runtime_capacity(),
    );
    let before = runtime.committed();
    let before_snapshot = before
        .spatial()
        .expect("spatial state should exist")
        .snapshot();
    let error = runtime
        .commit(change_root(&runtime, 84))
        .expect_err("second layout call should reject the rebuild");
    let TransactionErrorKind::Spatial(RuntimeSpatialErrorV2::Resolve(resolve)) = error.kind()
    else {
        panic!("expected one wrapped layout rejection");
    };

    assert_eq!(
        resolve.kind(),
        SpatialResolveErrorKindV2::Layout(SpatialLayoutErrorKindV2::Engine(
            LayoutEngineErrorKindV1::RejectedInput,
        ))
    );
    assert_eq!(
        resolve.location(),
        SpatialErrorLocationV2::Node { index: 2 }
    );
    assert_eq!(resolve.observed(), None);
    assert_eq!(resolve.maximum(), None);
    assert_eq!(error.operation_index(), None);
    assert_eq!((program.calls(), engine.calls()), (2, 2));
    assert!(before.shares_state_with(&runtime.committed()));
    assert!(ptr::eq(
        before_snapshot,
        runtime
            .committed()
            .spatial()
            .expect("spatial state should remain")
            .snapshot()
    ));
    assert_prior_source_alive_and_candidate_dropped(&program);
}

#[test]
fn program_and_engine_panics_preserve_state_and_allow_a_fresh_retry() {
    assert_panic_rollback(SourcePlan::PanicOnCall(2), EnginePlan::Reference, true);
    assert_panic_rollback(
        SourcePlan::FreshCanonical,
        EnginePlan::PanicOnCall(2),
        false,
    );
}

fn assert_panic_rollback(source: SourcePlan, engine_plan: EnginePlan, program_panics: bool) {
    let (mut runtime, program, engine) = runtime_with(source, engine_plan, runtime_capacity());
    let before = runtime.committed();
    let before_snapshot = before
        .spatial()
        .expect("spatial state should exist")
        .snapshot();
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = runtime.commit(change_root(&runtime, 84));
    }))
    .expect_err("scripted callback panic should propagate");
    if program_panics {
        assert_eq!(panic.downcast_ref::<ProgramMarker>(), Some(&ProgramMarker));
        assert_eq!((program.calls(), engine.calls()), (2, 1));
    } else {
        assert_eq!(panic.downcast_ref::<EngineMarker>(), Some(&EngineMarker));
        assert_eq!((program.calls(), engine.calls()), (2, 2));
    }
    assert!(before.shares_state_with(&runtime.committed()));
    assert!(ptr::eq(
        before_snapshot,
        runtime
            .committed()
            .spatial()
            .expect("spatial state should remain")
            .snapshot()
    ));
    if !program_panics {
        assert_prior_source_alive_and_candidate_dropped(&program);
    }

    drop(
        runtime
            .commit(change_root(&runtime, 85))
            .expect("one-shot callback panic should permit retry"),
    );
    assert_eq!(runtime.committed().generation().get(), 1);
}

#[test]
fn retained_generation_capacity_is_checked_after_complete_spatial_rebuild() {
    let (mut runtime, program, engine) = runtime_with(
        SourcePlan::FreshCanonical,
        EnginePlan::Reference,
        runtime_capacity().with_retained_generations(0),
    );
    let before = runtime.committed();
    let before_snapshot = before
        .spatial()
        .expect("spatial state should exist")
        .snapshot();
    let error = runtime
        .commit(change_root(&runtime, 86))
        .expect_err("retained generation capacity should reject publication");

    assert_eq!(
        error.kind(),
        TransactionErrorKind::CapacityExceeded(CapacityKind::RetainedGenerations)
    );
    assert_eq!(error.operation_index(), None);
    assert_eq!(program.calls(), 2);
    assert_eq!(engine.calls(), 2);
    assert!(before.shares_state_with(&runtime.committed()));
    assert!(ptr::eq(
        before_snapshot,
        runtime
            .committed()
            .spatial()
            .expect("spatial state should remain")
            .snapshot()
    ));
    assert_prior_source_alive_and_candidate_dropped(&program);
}

fn assert_prior_source_alive_and_candidate_dropped(program: &ProgramState) {
    let sources = program.source_weaks();
    assert_eq!(sources.len(), 2);
    assert!(
        sources[0].upgrade().is_some(),
        "the prior committed snapshot should retain its source"
    );
    assert!(
        sources[1].upgrade().is_none(),
        "the unpublished candidate source should be dropped"
    );
}
