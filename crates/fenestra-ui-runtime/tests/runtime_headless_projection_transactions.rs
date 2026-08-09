#[path = "support/headless_projection.rs"]
mod headless_projection;
mod support;

use fenestra_ui_ir::prototype::{InputPolicy, PropertyValue};
use fenestra_ui_runtime::prototype::{
    CommittedRuntimeSnapshot, HeadlessSurface, UiRuntime, UiTransaction,
};

use headless_projection::{ProjectionNodes, nodes, rect, runtime};
use support::headless::{
    COLOR, DIRECT_COLOR, HEIGHT, INSERTED_KEY, ITEM_STYLE_COLOR, SECOND_KEY, VISIBLE,
};
use support::headless_projection_state::{
    ComputedRecord, GeometryRecord, HitRecord, ProjectionRecords, SceneRecord, capture_projection,
};

fn publish(
    runtime: &mut UiRuntime,
    before: &CommittedRuntimeSnapshot,
    transaction: UiTransaction,
    update_expected: impl FnOnce(&CommittedRuntimeSnapshot, &mut ProjectionRecords),
) -> CommittedRuntimeSnapshot {
    let retained = capture_projection(before);
    let receipt = runtime
        .commit(transaction)
        .expect("valid projection transaction should publish");
    let after = runtime.committed();
    let mut expected = retained.clone();
    expected.generation = after.generation();
    update_expected(&after, &mut expected);

    assert_eq!(receipt.generation(), after.generation());
    assert_eq!(after.generation().get(), before.generation().get() + 1);
    assert_eq!(capture_projection(&after), expected);
    assert_eq!(capture_projection(before), retained);
    assert!(!before.shares_state_with(&after));
    after
}

fn move_record<T>(records: &mut Vec<T>, old_index: usize, final_index: usize) {
    let record = records.remove(old_index);
    records.insert(final_index, record);
}

fn insert_expected_item(
    after: &CommittedRuntimeSnapshot,
    expected: &mut ProjectionRecords,
    nodes: ProjectionNodes,
) {
    let inserted = after
        .keyed_member(nodes.items, INSERTED_KEY)
        .expect("inserted item should exist");
    expected.computed.insert(
        4,
        ComputedRecord {
            node: inserted,
            width: 40,
            height: 12,
            color: ITEM_STYLE_COLOR,
            visible: true,
            input: InputPolicy::Accept,
        },
    );
    expected.geometry.insert(
        4,
        GeometryRecord {
            node: inserted,
            bounds: rect(0, 22, 40, 12),
            clip: rect(0, 22, 40, 12),
        },
    );
    expected.geometry_mut(nodes.second).bounds = rect(0, 34, 40, 12);
    expected.geometry_mut(nodes.second).clip = rect(0, 34, 40, 12);
    expected.hits.insert(
        2,
        HitRecord {
            node: inserted,
            clip: rect(0, 22, 40, 12),
        },
    );
    expected.hit_mut(nodes.second).clip = rect(0, 34, 40, 12);
    expected.scenes.insert(
        4,
        SceneRecord {
            node: inserted,
            rectangle: rect(0, 22, 40, 12),
            color: ITEM_STYLE_COLOR,
        },
    );
    expected.scene_mut(nodes.second).rectangle = rect(0, 34, 40, 12);
}

