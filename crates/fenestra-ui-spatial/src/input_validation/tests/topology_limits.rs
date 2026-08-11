use std::error::Error;

use fenestra_ui_layout::prototype::{LayoutAxisV1, LayoutPaddingV1};

use super::super::{prepare_topology_limits, validate_topology_fact};
use super::fixture::RawInputFixture;
use super::{prepare_direct_counts, prepare_topology};
use crate::error::{SpatialErrorLocationV2, SpatialInputErrorKindV2};
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::model::SpatialNodeKeyV2;
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};
use crate::topology::{SpatialContainerV2, SpatialNodeV2, SpatialPlacementV2};
use crate::vocabulary::SpatialNodeFieldV2;

#[test]
fn root_depth_one_and_zero_children_are_inclusive() {
    expect_valid(phase_three_result(vec![root()], limits(1, 0)));

    expect_topology_limit(
        phase_three_result(vec![root()], limits(0, usize::MAX)),
        SpatialLimitKindV2::Depth,
        0,
        1,
        0,
    );
}

#[test]
fn depth_scan_reports_the_first_offending_trusted_node() {
    let nodes = vec![root(), node(1, 0), node(2, 1), node(3, 0), node(4, 3)];

    expect_topology_limit(
        phase_three_result(nodes, limits(2, usize::MAX)),
        SpatialLimitKindV2::Depth,
        2,
        3,
        2,
    );
}

#[test]
fn complete_child_counts_scan_owners_in_node_order() {
    let nodes = vec![
        root(),
        node(1, 0),
        node(2, 1),
        node(3, 1),
        node(4, 1),
        node(5, 0),
        node(6, 0),
    ];

    expect_topology_limit(
        phase_three_result(nodes, limits(3, 1)),
        SpatialLimitKindV2::ChildrenPerNode,
        0,
        3,
        1,
    );

    let non_root_owner = vec![root(), node(1, 0), node(2, 1), node(3, 1)];
    expect_topology_limit(
        phase_three_result(non_root_owner, limits(3, 1)),
        SpatialLimitKindV2::ChildrenPerNode,
        1,
        2,
        1,
    );
}

#[test]
fn depth_global_pass_precedes_any_children_failure() {
    let nodes = vec![root(), node(1, 0), node(2, 0), node(3, 2), node(4, 3)];

    expect_topology_limit(
        phase_three_result(nodes, limits(2, 1)),
        SpatialLimitKindV2::Depth,
        3,
        3,
        2,
    );
}

#[test]
fn exact_custom_limits_succeed_and_phase_four_remains_deferred() {
    let nodes = vec![
        root(),
        node(1, 0),
        node(2, 1),
        node(3, 2),
        node(4, 0),
        node(5, 4),
        node(6, 0),
    ];

    expect_valid(phase_three_result(nodes.clone(), limits(4, 3)));
    expect_topology_limit(
        phase_three_result(nodes.clone(), limits(3, usize::MAX)),
        SpatialLimitKindV2::Depth,
        3,
        4,
        3,
    );
    expect_topology_limit(
        phase_three_result(nodes, limits(usize::MAX, 2)),
        SpatialLimitKindV2::ChildrenPerNode,
        0,
        3,
        2,
    );
}

#[cfg(target_pointer_width = "64")]
#[test]
fn widened_depth_fact_has_no_u32_capacity_ceiling() {
    let observed =
        usize::try_from(u32::MAX as u128 + 1).expect("depth above u32 fits a 64-bit usize");

    expect_valid(validate_topology_fact(
        SpatialLimitKindV2::Depth,
        u32::MAX,
        observed,
        limits(observed, usize::MAX),
    ));
    expect_topology_limit(
        validate_topology_fact(
            SpatialLimitKindV2::Depth,
            u32::MAX,
            observed,
            limits(u32::MAX as usize, usize::MAX),
        ),
        SpatialLimitKindV2::Depth,
        u32::MAX,
        u32::MAX as u128 + 1,
        u32::MAX as u128,
    );
}

#[test]
fn topology_input_failure_precedes_zero_derived_limits() {
    let nodes = vec![root(), node(1, 0), node(2, 0), node(3, 1)];
    let error = match phase_three_result(nodes, limits(0, 0)) {
        Ok(()) => panic!("expected topology input failure"),
        Err(error) => error,
    };

    assert_eq!(
        error.kind(),
        SpatialResolveErrorKindV2::Input(SpatialInputErrorKindV2::InvalidPreorder)
    );
    assert_eq!(
        error.location(),
        SpatialErrorLocationV2::NodeField {
            index: 3,
            field: SpatialNodeFieldV2::Parent,
        }
    );
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
    assert_eq!(error.to_string(), "spatial-resolve-error(input)");
    assert_eq!(
        format!("{error:?}"),
        "SpatialResolveErrorV2(spatial-resolve-error(input))"
    );
    assert!(Error::source(&error).is_none());
}

fn phase_three_result(
    nodes: Vec<SpatialNodeV2>,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    let fixture = RawInputFixture::with_nodes(nodes);
    let direct_counts = prepare_direct_counts(fixture.input(), limits)?;
    let topology = prepare_topology(direct_counts)?;
    prepare_topology_limits(topology).map(|_| ())
}

fn limits(depth: usize, children_per_node: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        match kind {
            SpatialLimitKindV2::Depth => *value = depth,
            SpatialLimitKindV2::ChildrenPerNode => *value = children_per_node,
            _ => {}
        }
    }
    SpatialLimitsV2::new(values)
}

fn expect_valid(result: Result<(), SpatialResolveErrorV2>) {
    if let Err(error) = result {
        panic!("expected topology-limit success, got {error:?}");
    }
}

fn expect_topology_limit(
    result: Result<(), SpatialResolveErrorV2>,
    limit: SpatialLimitKindV2,
    index: u32,
    observed: u128,
    maximum: u128,
) {
    let error = match result {
        Ok(()) => panic!("expected topology-limit failure"),
        Err(error) => error,
    };

    assert_eq!(
        error.kind(),
        SpatialResolveErrorKindV2::LimitExceeded(limit)
    );
    assert_eq!(error.location(), SpatialErrorLocationV2::Node { index });
    assert_eq!(error.observed(), Some(observed));
    assert_eq!(error.maximum(), Some(maximum));
    assert_eq!(error.to_string(), "spatial-resolve-error(limit-exceeded)");
    assert_eq!(
        format!("{error:?}"),
        "SpatialResolveErrorV2(spatial-resolve-error(limit-exceeded))"
    );
    assert!(Error::source(&error).is_none());
}

fn root() -> SpatialNodeV2 {
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(0),
        None,
        SpatialPlacementV2::Root,
        invalid_container(),
    )
}

fn node(key: u32, parent: u32) -> SpatialNodeV2 {
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(key),
        Some(SpatialNodeKeyV2::new(parent)),
        SpatialPlacementV2::Root,
        invalid_container(),
    )
}

fn invalid_container() -> SpatialContainerV2 {
    SpatialContainerV2::new(
        LayoutAxisV1::Column,
        LayoutPaddingV1::new(-1, -2, -3, -4),
        -5,
    )
}
