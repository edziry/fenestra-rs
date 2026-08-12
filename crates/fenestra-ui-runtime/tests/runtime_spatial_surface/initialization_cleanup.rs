use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fenestra_ui_runtime::prototype::UiRuntime;
use fenestra_ui_spatial::prototype::SpatialViewportV2;

use crate::spatial_support::engine::{EngineMarker, EnginePlan, EngineSpy};
use crate::spatial_support::input::{canonical_source, malformed_source};
use crate::spatial_support::program::{MappingPlan, ProgramMarker, ProgramSpy, SourcePlan};
use crate::spatial_support::{VIEWPORT, limits, styled_program};
use crate::support::headless::runtime_capacity;

#[test]
fn program_panic_propagates_exactly_and_drops_both_owned_callbacks() {
    let program_drops = Arc::new(AtomicUsize::new(0));
    let engine_drops = Arc::new(AtomicUsize::new(0));
    let (program, program_state) = ProgramSpy::with_drop_probe(
        SourcePlan::Panic,
        MappingPlan::Canonical,
        Arc::clone(&program_drops),
    );
    let (engine, engine_state) =
        EngineSpy::with_drops(EnginePlan::Panic, Arc::clone(&engine_drops));

    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = UiRuntime::new_spatial_with_layout_engine(
            styled_program(),
            Box::new(program),
            VIEWPORT,
            limits(),
            runtime_capacity(),
            Box::new(engine),
        );
    }))
    .expect_err("program panic should propagate");

    assert_eq!(panic.downcast_ref::<ProgramMarker>(), Some(&ProgramMarker));
    assert_eq!(program_state.calls(), 1);
    assert_eq!(engine_state.calls(), 0);
    assert_eq!(program_drops.load(Ordering::SeqCst), 1);
    assert_eq!(engine_drops.load(Ordering::SeqCst), 1);
}

#[test]
fn engine_panic_rolls_back_candidate_ownership_and_drops_both_callbacks() {
    let source = canonical_source(VIEWPORT);
    let weak = Arc::downgrade(&source);
    let program_drops = Arc::new(AtomicUsize::new(0));
    let engine_drops = Arc::new(AtomicUsize::new(0));
    let (program, program_state) = ProgramSpy::with_drop_probe(
        SourcePlan::Exact(source),
        MappingPlan::Canonical,
        Arc::clone(&program_drops),
    );
    let (engine, engine_state) =
        EngineSpy::with_drops(EnginePlan::Panic, Arc::clone(&engine_drops));

    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = UiRuntime::new_spatial_with_layout_engine(
            styled_program(),
            Box::new(program),
            VIEWPORT,
            limits(),
            runtime_capacity(),
            Box::new(engine),
        );
    }))
    .expect_err("engine panic should propagate");

    assert_eq!(panic.downcast_ref::<EngineMarker>(), Some(&EngineMarker));
    assert_eq!(program_state.calls(), 1);
    assert_eq!(engine_state.calls(), 1);
    assert_eq!(program_drops.load(Ordering::SeqCst), 1);
    assert_eq!(engine_drops.load(Ordering::SeqCst), 1);
    assert!(weak.upgrade().is_none());
}

#[test]
fn typed_wrapper_and_resolver_failures_drop_candidate_and_callback_boxes() {
    assert_typed_cleanup(
        malformed_source(SpatialViewportV2::new(91, 70)),
        MappingPlan::Empty,
        EnginePlan::Panic,
        0,
    );
    assert_typed_cleanup(
        canonical_source(VIEWPORT),
        MappingPlan::Canonical,
        EnginePlan::Reject,
        1,
    );
}

fn assert_typed_cleanup(
    source: Arc<fenestra_ui_spatial::prototype::SpatialOwnedInputV2>,
    mapping: MappingPlan,
    engine_plan: EnginePlan,
    expected_engine_calls: usize,
) {
    let weak = Arc::downgrade(&source);
    let program_drops = Arc::new(AtomicUsize::new(0));
    let engine_drops = Arc::new(AtomicUsize::new(0));
    let (program, program_state) = ProgramSpy::with_drop_probe(
        SourcePlan::Exact(source),
        mapping,
        Arc::clone(&program_drops),
    );
    let (engine, engine_state) = EngineSpy::with_drops(engine_plan, Arc::clone(&engine_drops));

    let result = UiRuntime::new_spatial_with_layout_engine(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        limits(),
        runtime_capacity(),
        Box::new(engine),
    );

    assert!(result.is_err());
    assert_eq!(program_state.calls(), 1);
    assert_eq!(engine_state.calls(), expected_engine_calls);
    assert_eq!(program_drops.load(Ordering::SeqCst), 1);
    assert_eq!(engine_drops.load(Ordering::SeqCst), 1);
    assert!(weak.upgrade().is_none());
}