#[test]
fn every_logical_mutation_rebuilds_complete_authored_projection_records() {
    let mut runtime = runtime(HeadlessSurface::new(120, 90));
    let initial = runtime.committed();
    let initial_projection = capture_projection(&initial);
    let nodes = nodes(&initial);
    let mut current = initial.clone();

    let mut direct = runtime.begin_transaction();
    direct
        .set_property(nodes.control, COLOR, PropertyValue::Rgba8(DIRECT_COLOR))
        .expect("direct color should stage");
    current = publish(&mut runtime, &current, direct, |_, expected| {
        expected.computed_mut(nodes.control).color = DIRECT_COLOR;
        expected.scene_mut(nodes.control).color = DIRECT_COLOR;
    });

    let mut insert = runtime.begin_transaction();
    insert
        .insert_keyed(nodes.items, INSERTED_KEY, 1)
        .expect("middle insertion should stage");
    current = publish(&mut runtime, &current, insert, |after, expected| {
        insert_expected_item(after, expected, nodes);
    });
    let inserted = current
        .keyed_member(nodes.items, INSERTED_KEY)
        .expect("inserted item should exist");

    let mut movement = runtime.begin_transaction();
    movement
        .move_keyed(nodes.items, INSERTED_KEY, 2)
        .expect("movement should stage");
    current = publish(&mut runtime, &current, movement, |_, expected| {
        move_record(&mut expected.computed, 4, 5);
        move_record(&mut expected.geometry, 4, 5);
        move_record(&mut expected.hits, 2, 3);
        move_record(&mut expected.scenes, 4, 5);
        expected.geometry_mut(nodes.second).bounds = rect(0, 22, 40, 12);
        expected.geometry_mut(nodes.second).clip = rect(0, 22, 40, 12);
        expected.geometry_mut(inserted).bounds = rect(0, 34, 40, 12);
        expected.geometry_mut(inserted).clip = rect(0, 34, 40, 12);
        expected.hit_mut(nodes.second).clip = rect(0, 22, 40, 12);
        expected.hit_mut(inserted).clip = rect(0, 34, 40, 12);
        expected.scene_mut(nodes.second).rectangle = rect(0, 22, 40, 12);
        expected.scene_mut(inserted).rectangle = rect(0, 34, 40, 12);
    });

    let mut update = runtime.begin_transaction();
    update
        .update_keyed(
            nodes.items,
            INSERTED_KEY,
            HEIGHT,
            PropertyValue::ScalarI32(14),
        )
        .expect("keyed height should stage");
    current = publish(&mut runtime, &current, update, |_, expected| {
        expected.computed_mut(inserted).height = 14;
        expected.geometry_mut(inserted).bounds = rect(0, 34, 40, 14);
        expected.geometry_mut(inserted).clip = rect(0, 34, 40, 14);
        expected.hit_mut(inserted).clip = rect(0, 34, 40, 14);
        expected.scene_mut(inserted).rectangle = rect(0, 34, 40, 14);
    });

    let mut removal = runtime.begin_transaction();
    removal
        .remove_keyed(nodes.items, SECOND_KEY)
        .expect("removal should stage");
    current = publish(&mut runtime, &current, removal, |_, expected| {
        expected.remove_node(nodes.second);
        expected.geometry_mut(inserted).bounds = rect(0, 22, 40, 14);
        expected.geometry_mut(inserted).clip = rect(0, 22, 40, 14);
        expected.hit_mut(inserted).clip = rect(0, 22, 40, 14);
        expected.scene_mut(inserted).rectangle = rect(0, 22, 40, 14);
    });

    assert_eq!(current.generation().get(), 5);
    assert_eq!(capture_projection(&initial), initial_projection);
    assert_eq!(
        current
            .keyed_members(nodes.items)
            .expect("item fragment should exist")
            .map(|member| member.0)
            .collect::<Vec<_>>(),
        vec![support::headless::FIRST_KEY, INSERTED_KEY]
    );
}

#[test]
fn direct_visibility_rebuilds_semantic_hit_and_scene_membership() {
    let mut runtime = runtime(HeadlessSurface::new(120, 90));
    let before = runtime.committed();
    let fixture = nodes(&before);
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(fixture.control, VISIBLE, PropertyValue::Bool(false))
        .expect("control visibility should stage");

    let after = publish(&mut runtime, &before, transaction, |_, expected| {
        expected.computed_mut(fixture.control).visible = false;
        expected
            .semantics
            .retain(|record| record.node != fixture.control);
        expected
            .hits
            .retain(|record| record.node != fixture.control);
        expected
            .scenes
            .retain(|record| record.node != fixture.control);
    });

    assert_eq!(capture_projection(&after).geometry.len(), 5);
    assert_eq!(capture_projection(&before).semantics.len(), 1);
}

#[test]
fn negative_intermediate_dimension_is_ignored_when_the_final_value_is_valid() {
    let mut runtime = runtime(HeadlessSurface::new(120, 90));
    let before = runtime.committed();
    let fixture = nodes(&before);
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(fixture.first, HEIGHT, PropertyValue::ScalarI32(-1))
        .expect("negative intermediate height should stage");
    transaction
        .set_property(fixture.first, HEIGHT, PropertyValue::ScalarI32(14))
        .expect("valid final height should stage");

    let after = publish(&mut runtime, &before, transaction, |_, expected| {
        expected.computed_mut(fixture.first).height = 14;
        expected.geometry_mut(fixture.first).bounds = rect(0, 10, 40, 14);
        expected.geometry_mut(fixture.first).clip = rect(0, 10, 40, 14);
        expected.geometry_mut(fixture.second).bounds = rect(0, 24, 40, 12);
        expected.geometry_mut(fixture.second).clip = rect(0, 24, 40, 12);
        expected.hit_mut(fixture.first).clip = rect(0, 10, 40, 14);
        expected.hit_mut(fixture.second).clip = rect(0, 24, 40, 12);
        expected.scene_mut(fixture.first).rectangle = rect(0, 10, 40, 14);
        expected.scene_mut(fixture.second).rectangle = rect(0, 24, 40, 12);
    });

    assert_eq!(after.generation().get(), 1);
}
