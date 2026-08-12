use super::dependency_support::{
    VIEWPORT, dependency_limits, expect_limit, expect_valid, fixture, free, node_target, root,
};
use crate::limits::{REGISTERED_SPATIAL_LIMITS_V2, SpatialLimitKindV2};
use crate::model::SpatialAnchorTargetV2;

const WIDE_EDGE_COUNT: u128 = u32::MAX as u128 + 1;

#[test]
fn vertex_and_edge_limits_report_complete_counts_and_accept_equality() {
    let fixture = fixture(vec![
        root(),
        free(1, 0, SpatialAnchorTargetV2::Viewport),
        free(2, 1, SpatialAnchorTargetV2::Viewport),
        free(3, 2, node_target(1)),
        free(4, 1, node_target(2)),
    ]);
    expect_valid(prepare_dependency_graph!(
        &fixture,
        VIEWPORT,
        dependency_limits(4, 5)
    ));
    expect_limit(
        prepare_dependency_graph!(&fixture, VIEWPORT, dependency_limits(3, 5)),
        SpatialLimitKindV2::DependencyVertices,
        4,
        3,
    );
    expect_limit(
        prepare_dependency_graph!(&fixture, VIEWPORT, dependency_limits(4, 1)),
        SpatialLimitKindV2::DependencyEdges,
        5,
        1,
    );
}

#[test]
fn vertex_limit_precedes_edges_and_edge_limit_precedes_cycle_detection() {
    let cycle = fixture(vec![
        root(),
        free(1, 0, node_target(2)),
        free(2, 0, node_target(1)),
    ]);
    expect_limit(
        prepare_dependency_graph!(&cycle, VIEWPORT, dependency_limits(0, 0)),
        SpatialLimitKindV2::DependencyVertices,
        2,
        0,
    );
    expect_limit(
        prepare_dependency_graph!(&cycle, VIEWPORT, dependency_limits(2, 1)),
        SpatialLimitKindV2::DependencyEdges,
        2,
        1,
    );
}

#[test]
fn caller_limits_are_not_capped_by_registered_vertex_or_edge_profiles() {
    assert_eq!(
        REGISTERED_SPATIAL_LIMITS_V2.limit(SpatialLimitKindV2::DependencyVertices),
        192
    );
    assert_eq!(
        REGISTERED_SPATIAL_LIMITS_V2.limit(SpatialLimitKindV2::DependencyEdges),
        256
    );

    let mut vertex_nodes = vec![root()];
    vertex_nodes.extend((1..=193).map(|key| free(key, 0, SpatialAnchorTargetV2::Viewport)));
    let vertices = fixture(vertex_nodes);
    expect_limit(
        prepare_dependency_graph!(&vertices, VIEWPORT, dependency_limits(192, usize::MAX)),
        SpatialLimitKindV2::DependencyVertices,
        193,
        192,
    );
    expect_valid(prepare_dependency_graph!(
        &vertices,
        VIEWPORT,
        dependency_limits(193, usize::MAX)
    ));

    let mut edge_nodes = vec![root(), free(1, 0, SpatialAnchorTargetV2::Viewport)];
    edge_nodes.extend((2..=258).map(|key| free(key, 0, node_target(1))));
    let edges = fixture(edge_nodes);
    expect_limit(
        prepare_dependency_graph!(&edges, VIEWPORT, dependency_limits(258, 256)),
        SpatialLimitKindV2::DependencyEdges,
        257,
        256,
    );
    expect_valid(prepare_dependency_graph!(
        &edges,
        VIEWPORT,
        dependency_limits(258, 257)
    ));
}

#[test]
fn production_limit_helper_preserves_reachable_edge_counts_above_u32() {
    expect_limit(
        super::check_dependency_fact(
            SpatialLimitKindV2::DependencyEdges,
            WIDE_EDGE_COUNT,
            dependency_limits(usize::MAX, u32::MAX as usize),
        ),
        SpatialLimitKindV2::DependencyEdges,
        WIDE_EDGE_COUNT,
        u32::MAX as u128,
    );

    #[cfg(target_pointer_width = "64")]
    expect_valid(super::check_dependency_fact(
        SpatialLimitKindV2::DependencyEdges,
        WIDE_EDGE_COUNT,
        dependency_limits(usize::MAX, WIDE_EDGE_COUNT as usize),
    ));
}

#[test]
fn root_only_accepts_zero_dependency_capacities() {
    let fixture = fixture(vec![root()]);
    expect_valid(prepare_dependency_graph!(
        &fixture,
        VIEWPORT,
        dependency_limits(0, 0)
    ));
}
