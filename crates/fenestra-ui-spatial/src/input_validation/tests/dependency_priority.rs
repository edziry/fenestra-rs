use super::dependency_support::{
    VIEWPORT, dependency_limits, expect_dependency, fixture, free, node_target, root,
};
use super::local_bounds_support::expect_bounds_error;
use super::validated_shape_support::rect_values;
use crate::error::{SpatialDependencyErrorKindV2, SpatialErrorLocationV2};
use crate::geometry_field::SpatialShapeFieldV2;
use crate::model::SpatialScalarV2;
use crate::vocabulary::{SpatialAxisV2, SpatialNodeFieldV2};

#[test]
fn complete_local_bounds_precede_every_dependency_target_and_limit() {
    let fixture = fixture(vec![root(), free(1, 0, node_target(u32::MAX))]).with_shapes(
        vec![rect_values(
            0,
            1,
            SpatialScalarV2::MAX_RAW,
            SpatialScalarV2::MAX_RAW,
            1,
            1,
        )],
        Vec::new(),
    );
    expect_bounds_error(
        prepare_dependency_graph!(&fixture, VIEWPORT, dependency_limits(0, 0)),
        SpatialAxisV2::X,
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::RectWidth,
        },
    );
}

#[test]
fn all_target_failures_precede_vertex_edge_and_cycle_checks() {
    for (target, kind, index) in [
        (u32::MAX, SpatialDependencyErrorKindV2::MissingTarget, 2),
        (0, SpatialDependencyErrorKindV2::SentinelNodeTarget, 2),
        (1, SpatialDependencyErrorKindV2::SelfTarget, 1),
    ] {
        let fixture = fixture(vec![
            root(),
            free(1, 0, node_target(if index == 1 { target } else { 2 })),
            free(2, 0, node_target(target)),
        ]);
        expect_dependency(
            prepare_dependency_graph!(&fixture, VIEWPORT, dependency_limits(0, 0)),
            kind,
            SpatialErrorLocationV2::NodeField {
                index,
                field: SpatialNodeFieldV2::TargetKey,
            },
        );
    }
}

#[test]
fn complete_target_scope_precedes_a_real_independent_cycle() {
    for (target, kind) in [
        (u32::MAX, SpatialDependencyErrorKindV2::MissingTarget),
        (0, SpatialDependencyErrorKindV2::SentinelNodeTarget),
    ] {
        let fixture = fixture(vec![
            root(),
            free(1, 0, node_target(2)),
            free(2, 0, node_target(1)),
            free(3, 0, node_target(target)),
        ]);
        expect_dependency(
            prepare_dependency_graph!(
                &fixture,
                VIEWPORT,
                dependency_limits(usize::MAX, usize::MAX)
            ),
            kind,
            SpatialErrorLocationV2::NodeField {
                index: 3,
                field: SpatialNodeFieldV2::TargetKey,
            },
        );
    }

    let self_before_cycle = fixture(vec![
        root(),
        free(1, 0, node_target(1)),
        free(2, 0, node_target(3)),
        free(3, 0, node_target(2)),
    ]);
    expect_dependency(
        prepare_dependency_graph!(
            &self_before_cycle,
            VIEWPORT,
            dependency_limits(usize::MAX, usize::MAX)
        ),
        SpatialDependencyErrorKindV2::SelfTarget,
        SpatialErrorLocationV2::NodeField {
            index: 1,
            field: SpatialNodeFieldV2::TargetKey,
        },
    );
}
