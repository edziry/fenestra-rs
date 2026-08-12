use fenestra_ui_ir::prototype::{InputPolicy, PropertyValue};
use fenestra_ui_runtime::prototype::{CommittedRuntimeSnapshot, NodeId};
use fenestra_ui_spatial::prototype::{
    ReferenceRasterLimitsV2, SpatialNodeKeyV2, SpatialViewportV2,
};

use crate::generation::{pixel, point};
use crate::new_ir_with_engine;
use crate::spatial_support::engine::{EnginePlan, EngineSpy};
use crate::support::spatial_ir::{
    COLOR, INNER_REGION, INSERTED_KEY, LogicalNodes, POLICY, SECOND_KEY, STYLED_COLOR, VIEWPORT,
    WIDTH, capacity, fixture, inner, limits,
};

#[test]
fn keyed_insert_move_and_remove_rebuild_one_complete_dense_expansion_each() {
    let fixture = fixture();
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Reference);
    let mut runtime = new_ir_with_engine(
        fixture.program,
        VIEWPORT,
        limits(),
        capacity(),
        Box::new(engine),
    )
    .expect("spatial IR runtime should initialize");
    let before = runtime.committed();
    let initial = LogicalNodes::capture(&before);
    assert_mapping(
        &before,
        &[
            initial.first_outer,
            initial.first_inner,
            initial.second_outer,
            initial.second_inner,
        ],
    );

    let mut insert = runtime.begin_transaction();
    insert
        .insert_keyed(initial.outer_fragment, INSERTED_KEY, 1)
        .expect("middle insertion should stage");
    runtime.commit(insert).expect("insertion should publish");
    let inserted_snapshot = runtime.committed();
    let inserted_outer = inserted_snapshot
        .keyed_member(initial.outer_fragment, INSERTED_KEY)
        .expect("inserted outer member should be live");
    let inserted_inner = inner(&inserted_snapshot, inserted_outer);
    assert_mapping(
        &inserted_snapshot,
        &[
            initial.first_outer,
            initial.first_inner,
            inserted_outer,
            inserted_inner,
            initial.second_outer,
            initial.second_inner,
        ],
    );
    assert_eq!(engine_state.calls(), 2);

    let mut movement = runtime.begin_transaction();
    movement
        .move_keyed(initial.outer_fragment, SECOND_KEY, 0)
        .expect("outer move should stage");
    runtime.commit(movement).expect("outer move should publish");
    let moved = runtime.committed();
    assert_mapping(
        &moved,
        &[
            initial.second_outer,
            initial.second_inner,
            initial.first_outer,
            initial.first_inner,
            inserted_outer,
            inserted_inner,
        ],
    );
    assert_eq!(engine_state.calls(), 3);

    let mut removal = runtime.begin_transaction();
    removal
        .remove_keyed(initial.outer_fragment, INSERTED_KEY)
        .expect("inserted member removal should stage");
    runtime.commit(removal).expect("removal should publish");
    let removed = runtime.committed();
    assert_mapping(
        &removed,
        &[
            initial.second_outer,
            initial.second_inner,
            initial.first_outer,
            initial.first_inner,
        ],
    );
    assert_eq!(removed.template(inserted_outer), None);
    assert_eq!(removed.fragment(inserted_outer, INNER_REGION), None);
    assert_eq!(engine_state.calls(), 4);

    assert_mapping(
        &before,
        &[
            initial.first_outer,
            initial.first_inner,
            initial.second_outer,
            initial.second_inner,
        ],
    );
}

