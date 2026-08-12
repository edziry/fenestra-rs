use std::sync::{Arc, Mutex};

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{MutationRecordView, UiRuntime};
use fenestra_ui_spatial::prototype::SpatialViewportV2;

use crate::spatial_support::engine::{EnginePlan, EngineSpy};
use crate::spatial_support::program::{MappingPlan, ProgramSpy, SourcePlan};
use crate::spatial_support::{VIEWPORT, limits, styled_program};
use crate::support::headless::{WIDTH, runtime_capacity};

#[test]
fn coalesced_resize_keeps_its_first_mutation_position() {
    let (program, program_state) = ProgramSpy::new(SourcePlan::Canonical, MappingPlan::Canonical);
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Reference);
    let mut runtime = UiRuntime::new_spatial_with_layout_engine(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        limits(),
        runtime_capacity(),
        Box::new(engine),
    )
    .expect("spatial runtime should initialize");
    let root = runtime.committed().root();
    let first = SpatialViewportV2::new(80, 60);
    let final_viewport = SpatialViewportV2::new(70, 55);
    let mut transaction = runtime.begin_transaction();
    transaction
        .resize_spatial(first)
        .expect("first resize should stage");
    transaction
        .set_property(root, WIDTH, PropertyValue::ScalarI32(83))
        .expect("property should stage between resizes");
    transaction
        .resize_spatial(final_viewport)
        .expect("final resize should stage");

    let receipt = runtime
        .commit(transaction)
        .expect("mixed transaction should publish");
    let mutations = receipt.mutations().collect::<Vec<_>>();

    assert_eq!(mutations.len(), 2);
    let MutationRecordView::SpatialViewportChanged(resize) = mutations[0] else {
        panic!("coalesced resize should retain its first position");
    };
    let MutationRecordView::PropertyChanged(property) = mutations[1] else {
        panic!("property should retain its authored position");
    };
    assert_eq!(resize.old_viewport(), VIEWPORT);
    assert_eq!(resize.new_viewport(), final_viewport);
    assert_eq!(property.node(), root);
    assert_eq!(property.property(), WIDTH);
    assert_eq!(property.old_value(), &PropertyValue::ScalarI32(100));
    assert_eq!(property.new_value(), &PropertyValue::ScalarI32(83));
    assert_eq!(program_state.calls(), 2);
    assert_eq!(engine_state.calls(), 2);
    assert_eq!(program_state.facts()[1].viewport, final_viewport);
}

#[test]
fn each_rebuild_calls_program_then_engine_exactly_once() {
    let trace = Arc::new(Mutex::new(Vec::new()));
    let (program, _) = ProgramSpy::with_trace(
        SourcePlan::Canonical,
        MappingPlan::Canonical,
        Arc::clone(&trace),
    );
    let (engine, _) = EngineSpy::with_trace(EnginePlan::Reference, Arc::clone(&trace));
    let mut runtime = UiRuntime::new_spatial_with_layout_engine(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        limits(),
        runtime_capacity(),
        Box::new(engine),
    )
    .expect("traced spatial runtime should initialize");
    let root = runtime.committed().root();
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(root, WIDTH, PropertyValue::ScalarI32(83))
        .expect("traced property should stage");
    drop(
        runtime
            .commit(transaction)
            .expect("traced property should publish"),
    );

    assert_eq!(
        *trace.lock().expect("callback trace should be available"),
        vec!["program", "engine", "program", "engine"]
    );
}
