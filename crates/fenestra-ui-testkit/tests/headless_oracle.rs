#[path = "headless/fixture_support.rs"]
mod support;

use fenestra_ui_ir::prototype::{InputPolicy, PropertyValue};
use fenestra_ui_runtime::prototype::{
    HeadlessRect, HeadlessSemanticAction, HeadlessSemanticRole, HeadlessSurface,
};
use fenestra_ui_testkit::prototype::{
    HarnessErrorKind, HeadlessFixtureV1, HeadlessOracleV1, SemanticOperationV1,
    compare_headless_projection_v1, observe_headless_projection_v1,
};

use support::{
    COLOR, FIRST_KEY, HEIGHT, INSERTED_KEY, SECOND_KEY, apply_operation, assert_oracle_matches,
    computed_tuples, container_path, control_path, geometry_tuples, item_path, items_path, node_id,
    path_order, rgba, root_path, runtime,
};

fn rect(x: i32, y: i32, width: i32, height: i32) -> HeadlessRect {
    HeadlessRect::new(x, y, width, height)
}

#[test]
fn clean_rebuild_matches_the_complete_manual_generation_zero_projection() {
    let fixture = HeadlessFixtureV1::build().expect("registered headless fixture should validate");
    let oracle = HeadlessOracleV1::new(&fixture).expect("initial desired state should rebuild");
    let expected = oracle.rebuild().expect("clean rebuild should succeed");

    assert_eq!(expected.surface(), HeadlessSurface::new(120, 90));
    assert_eq!(
        computed_tuples(&expected),
        vec![
            (
                root_path(),
                100,
                80,
                [1, 1, 1, 255],
                true,
                InputPolicy::Ignore
            ),
            (
                container_path(),
                80,
                50,
                [2, 2, 2, 255],
                true,
                InputPolicy::Ignore,
            ),
            (
                control_path(),
                30,
                10,
                [10, 20, 30, 255],
                true,
                InputPolicy::Accept,
            ),
            (
                item_path(FIRST_KEY),
                40,
                12,
                [80, 90, 100, 255],
                true,
                InputPolicy::Accept,
            ),
            (
                item_path(SECOND_KEY),
                40,
                12,
                [80, 90, 100, 255],
                true,
                InputPolicy::Accept,
            ),
        ]
    );
    assert_eq!(
        geometry_tuples(&expected),
        vec![
            (root_path(), rect(0, 0, 100, 80), rect(0, 0, 100, 80)),
            (container_path(), rect(0, 0, 80, 50), rect(0, 0, 80, 50),),
            (control_path(), rect(0, 0, 30, 10), rect(0, 0, 30, 10),),
            (
                item_path(FIRST_KEY),
                rect(0, 10, 40, 12),
                rect(0, 10, 40, 12),
            ),
            (
                item_path(SECOND_KEY),
                rect(0, 22, 40, 12),
                rect(0, 22, 40, 12),
            ),
        ]
    );
    assert_eq!(
        expected
            .semantics()
            .iter()
            .map(|record| (
                record.path().clone(),
                record.role(),
                record.label(),
                record.action(),
            ))
            .collect::<Vec<_>>(),
        vec![(
            control_path(),
            HeadlessSemanticRole::Control,
            1,
            HeadlessSemanticAction::Activate,
        )]
    );
    assert_eq!(
        expected
            .hit_regions()
            .iter()
            .map(|record| (record.path().clone(), record.clip()))
            .collect::<Vec<_>>(),
        vec![
            (control_path(), rect(0, 0, 30, 10)),
            (item_path(FIRST_KEY), rect(0, 10, 40, 12)),
            (item_path(SECOND_KEY), rect(0, 22, 40, 12)),
        ]
    );
    assert_eq!(
        expected
            .scene_rectangles()
            .iter()
            .map(|record| (record.path().clone(), record.rectangle(), record.color()))
            .collect::<Vec<_>>(),
        vec![
            (root_path(), rect(0, 0, 100, 80), [1, 1, 1, 255]),
            (container_path(), rect(0, 0, 80, 50), [2, 2, 2, 255],),
            (control_path(), rect(0, 0, 30, 10), [10, 20, 30, 255],),
            (
                item_path(FIRST_KEY),
                rect(0, 10, 40, 12),
                [80, 90, 100, 255],
            ),
            (
                item_path(SECOND_KEY),
                rect(0, 22, 40, 12),
                [80, 90, 100, 255],
            ),
        ]
    );

    let runtime = runtime(&fixture);
    let snapshot = runtime.committed();
    let observed = assert_oracle_matches(&fixture, &oracle, &snapshot);
    assert_eq!(observed.generation().get(), 0);
}

