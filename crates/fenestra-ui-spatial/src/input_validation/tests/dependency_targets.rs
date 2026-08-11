use super::dependency_support::{
    VIEWPORT, dependency_limits, expect_dependency, expect_valid, fixture, free, node_target, root,
};
use crate::error::{SpatialDependencyErrorKindV2, SpatialErrorLocationV2};
use crate::model::{SpatialAnchorTargetV2, SpatialNodeKeyV2};
use crate::vocabulary::SpatialNodeFieldV2;

#[test]
fn missing_sentinel_and_self_targets_use_the_exact_trusted_node_field() {
    for (target, kind) in [
        (2, SpatialDependencyErrorKindV2::MissingTarget),
        (u32::MAX, SpatialDependencyErrorKindV2::MissingTarget),
        (0, SpatialDependencyErrorKindV2::SentinelNodeTarget),
        (1, SpatialDependencyErrorKindV2::SelfTarget),
    ] {
        let fixture = fixture(vec![root(), free(1, 0, node_target(target))]);
        expect_dependency(
            prepare_dependency_graph!(&fixture, VIEWPORT, unlimited()),
            kind,
            target_location(1),
        );
    }
}

#[test]
fn complete_scope_beats_an_earlier_self_target_for_missing_and_sentinel() {
    for (later_target, kind) in [
        (u32::MAX, SpatialDependencyErrorKindV2::MissingTarget),
        (0, SpatialDependencyErrorKindV2::SentinelNodeTarget),
    ] {
        let fixture = fixture(vec![
            root(),
            free(1, 0, node_target(1)),
            free(2, 0, node_target(later_target)),
        ]);
        expect_dependency(
            prepare_dependency_graph!(&fixture, VIEWPORT, unlimited()),
            kind,
            target_location(2),
        );
    }
}

#[test]
fn missing_and_sentinel_share_one_record_major_scope_pass() {
    let sentinel_first = fixture(vec![
        root(),
        free(1, 0, node_target(0)),
        free(2, 0, node_target(u32::MAX)),
    ]);
    expect_dependency(
        prepare_dependency_graph!(&sentinel_first, VIEWPORT, unlimited()),
        SpatialDependencyErrorKindV2::SentinelNodeTarget,
        target_location(1),
    );

    let missing_first = fixture(vec![
        root(),
        free(1, 0, node_target(u32::MAX)),
        free(2, 0, node_target(0)),
    ]);
    expect_dependency(
        prepare_dependency_graph!(&missing_first, VIEWPORT, unlimited()),
        SpatialDependencyErrorKindV2::MissingTarget,
        target_location(1),
    );
}

#[test]
fn the_self_pass_uses_the_first_free_node_ordinal() {
    let fixture = fixture(vec![
        root(),
        free(1, 0, node_target(1)),
        free(2, 0, node_target(2)),
    ]);
    expect_dependency(
        prepare_dependency_graph!(&fixture, VIEWPORT, unlimited()),
        SpatialDependencyErrorKindV2::SelfTarget,
        target_location(1),
    );
}

#[test]
fn viewport_parent_and_existing_earlier_or_future_nodes_are_in_scope() {
    let fixture = fixture(vec![
        root(),
        free(1, 0, node_target(3)),
        free(2, 0, SpatialAnchorTargetV2::Parent),
        free(3, 0, SpatialAnchorTargetV2::Viewport),
        free(4, 0, SpatialAnchorTargetV2::Node(SpatialNodeKeyV2::new(1))),
    ]);
    let proof = expect_valid(prepare_dependency_graph!(&fixture, VIEWPORT, unlimited()));

    assert_eq!(proof.dependency_order_facts(), vec![2, 3, 1, 4]);
}

fn unlimited() -> crate::limits::SpatialLimitsV2 {
    dependency_limits(usize::MAX, usize::MAX)
}

const fn target_location(index: u32) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::NodeField {
        index,
        field: SpatialNodeFieldV2::TargetKey,
    }
}
