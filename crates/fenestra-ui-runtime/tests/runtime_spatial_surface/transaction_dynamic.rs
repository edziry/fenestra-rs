use std::ptr;
use std::sync::Arc;

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{NodeId, UiRuntime};
use fenestra_ui_spatial::prototype::{SpatialNodeKeyV2, SpatialScalarV2};

use crate::spatial_support::dynamic::{DynamicProgram, DynamicProgramState};
use crate::spatial_support::engine::{EnginePlan, EngineSpy, EngineState};
use crate::spatial_support::{VIEWPORT, limits, styled_program};
use crate::support::headless::{
    CONTROL_STYLE_COLOR, FIRST_KEY, HEIGHT, INSERTED_KEY, ITEM_STYLE_COLOR, ITEMS, SECOND_KEY,
    WIDTH, runtime_capacity,
};

fn dynamic_runtime() -> (UiRuntime, Arc<DynamicProgramState>, Arc<EngineState>) {
    let (program, program_state) = DynamicProgram::new();
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Reference);
    let runtime = UiRuntime::new_spatial_with_layout_engine(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        limits(),
        runtime_capacity(),
        Box::new(engine),
    )
    .expect("dynamic spatial runtime should initialize");
    (runtime, program_state, engine_state)
}

fn items(runtime: &UiRuntime) -> fenestra_ui_runtime::prototype::FragmentId {
    let committed = runtime.committed();
    let root = committed.root();
    let container = committed.children(root).expect("root should be live")[0];
    committed
        .fragment(container, ITEMS)
        .expect("items fragment should be live")
}

fn key_for(runtime: &UiRuntime, node: NodeId) -> Option<SpatialNodeKeyV2> {
    runtime
        .committed()
        .spatial()
        .expect("spatial state should exist")
        .spatial_key(node)
}

#[test]
fn final_property_value_rebuilds_one_fresh_snapshot_and_geometry() {
    let (mut runtime, program, engine) = dynamic_runtime();
    let before = runtime.committed();
    let root = before.root();
    let container = before.children(root).expect("root should be live")[0];
    let control = before
        .children(container)
        .expect("container should be live")[0];
    let old_snapshot = before
        .spatial()
        .expect("spatial state should exist")
        .snapshot();
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(control, WIDTH, PropertyValue::ScalarI32(41))
        .expect("first property value should stage");
    transaction
        .set_property(control, WIDTH, PropertyValue::ScalarI32(43))
        .expect("final property value should stage");

    let receipt = runtime
        .commit(transaction)
        .expect("coalesced property should publish");
    let after = runtime.committed();
    let spatial = after.spatial().expect("spatial state should remain");
    let key = spatial
        .spatial_key(control)
        .expect("control should remain mapped");
    let row = spatial.snapshot().output().geometry()[key.get() as usize];
    let facts = program.facts();
    let last = facts.last().expect("rebuild fact should exist");
    let index = last
        .nodes
        .iter()
        .position(|candidate| *candidate == control)
        .expect("control fact should exist");

    assert_eq!(program.calls(), 2);
    assert_eq!(engine.calls(), 2);
    assert_eq!(last.widths[index], 43);
    assert_eq!(logical(row.base_width()), 43);
    assert!(!ptr::eq(old_snapshot, spatial.snapshot()));
    assert_eq!(receipt.mutations().len(), 1);
}

