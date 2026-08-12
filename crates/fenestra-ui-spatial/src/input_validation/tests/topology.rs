use std::error::Error;

use fenestra_ui_layout::prototype::{LayoutAxisV1, LayoutDimensionV1, LayoutPaddingV1};

use super::fixture::RawInputFixture;
use super::{DIRECT_COUNT, prepare_direct_counts, prepare_topology};
use crate::error::{SpatialErrorLocationV2, SpatialInputErrorKindV2};
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::model::{
    Affine2V2, SpatialAnchorComponentV2, SpatialAnchorTargetV2, SpatialAnchorV2,
    SpatialLocalTransformV2, SpatialNodeKeyV2, SpatialOffsetV2, SpatialPointV2, SpatialScalarV2,
};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};
use crate::topology::{
    SpatialContainerV2, SpatialFreePlacementV2, SpatialLayoutPlacementV2, SpatialNodeV2,
    SpatialPlacementV2,
};
use crate::vocabulary::SpatialNodeFieldV2;

#[test]
fn direct_counts_precede_empty_topology() {
    let fixture = RawInputFixture::with_nodes(Vec::new());
    let mut values = phase_two_limit_values();
    values[1] = 0;

    let error = prepare_fixture_topology(&fixture, SpatialLimitsV2::new(values))
        .expect_err("shape count must fail before empty topology");
    assert_eq!(
        error.kind(),
        SpatialResolveErrorKindV2::LimitExceeded(SpatialLimitKindV2::Shapes)
    );
    assert_eq!(error.location(), SpatialErrorLocationV2::Input);
    assert_eq!(error.observed(), Some(1));
    assert_eq!(error.maximum(), Some(0));
}

#[test]
fn empty_topology_fails_at_input() {
    expect_input_failure(
        prepare_fixture_topology(&RawInputFixture::with_nodes(Vec::new()), phase_two_limits()),
        SpatialInputErrorKindV2::EmptyInput,
        SpatialErrorLocationV2::Input,
    );
}

#[test]
fn root_fields_follow_key_parent_placement_order() {
    let cases = [
        (
            root_node(7, Some(8), layout_placement()),
            SpatialInputErrorKindV2::InvalidRootKey,
            SpatialNodeFieldV2::Key,
        ),
        (
            root_node(0, Some(8), free_placement()),
            SpatialInputErrorKindV2::RootHasParent,
            SpatialNodeFieldV2::Parent,
        ),
    ];

    for (root, kind, field) in cases {
        expect_node_failure(vec![root], kind, 0, field);
    }

    for placement in [layout_placement(), free_placement()] {
        expect_node_failure(
            vec![root_node(0, None, placement)],
            SpatialInputErrorKindV2::InvalidRootPlacement,
            0,
            SpatialNodeFieldV2::Placement,
        );
    }
}

#[test]
fn later_node_fields_use_trusted_ordinals_and_parent_classification() {
    let cases = [
        (
            vec![root(), node(u32::MAX, None)],
            SpatialInputErrorKindV2::NonDenseNodeKey,
            1,
        ),
        (
            vec![root(), node(1, None), node(u32::MAX, None)],
            SpatialInputErrorKindV2::MissingSpatialParent,
            1,
        ),
        (
            vec![root(), node(1, Some(u32::MAX))],
            SpatialInputErrorKindV2::MissingSpatialParent,
            1,
        ),
        (
            vec![root(), node(1, Some(2)), node(99, Some(0))],
            SpatialInputErrorKindV2::MissingSpatialParent,
            1,
        ),
    ];

    for (nodes, kind, index) in cases {
        expect_node_failure(nodes, kind, index, field_for(kind));
    }
}

#[test]
fn matching_self_and_future_parents_are_forward() {
    let cases = [
        vec![root(), node(1, Some(1)), node(u32::MAX, None)],
        vec![root(), node(1, Some(2)), node(2, Some(0)), node(99, None)],
    ];

    for nodes in cases {
        expect_node_failure(
            nodes,
            SpatialInputErrorKindV2::ForwardSpatialParent,
            1,
            SpatialNodeFieldV2::Parent,
        );
    }
}

#[test]
fn a_closed_subtree_cannot_be_reopened() {
    let nodes = vec![
        root(),
        node(1, Some(0)),
        node(2, Some(1)),
        node(3, Some(0)),
        node(4, Some(1)),
        node(u32::MAX, None),
    ];

    expect_node_failure(
        nodes,
        SpatialInputErrorKindV2::InvalidPreorder,
        4,
        SpatialNodeFieldV2::Parent,
    );
}

