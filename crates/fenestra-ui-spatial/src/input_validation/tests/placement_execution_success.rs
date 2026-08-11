use super::local_transform_support::{fixed, layout_node, root as transform_root, transform};
use super::placement_execution_support::{
    ScriptedLayoutEngine, VIEWPORT, expect_valid, fixture, free, limits, logical, node_target,
    output, placement, root,
};
use crate::model::{
    SpatialAnchorComponentV2, SpatialAnchorTargetV2, SpatialScalarV2, SpatialViewportV2,
};

#[test]
fn root_only_execution_returns_a_distinct_proof_without_calling_layout() {
    let fixture = fixture(vec![root()]);
    let engine = ScriptedLayoutEngine::new(Vec::new());
    let proof = expect_valid(execute_dependency_graph!(
        &fixture,
        VIEWPORT,
        limits(),
        &engine
    ));

    assert_eq!(
        proof.placement_facts(),
        vec![placement(0, 0, 0, 20, 20, logical(20), logical(20), 0, 0,)]
    );
    assert!(proof.dependency_unit_facts().is_empty());
    assert_eq!(engine.call_count(), 0);
}

#[test]
fn all_anchor_kinds_forward_targets_and_fractional_offsets_resolve_exactly() {
    use SpatialAnchorComponentV2::{Center, End, Start};

    let half = SpatialScalarV2::SCALE / 2;
    let fixture = fixture(vec![
        root(),
        free(
            1,
            0,
            5,
            7,
            Center,
            Center,
            node_target(4),
            Center,
            Center,
            half,
            -half,
        ),
        free(
            2,
            0,
            2,
            2,
            Start,
            Start,
            SpatialAnchorTargetV2::Viewport,
            End,
            Start,
            0,
            0,
        ),
        free(
            3,
            2,
            1,
            1,
            End,
            Start,
            SpatialAnchorTargetV2::Parent,
            End,
            End,
            0,
            0,
        ),
        free(
            4,
            0,
            10,
            10,
            Start,
            Start,
            SpatialAnchorTargetV2::Viewport,
            Start,
            Start,
            0,
            0,
        ),
    ]);
    let engine = ScriptedLayoutEngine::new(Vec::new());
    let proof = expect_valid(execute_dependency_graph!(
        &fixture,
        VIEWPORT,
        limits(),
        &engine
    ));

    assert_eq!(proof.dependency_order_facts(), vec![2, 3, 4, 1]);
    assert_eq!(
        proof.placement_facts(),
        vec![
            placement(0, 0, 0, 20, 20, logical(20), logical(20), 0, 0),
            placement(
                1,
                logical(3),
                logical(1),
                5,
                7,
                logical(8),
                logical(8),
                logical(3),
                logical(1)
            ),
            placement(
                2,
                logical(20),
                0,
                2,
                2,
                logical(22),
                logical(2),
                logical(20),
                0
            ),
            placement(
                3,
                logical(21),
                logical(2),
                1,
                1,
                logical(22),
                logical(3),
                logical(1),
                logical(2)
            ),
            placement(4, 0, 0, 10, 10, logical(10), logical(10), 0, 0),
        ]
    );
    assert_eq!(engine.call_count(), 0);
}

#[test]
fn placement_execution_stops_before_world_transform_composition() {
    let tiny = transform([1, 0, 0, 1, 0, 0, 0, 0]);
    let fixture = fixture(vec![
        transform_root(),
        layout_node(1, 0, fixed(10), fixed(10), tiny),
        layout_node(2, 1, fixed(10), fixed(10), tiny),
    ]);
    let engine = ScriptedLayoutEngine::new(vec![Ok(output(&[
        (0, 0, 0, 20, 20),
        (1, 0, 0, 10, 10),
        (2, 0, 10, 10, 10),
    ]))]);
    let proof = expect_valid(execute_dependency_graph!(
        &fixture,
        SpatialViewportV2::new(20, 20),
        limits(),
        &engine
    ));

    assert_eq!(proof.placement_facts().len(), 3);
    assert_eq!(engine.call_count(), 1);
}