#[test]
fn property_rebuild_reads_the_draft_and_retains_the_old_spatial_snapshot() {
    let fixture = fixture();
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Reference);
    let mut runtime = new_ir_with_engine(
        fixture.program,
        VIEWPORT,
        limits(),
        capacity(),
        Box::new(engine),
    )
    .expect("spatial IR runtime should initialize");
    let before = runtime.committed();
    let logical = LogicalNodes::capture(&before);
    let before_spatial = before.spatial().expect("spatial state should exist");
    let before_raster = before_spatial
        .snapshot()
        .rasterize_reference(ReferenceRasterLimitsV2::new(1_200))
        .expect("old raster should resolve");
    assert_eq!(pixel(&before_raster, 2, 2), STYLED_COLOR);
    assert!(before_spatial.snapshot().hit_test(point(2, 2)).is_some());

    let changed_color = [80, 50, 20, 255];
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(logical.first_outer, WIDTH, PropertyValue::ScalarI32(14))
        .expect("width update should stage");
    transaction
        .set_property(
            logical.first_outer,
            COLOR,
            PropertyValue::Rgba8(changed_color),
        )
        .expect("color update should stage");
    transaction
        .set_property(
            logical.first_outer,
            POLICY,
            PropertyValue::InputPolicy(InputPolicy::Ignore),
        )
        .expect("input-policy update should stage");
    runtime
        .commit(transaction)
        .expect("property rebuild should publish");
    let after = runtime.committed();
    let after_snapshot = after
        .spatial()
        .expect("spatial state should remain")
        .snapshot();
    let after_raster = after_snapshot
        .rasterize_reference(ReferenceRasterLimitsV2::new(1_200))
        .expect("new raster should resolve");

    assert_eq!(
        logical_scalar(after_snapshot.output().geometry()[1].base_width()),
        14
    );
    assert_eq!(pixel(&after_raster, 2, 2), changed_color);
    assert_eq!(after_snapshot.hit_test(point(2, 2)), None);
    assert!(after_snapshot.hit_test(point(2, 7)).is_some());
    assert_eq!(engine_state.calls(), 2);
    assert_eq!(
        logical_scalar(before_spatial.snapshot().output().geometry()[1].base_width()),
        12
    );
    assert_eq!(pixel(&before_raster, 2, 2), STYLED_COLOR);
    assert!(before_spatial.snapshot().hit_test(point(2, 2)).is_some());

    let stable = runtime.committed();
    assert!(
        runtime
            .commit(runtime.begin_transaction())
            .unwrap()
            .is_empty()
    );
    let mut same = runtime.begin_transaction();
    same.set_property(logical.first_outer, WIDTH, PropertyValue::ScalarI32(14))
        .expect("same width should stage");
    assert!(runtime.commit(same).unwrap().is_empty());
    let mut round_trip = runtime.begin_transaction();
    round_trip
        .set_property(logical.first_outer, WIDTH, PropertyValue::ScalarI32(15))
        .unwrap();
    round_trip
        .set_property(logical.first_outer, WIDTH, PropertyValue::ScalarI32(14))
        .unwrap();
    assert!(runtime.commit(round_trip).unwrap().is_empty());
    assert!(stable.shares_state_with(&runtime.committed()));
    assert_eq!(engine_state.calls(), 2);
}

#[test]
fn resize_rebuilds_once_while_round_trips_skip_materialization() {
    let fixture = fixture();
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Reference);
    let mut runtime = new_ir_with_engine(
        fixture.program,
        VIEWPORT,
        limits(),
        capacity(),
        Box::new(engine),
    )
    .expect("spatial IR runtime should initialize");
    let before = runtime.committed();
    let before_snapshot = before
        .spatial()
        .expect("spatial state should exist")
        .snapshot();
    let resized = SpatialViewportV2::new(50, 35);
    let mut resize = runtime.begin_transaction();
    resize.resize_spatial(resized).expect("resize should stage");
    runtime.commit(resize).expect("resize should publish");
    let after = runtime.committed();
    let after_snapshot = after
        .spatial()
        .expect("spatial state should remain")
        .snapshot();

    assert_eq!(before_snapshot.viewport(), VIEWPORT);
    assert_eq!(after_snapshot.viewport(), resized);
    assert_eq!(
        (
            logical_scalar(after_snapshot.output().geometry()[0].base_width()),
            logical_scalar(after_snapshot.output().geometry()[0].base_height()),
        ),
        (50, 35)
    );
    assert_eq!(engine_state.calls(), 2);

    let stable = runtime.committed();
    let mut round_trip = runtime.begin_transaction();
    round_trip
        .resize_spatial(SpatialViewportV2::new(55, 40))
        .expect("intermediate resize should stage");
    round_trip
        .resize_spatial(resized)
        .expect("original resize should stage");
    assert!(runtime.commit(round_trip).unwrap().is_empty());
    assert!(stable.shares_state_with(&runtime.committed()));
    assert_eq!(engine_state.calls(), 2);
}

fn assert_mapping(committed: &CommittedRuntimeSnapshot, expected: &[NodeId]) {
    let spatial = committed.spatial().expect("spatial state should exist");
    assert_eq!(spatial.logical_node(SpatialNodeKeyV2::new(0)), None);
    for (index, node) in expected.iter().copied().enumerate() {
        let key = SpatialNodeKeyV2::new(u32::try_from(index + 1).expect("fixture key should fit"));
        assert_eq!(spatial.logical_node(key), Some(node));
        assert_eq!(spatial.spatial_key(node), Some(key));
    }
    assert_eq!(
        spatial.logical_node(SpatialNodeKeyV2::new(
            u32::try_from(expected.len() + 1).expect("fixture key should fit")
        )),
        None
    );
}

fn logical_scalar(value: fenestra_ui_spatial::prototype::SpatialScalarV2) -> i32 {
    i32::try_from(value.raw() / fenestra_ui_spatial::prototype::SpatialScalarV2::SCALE)
        .expect("fixture scalar should fit")
}