#[test]
fn valid_preorder_defers_derived_limits_and_later_input_checks() {
    let nodes = vec![
        root(),
        node(1, Some(0)),
        node(2, Some(1)),
        node(3, Some(2)),
        node(4, Some(0)),
        node(5, Some(4)),
        node(6, Some(0)),
    ];
    let fixture = RawInputFixture::with_nodes(nodes);
    let direct_counts = match prepare_direct_counts(fixture.input(), phase_two_limits()) {
        Ok(proof) => proof,
        Err(error) => panic!("expected direct-count success, got {error:?}"),
    };
    let topology = match prepare_topology(direct_counts) {
        Ok(proof) => proof,
        Err(error) => panic!("expected phase-two topology success, got {error:?}"),
    };

    assert_eq!(topology.depths(), &[1, 2, 3, 4, 2, 3, 2]);
    assert_eq!(topology.child_counts(), &[3, 1, 1, 0, 1, 0, 0]);
}

fn prepare_fixture_topology(
    fixture: &RawInputFixture,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    let direct_counts = prepare_direct_counts(fixture.input(), limits)?;
    prepare_topology(direct_counts).map(|_| ())
}

fn phase_two_limits() -> SpatialLimitsV2 {
    SpatialLimitsV2::new(phase_two_limit_values())
}

fn phase_two_limit_values() -> [usize; SpatialLimitKindV2::ALL.len()] {
    let mut values = [0; SpatialLimitKindV2::ALL.len()];
    values[..DIRECT_COUNT].fill(usize::MAX);
    values
}

fn expect_node_failure(
    nodes: Vec<SpatialNodeV2>,
    kind: SpatialInputErrorKindV2,
    index: u32,
    field: SpatialNodeFieldV2,
) {
    expect_input_failure(
        prepare_fixture_topology(&RawInputFixture::with_nodes(nodes), phase_two_limits()),
        kind,
        SpatialErrorLocationV2::NodeField { index, field },
    );
}

fn expect_input_failure<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialInputErrorKindV2,
    location: SpatialErrorLocationV2,
) {
    let error = match result {
        Ok(_) => panic!("expected topology input failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), SpatialResolveErrorKindV2::Input(kind));
    assert_eq!(error.location(), location);
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
    assert_eq!(error.to_string(), "spatial-resolve-error(input)");
    assert_eq!(
        format!("{error:?}"),
        "SpatialResolveErrorV2(spatial-resolve-error(input))"
    );
    assert!(Error::source(&error).is_none());
}

fn field_for(kind: SpatialInputErrorKindV2) -> SpatialNodeFieldV2 {
    match kind {
        SpatialInputErrorKindV2::NonDenseNodeKey => SpatialNodeFieldV2::Key,
        SpatialInputErrorKindV2::MissingSpatialParent => SpatialNodeFieldV2::Parent,
        _ => panic!("unexpected test input kind"),
    }
}

fn root() -> SpatialNodeV2 {
    root_node(0, None, SpatialPlacementV2::Root)
}

fn node(key: u32, parent: Option<u32>) -> SpatialNodeV2 {
    root_node(key, parent, SpatialPlacementV2::Root)
}

fn root_node(key: u32, parent: Option<u32>, placement: SpatialPlacementV2) -> SpatialNodeV2 {
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(key),
        parent.map(SpatialNodeKeyV2::new),
        placement,
        SpatialContainerV2::new(
            LayoutAxisV1::Column,
            LayoutPaddingV1::new(-1, -2, -3, -4),
            -5,
        ),
    )
}

fn layout_placement() -> SpatialPlacementV2 {
    SpatialPlacementV2::Layout(SpatialLayoutPlacementV2::new(
        LayoutDimensionV1::new(-1, -2, -3),
        LayoutDimensionV1::new(-4, -5, -6),
        malformed_transform(),
    ))
}

fn free_placement() -> SpatialPlacementV2 {
    let anchor = SpatialAnchorV2::new(
        SpatialAnchorComponentV2::Start,
        SpatialAnchorComponentV2::End,
    );
    SpatialPlacementV2::Free(SpatialFreePlacementV2::new(
        -1,
        -2,
        anchor,
        SpatialAnchorTargetV2::Node(SpatialNodeKeyV2::new(u32::MAX)),
        anchor,
        SpatialOffsetV2::new(
            SpatialScalarV2::new(i64::MAX),
            SpatialScalarV2::new(i64::MIN),
        ),
        malformed_transform(),
    ))
}

fn malformed_transform() -> SpatialLocalTransformV2 {
    let scalar = SpatialScalarV2::new(i64::MAX);
    SpatialLocalTransformV2::new(
        Affine2V2::new(scalar, scalar, scalar, scalar, scalar, scalar),
        SpatialPointV2::new(scalar, scalar),
    )
}
