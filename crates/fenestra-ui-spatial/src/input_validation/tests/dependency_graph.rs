use super::dependency_support::{
    DependencyUnitFact, VIEWPORT, dependency_limits, expect_limit, expect_valid, fixture, free,
    layout, node_target, root,
};
use crate::limits::SpatialLimitKindV2;
use crate::model::SpatialAnchorTargetV2;

#[test]
fn mixed_free_and_island_units_retain_exact_producers_edges_and_stable_order() {
    let fixture = fixture(vec![
        root(),
        free(1, 0, SpatialAnchorTargetV2::Viewport),
        layout(2, 1),
        layout(3, 2),
        free(4, 2, node_target(3)),
        layout(5, 4),
        free(6, 0, node_target(3)),
        free(7, 0, node_target(5)),
        layout(8, 0),
    ]);
    let proof = expect_valid(prepare_dependency_graph!(
        &fixture,
        VIEWPORT,
        dependency_limits(7, 5)
    ));

    let expected: Vec<DependencyUnitFact> = vec![
        (1, None, vec![1], vec![]),
        (2, Some((0, 1)), vec![2, 3], vec![1]),
        (4, None, vec![4], vec![2]),
        (5, Some((1, 4)), vec![5], vec![4]),
        (6, None, vec![6], vec![2]),
        (7, None, vec![7], vec![5]),
        (8, Some((2, 0)), vec![8], vec![]),
    ];
    assert_eq!(proof.dependency_unit_facts(), expected);
    assert_eq!(proof.dependency_edge_count(), 5);
    assert_eq!(proof.dependency_order_facts(), vec![1, 2, 4, 5, 6, 7, 8]);
}

#[test]
fn parent_and_equivalent_named_target_edges_are_collapsed_before_counting() {
    let fixture = fixture(vec![
        root(),
        free(1, 0, SpatialAnchorTargetV2::Viewport),
        free(2, 1, SpatialAnchorTargetV2::Parent),
        free(3, 1, node_target(1)),
    ]);
    let proof = expect_valid(prepare_dependency_graph!(
        &fixture,
        VIEWPORT,
        dependency_limits(3, 2)
    ));

    assert_eq!(
        proof.dependency_unit_facts(),
        vec![
            (1, None, vec![1], vec![]),
            (2, None, vec![2], vec![1]),
            (3, None, vec![3], vec![1]),
        ]
    );
    assert_eq!(proof.dependency_edge_count(), 2);

    expect_limit(
        prepare_dependency_graph!(&fixture, VIEWPORT, dependency_limits(3, 1)),
        SpatialLimitKindV2::DependencyEdges,
        2,
        1,
    );
}

#[test]
fn distinct_parent_and_named_target_producers_are_both_retained_in_sorted_order() {
    let fixture = fixture(vec![
        root(),
        free(1, 0, SpatialAnchorTargetV2::Viewport),
        free(2, 1, SpatialAnchorTargetV2::Viewport),
        free(3, 2, node_target(1)),
    ]);
    let proof = expect_valid(prepare_dependency_graph!(
        &fixture,
        VIEWPORT,
        dependency_limits(3, 3)
    ));

    assert_eq!(
        proof.dependency_unit_facts(),
        vec![
            (1, None, vec![1], vec![]),
            (2, None, vec![2], vec![1]),
            (3, None, vec![3], vec![1, 2]),
        ]
    );
    assert_eq!(proof.dependency_edge_count(), 3);
    assert_eq!(proof.dependency_order_facts(), vec![1, 2, 3]);
}

#[test]
fn noncontiguous_island_members_share_one_retained_producer() {
    let fixture = fixture(vec![
        root(),
        layout(1, 0),
        free(2, 1, node_target(3)),
        layout(3, 1),
    ]);
    let proof = expect_valid(prepare_dependency_graph!(
        &fixture,
        VIEWPORT,
        dependency_limits(2, 1)
    ));

    assert_eq!(
        proof.dependency_unit_facts(),
        vec![
            (1, Some((0, 0)), vec![1, 3], vec![]),
            (2, None, vec![2], vec![1]),
        ]
    );
    assert_eq!(proof.dependency_edge_count(), 1);
    assert_eq!(proof.dependency_order_facts(), vec![1, 2]);
}

#[test]
fn root_viewport_and_root_host_are_sources_not_units_or_edges() {
    let root_only = fixture(vec![root()]);
    let empty = expect_valid(prepare_dependency_graph!(
        &root_only,
        VIEWPORT,
        dependency_limits(0, 0)
    ));
    assert!(empty.dependency_unit_facts().is_empty());
    assert!(empty.dependency_order_facts().is_empty());
    assert_eq!(empty.dependency_edge_count(), 0);

    let mixed = fixture(vec![
        root(),
        layout(1, 0),
        free(2, 0, SpatialAnchorTargetV2::Viewport),
        free(3, 0, SpatialAnchorTargetV2::Parent),
    ]);
    let proof = expect_valid(prepare_dependency_graph!(
        &mixed,
        VIEWPORT,
        dependency_limits(3, 0)
    ));
    assert_eq!(
        proof.dependency_unit_facts(),
        vec![
            (1, Some((0, 0)), vec![1], vec![]),
            (2, None, vec![2], vec![]),
            (3, None, vec![3], vec![]),
        ]
    );
    assert_eq!(proof.dependency_order_facts(), vec![1, 2, 3]);
}

#[test]
fn the_ready_queue_uses_lowest_stable_ordinal_after_each_commit() {
    let fixture = fixture(vec![
        root(),
        free(1, 0, node_target(4)),
        free(2, 0, SpatialAnchorTargetV2::Viewport),
        layout(3, 2),
        free(4, 3, SpatialAnchorTargetV2::Parent),
        free(5, 0, SpatialAnchorTargetV2::Viewport),
    ]);
    let proof = expect_valid(prepare_dependency_graph!(
        &fixture,
        VIEWPORT,
        dependency_limits(5, 3)
    ));

    assert_eq!(
        proof.dependency_unit_facts(),
        vec![
            (1, None, vec![1], vec![4]),
            (2, None, vec![2], vec![]),
            (3, Some((0, 2)), vec![3], vec![2]),
            (4, None, vec![4], vec![3]),
            (5, None, vec![5], vec![]),
        ]
    );
    assert_eq!(proof.dependency_order_facts(), vec![2, 3, 4, 1, 5]);
}