#[test]
fn keyed_insert_move_and_remove_rebuild_the_exact_accepted_mapping() {
    let (mut runtime, program, engine) = dynamic_runtime();
    let fragment = items(&runtime);
    let initial = runtime.committed();
    let first = initial
        .keyed_member(fragment, FIRST_KEY)
        .expect("first item should exist");
    let second = initial
        .keyed_member(fragment, SECOND_KEY)
        .expect("second item should exist");
    assert_eq!(key_for(&runtime, second), Some(SpatialNodeKeyV2::new(1)));
    assert_eq!(key_for(&runtime, first), Some(SpatialNodeKeyV2::new(2)));
    drop(initial);

    let before_insert = runtime.committed();
    let old_snapshot = before_insert
        .spatial()
        .expect("spatial state should exist")
        .snapshot();
    let mut insert = runtime.begin_transaction();
    insert
        .insert_keyed(fragment, INSERTED_KEY, 1)
        .expect("keyed insert should stage");
    drop(runtime.commit(insert).expect("keyed insert should publish"));
    let after_insert = runtime.committed();
    let inserted = after_insert
        .keyed_member(fragment, INSERTED_KEY)
        .expect("inserted item should exist");
    let inserted_key = after_insert
        .spatial()
        .expect("spatial state should exist")
        .spatial_key(inserted)
        .expect("inserted item should be mapped");
    assert_eq!(inserted_key, SpatialNodeKeyV2::new(2));
    assert_eq!(
        after_insert.property(inserted, HEIGHT),
        Some(&PropertyValue::ScalarI32(17))
    );
    assert_eq!(
        dynamic_color(&program, inserted),
        ITEM_STYLE_COLOR,
        "new spatial member must use the retained style program"
    );
    assert_eq!(
        dynamic_scalar(&program, inserted, |fact| &fact.heights),
        17,
        "the callback must observe the inserted member's styled height"
    );
    assert_eq!(
        before_insert
            .spatial()
            .expect("old spatial state should exist")
            .spatial_key(inserted),
        None
    );
    assert!(!ptr::eq(
        old_snapshot,
        after_insert
            .spatial()
            .expect("new spatial state should exist")
            .snapshot()
    ));
    drop(before_insert);
    drop(after_insert);

    let before_move = runtime.committed();
    let old_key = before_move
        .spatial()
        .expect("spatial state should exist")
        .spatial_key(inserted);
    let mut movement = runtime.begin_transaction();
    movement
        .move_keyed(fragment, INSERTED_KEY, 2)
        .expect("keyed move should stage");
    drop(runtime.commit(movement).expect("keyed move should publish"));
    let after_move = runtime.committed();
    let new_key = after_move
        .spatial()
        .expect("spatial state should exist")
        .spatial_key(inserted);
    assert_eq!(old_key, Some(SpatialNodeKeyV2::new(2)));
    assert_eq!(new_key, Some(SpatialNodeKeyV2::new(1)));
    assert_eq!(
        after_move
            .spatial()
            .expect("new spatial state should exist")
            .logical_node(SpatialNodeKeyV2::new(1)),
        Some(inserted)
    );
    assert_eq!(
        before_move
            .spatial()
            .expect("old spatial state should exist")
            .logical_node(SpatialNodeKeyV2::new(2)),
        Some(inserted)
    );
    drop(before_move);
    drop(after_move);

    let before_remove = runtime.committed();
    let old_second_key = before_remove
        .spatial()
        .expect("spatial state should exist")
        .spatial_key(second)
        .expect("second item should still be mapped");
    let mut removal = runtime.begin_transaction();
    removal
        .remove_keyed(fragment, SECOND_KEY)
        .expect("keyed removal should stage");
    drop(
        runtime
            .commit(removal)
            .expect("keyed removal should publish"),
    );
    let after_remove = runtime.committed();
    assert_eq!(after_remove.template(second), None);
    assert_eq!(
        after_remove
            .spatial()
            .expect("spatial state should exist")
            .spatial_key(second),
        None
    );
    let after_remove_spatial = after_remove.spatial().expect("spatial state should exist");
    for key in 1..=u32::try_from(after_remove.node_count() - 1).expect("fixture count fits") {
        assert_ne!(
            after_remove_spatial.logical_node(SpatialNodeKeyV2::new(key)),
            Some(second)
        );
    }
    assert_eq!(
        before_remove
            .spatial()
            .expect("old spatial state should exist")
            .logical_node(old_second_key),
        Some(second)
    );
    assert_eq!(program.calls(), 4);
    assert_eq!(engine.calls(), 4);

    let final_fact = program.facts().pop().expect("final fact should exist");
    assert_eq!(final_fact.nodes.len(), 4);
    assert_eq!(final_fact.nodes[0], inserted);
    assert_eq!(dynamic_color(&program, inserted), ITEM_STYLE_COLOR);
    let control = final_fact.nodes[2];
    assert_eq!(dynamic_color(&program, control), CONTROL_STYLE_COLOR);
}

fn dynamic_color(program: &DynamicProgramState, node: NodeId) -> [u8; 4] {
    dynamic_scalar(program, node, |fact| &fact.colors)
}

fn dynamic_scalar<T: Copy>(
    program: &DynamicProgramState,
    node: NodeId,
    values: impl FnOnce(&crate::spatial_support::dynamic::DynamicBuildFact) -> &[T],
) -> T {
    let facts = program.facts();
    let last = facts.last().expect("dynamic fact should exist");
    let index = last
        .nodes
        .iter()
        .position(|candidate| *candidate == node)
        .expect("dynamic node should be present");
    values(last)[index]
}

fn logical(value: SpatialScalarV2) -> i64 {
    assert_eq!(value.raw() % SpatialScalarV2::SCALE, 0);
    value.raw() / SpatialScalarV2::SCALE
}