#[test]
fn clean_rebuild_tracks_every_logical_operation_resize_and_semantic_identity() {
    let fixture = HeadlessFixtureV1::build().expect("registered headless fixture should validate");
    let mut oracle = HeadlessOracleV1::new(&fixture).expect("initial desired state should rebuild");
    let mut runtime = runtime(&fixture);
    let mut snapshot = runtime.committed();
    let control_id = node_id(&snapshot, &control_path());
    let first_id = node_id(&snapshot, &item_path(FIRST_KEY));
    let second_id = node_id(&snapshot, &item_path(SECOND_KEY));

    snapshot = apply_operation(
        &fixture,
        &mut runtime,
        &mut oracle,
        &SemanticOperationV1::SetProperty {
            node: control_path(),
            property: COLOR,
            value: rgba([20, 30, 40, 255]),
        },
    );
    assert_eq!(node_id(&snapshot, &control_path()), control_id);

    snapshot = apply_operation(
        &fixture,
        &mut runtime,
        &mut oracle,
        &SemanticOperationV1::InsertKeyed {
            fragment: items_path(),
            key: INSERTED_KEY,
            final_index: 1,
        },
    );
    let inserted_id = node_id(&snapshot, &item_path(INSERTED_KEY));

    snapshot = apply_operation(
        &fixture,
        &mut runtime,
        &mut oracle,
        &SemanticOperationV1::MoveKeyed {
            fragment: items_path(),
            key: SECOND_KEY,
            final_index: 0,
        },
    );
    let observed = observe_headless_projection_v1(&fixture, &snapshot)
        .expect("moved projection should normalize");
    let authored = vec![
        root_path(),
        container_path(),
        control_path(),
        item_path(SECOND_KEY),
        item_path(FIRST_KEY),
        item_path(INSERTED_KEY),
    ];
    assert_eq!(
        path_order(
            observed
                .projection()
                .computed_styles()
                .iter()
                .map(|record| record.path())
        ),
        authored
    );
    assert_eq!(
        path_order(
            observed
                .projection()
                .hit_regions()
                .iter()
                .map(|record| record.path())
        ),
        vec![
            control_path(),
            item_path(SECOND_KEY),
            item_path(FIRST_KEY),
            item_path(INSERTED_KEY),
        ]
    );
    assert_eq!(
        path_order(
            observed
                .projection()
                .scene_rectangles()
                .iter()
                .map(|record| record.path())
        ),
        authored
    );
    assert_eq!(
        path_order(
            observed
                .projection()
                .geometries()
                .iter()
                .map(|record| record.path())
        ),
        authored
    );
    assert_eq!(node_id(&snapshot, &item_path(FIRST_KEY)), first_id);
    assert_eq!(node_id(&snapshot, &item_path(SECOND_KEY)), second_id);
    assert_eq!(node_id(&snapshot, &item_path(INSERTED_KEY)), inserted_id);

    snapshot = apply_operation(
        &fixture,
        &mut runtime,
        &mut oracle,
        &SemanticOperationV1::UpdateKeyed {
            fragment: items_path(),
            key: INSERTED_KEY,
            property: HEIGHT,
            value: PropertyValue::ScalarI32(14),
        },
    );
    assert_eq!(node_id(&snapshot, &item_path(INSERTED_KEY)), inserted_id);

    snapshot = apply_operation(
        &fixture,
        &mut runtime,
        &mut oracle,
        &SemanticOperationV1::RemoveKeyed {
            fragment: items_path(),
            key: FIRST_KEY,
        },
    );
    assert!(snapshot.property(first_id, HEIGHT).is_none());

    let resized = HeadlessSurface::new(90, 70);
    oracle
        .resize(resized)
        .expect("valid desired resize should apply");
    let mut transaction = runtime.begin_transaction();
    transaction
        .resize_headless(resized)
        .expect("valid candidate resize should stage");
    runtime
        .commit(transaction)
        .expect("valid candidate resize should publish");
    snapshot = runtime.committed();
    let observed = assert_oracle_matches(&fixture, &oracle, &snapshot);
    assert_eq!(observed.projection().surface(), resized);
    assert_eq!(node_id(&snapshot, &control_path()), control_id);
    assert_eq!(node_id(&snapshot, &item_path(SECOND_KEY)), second_id);
    assert_eq!(node_id(&snapshot, &item_path(INSERTED_KEY)), inserted_id);
}

#[test]
fn invalid_desired_operation_rolls_back_the_oracle_draft() {
    let fixture = HeadlessFixtureV1::build().expect("registered headless fixture should validate");
    let mut oracle = HeadlessOracleV1::new(&fixture).expect("initial desired state should rebuild");
    let before = oracle.rebuild().expect("initial rebuild should succeed");
    let invalid = SemanticOperationV1::RemoveKeyed {
        fragment: items_path(),
        key: u64::MAX,
    };

    let error = oracle
        .apply_operation(&invalid)
        .expect_err("missing desired key should fail");

    assert_eq!(error.kind(), HarnessErrorKind::InvalidOperation);
    assert_eq!(
        oracle
            .rebuild()
            .expect("failed operation must not corrupt state"),
        before
    );
}

#[test]
fn surface_mismatch_precedes_the_five_projection_families() {
    let fixture = HeadlessFixtureV1::build().expect("registered headless fixture should validate");
    let mut oracle = HeadlessOracleV1::new(&fixture).expect("initial desired state should rebuild");
    oracle
        .resize(HeadlessSurface::new(90, 70))
        .expect("valid desired resize should apply");
    let expected = oracle.rebuild().expect("resized rebuild should succeed");
    let runtime = runtime(&fixture);
    let snapshot = runtime.committed();
    let observed = observe_headless_projection_v1(&fixture, &snapshot)
        .expect("candidate projection should normalize");

    let error = compare_headless_projection_v1(&expected, observed.projection())
        .expect_err("surface mismatch should be outside projection families");

    assert_eq!(error.kind(), HarnessErrorKind::StateMismatch);
}
