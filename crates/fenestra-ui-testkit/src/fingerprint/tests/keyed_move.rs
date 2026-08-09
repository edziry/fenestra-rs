use super::*;
use crate::case::SemanticOperationV1;
use crate::desired::DesiredStateV1;
use crate::fixture::RuntimeOracleFixtureV1;
use crate::model::clean_rebuild_v1;

#[test]
fn keyed_move_reorders_preorder_but_reports_fragment_keyed_order() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let limits = fixture.harness_limits();
    let primary = FragmentPathV1::new(NodePathV1::root(), 1);
    let mut observed_desired = DesiredStateV1::from_construction(fixture.construction(), limits)
        .expect("desired state should initialize");
    observed_desired
        .apply_operation(
            &SemanticOperationV1::InsertKeyed {
                fragment: primary.clone(),
                key: 9,
                final_index: 2,
            },
            limits,
        )
        .expect("insert should update observed model");
    for (key, value) in [(7, 70), (8, 80), (9, 90)] {
        observed_desired
            .apply_operation(
                &SemanticOperationV1::UpdateKeyed {
                    fragment: primary.clone(),
                    key,
                    property: fenestra_ui_ir::prototype::PropertyId::new(0),
                    value: fenestra_ui_ir::prototype::PropertyValue::ScalarI32(value),
                },
                limits,
            )
            .expect("member values should distinguish semantic paths");
    }
    let mut expected_desired = observed_desired.clone();
    expected_desired
        .apply_operation(
            &SemanticOperationV1::MoveKeyed {
                fragment: primary.clone(),
                key: 9,
                final_index: 0,
            },
            limits,
        )
        .expect("move should update expected model");

    let expected = clean_rebuild_v1(fixture.construction(), &expected_desired, limits)
        .expect("expected state should rebuild");
    let observed = clean_rebuild_v1(fixture.construction(), &observed_desired, limits)
        .expect("observed state should rebuild");
    let expected_preorder: Vec<_> = expected
        .nodes()
        .iter()
        .map(|node| node.path().clone())
        .collect();
    let observed_preorder: Vec<_> = observed
        .nodes()
        .iter()
        .map(|node| node.path().clone())
        .collect();
    assert_ne!(expected_preorder, observed_preorder);
    let mut expected_paths = expected_preorder;
    let mut observed_paths = observed_preorder;
    expected_paths.sort();
    observed_paths.sort();
    assert_eq!(expected_paths, observed_paths);
    for (key, value) in [(7, 70), (8, 80), (9, 90)] {
        let path = NodePathV1::root().member(1, key);
        for state in [&expected, &observed] {
            let property = state
                .node(&path)
                .expect("member path should remain present")
                .properties()
                .first()
                .expect("member should retain its value property");
            assert_eq!(
                property.value(),
                &fenestra_ui_ir::prototype::PropertyValue::ScalarI32(value)
            );
        }
    }

    assert_state_mismatch(
        &expected,
        &observed,
        FingerprintLocationV1::Fragment(primary),
        FingerprintFieldV1::KeyedOrder,
        FingerprintSummaryV1::Keys(vec![9, 7, 8]),
        FingerprintSummaryV1::Keys(vec![7, 8, 9]),
    );
}
