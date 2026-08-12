use std::error::Error;

use fenestra_ui_layout::prototype::{LayoutAxisV1, LayoutDimensionV1, LayoutPaddingV1};

use super::super::{prepare_placement_input, prepare_topology_limits};
use super::fixture::RawInputFixture;
use super::{prepare_direct_counts, prepare_topology};
use crate::error::{SpatialErrorLocationV2, SpatialInputErrorKindV2};
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::model::{
    Affine2V2, SpatialAnchorComponentV2, SpatialAnchorTargetV2, SpatialAnchorV2,
    SpatialLocalTransformV2, SpatialNodeKeyV2, SpatialOffsetV2, SpatialPointV2, SpatialScalarV2,
    SpatialViewportV2,
};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};
use crate::topology::{
    SpatialContainerV2, SpatialFreePlacementV2, SpatialLayoutPlacementV2, SpatialNodeV2,
    SpatialPlacementV2,
};
use crate::vocabulary::{SpatialAxisV2, SpatialExtentV2, SpatialNodeFieldV2};

#[test]
fn complete_nonroot_root_pass_precedes_viewport_and_free_fields() {
    let nodes = vec![
        root(),
        node(1, free(-1, -2, invalid_low(), invalid_high())),
        node(2, SpatialPlacementV2::Root),
        node(3, SpatialPlacementV2::Root),
    ];

    expect_input(
        phase_four_result(nodes, SpatialViewportV2::new(-1, -1), generous_limits()),
        SpatialInputErrorKindV2::RootPlacementOnNonRoot,
        node_field(2, SpatialNodeFieldV2::Placement),
    );
}

#[test]
fn viewport_width_then_height_precede_free_fields() {
    let nodes = vec![root(), node(1, free(-1, -1, invalid_low(), invalid_high()))];

    expect_input(
        phase_four_result(
            nodes.clone(),
            SpatialViewportV2::new(-1, -1),
            generous_limits(),
        ),
        SpatialInputErrorKindV2::NegativeViewport(SpatialExtentV2::Width),
        SpatialErrorLocationV2::Viewport {
            extent: SpatialExtentV2::Width,
        },
    );
    expect_input(
        phase_four_result(nodes, SpatialViewportV2::new(0, -1), generous_limits()),
        SpatialInputErrorKindV2::NegativeViewport(SpatialExtentV2::Height),
        SpatialErrorLocationV2::Viewport {
            extent: SpatialExtentV2::Height,
        },
    );
}

#[test]
fn free_fields_follow_width_height_offset_x_offset_y_order() {
    let cases = [
        (
            free(-1, -1, invalid_low(), invalid_high()),
            SpatialInputErrorKindV2::NegativeFreeExtent(SpatialExtentV2::Width),
            SpatialNodeFieldV2::FreeWidth,
        ),
        (
            free(0, -1, invalid_low(), invalid_high()),
            SpatialInputErrorKindV2::NegativeFreeExtent(SpatialExtentV2::Height),
            SpatialNodeFieldV2::FreeHeight,
        ),
        (
            free(0, 0, invalid_low(), invalid_high()),
            SpatialInputErrorKindV2::FreeOffsetOutOfDomain(SpatialAxisV2::X),
            SpatialNodeFieldV2::FreeOffsetX,
        ),
        (
            free(0, 0, SpatialScalarV2::MIN_RAW, invalid_high()),
            SpatialInputErrorKindV2::FreeOffsetOutOfDomain(SpatialAxisV2::Y),
            SpatialNodeFieldV2::FreeOffsetY,
        ),
    ];

    for (placement, kind, field) in cases {
        expect_input(
            phase_four_result(
                vec![root(), node(1, placement)],
                SpatialViewportV2::new(0, 0),
                generous_limits(),
            ),
            kind,
            node_field(1, field),
        );
    }
}

#[test]
fn both_offset_axes_reject_both_sides_of_the_scalar_domain() {
    for raw in [invalid_low(), invalid_high()] {
        expect_input(
            one_free_result(free(0, 0, raw, 0)),
            SpatialInputErrorKindV2::FreeOffsetOutOfDomain(SpatialAxisV2::X),
            node_field(1, SpatialNodeFieldV2::FreeOffsetX),
        );
    }
    for raw in [invalid_low(), invalid_high()] {
        expect_input(
            one_free_result(free(0, 0, SpatialScalarV2::MAX_RAW, raw)),
            SpatialInputErrorKindV2::FreeOffsetOutOfDomain(SpatialAxisV2::Y),
            node_field(1, SpatialNodeFieldV2::FreeOffsetY),
        );
    }
}

#[test]
fn free_validation_is_record_major_and_skips_layout_nodes() {
    let nodes = vec![
        root(),
        node(1, invalid_layout()),
        node(2, free(0, 0, 0, invalid_high())),
        node(3, free(-1, 0, 0, 0)),
    ];

    expect_input(
        phase_four_result(nodes, SpatialViewportV2::new(0, 0), generous_limits()),
        SpatialInputErrorKindV2::FreeOffsetOutOfDomain(SpatialAxisV2::Y),
        node_field(2, SpatialNodeFieldV2::FreeOffsetY),
    );
}

#[test]
fn canonical_edges_succeed_while_later_validation_remains_deferred() {
    let minimum = SpatialScalarV2::MIN_RAW;
    let maximum = SpatialScalarV2::MAX_RAW;
    let nodes = vec![
        root(),
        node(1, invalid_layout()),
        node(2, free(0, 0, minimum, minimum)),
        node(3, free(i32::MAX, 0, minimum, maximum)),
        node(4, free(0, i32::MAX, maximum, minimum)),
        node(5, free(i32::MAX, i32::MAX, maximum, maximum)),
    ];

    expect_valid(phase_four_result(
        nodes.clone(),
        SpatialViewportV2::new(0, 0),
        generous_limits(),
    ));
    expect_valid(phase_four_result(
        nodes,
        SpatialViewportV2::new(i32::MAX, i32::MAX),
        generous_limits(),
    ));
}

#[test]
fn topology_limits_precede_remaining_placement_input() {
    let nodes = vec![root(), node(1, SpatialPlacementV2::Root)];

    expect_limit(
        phase_four_result(nodes, SpatialViewportV2::new(-1, -1), limits(2, 0)),
        SpatialLimitKindV2::ChildrenPerNode,
        0,
        1,
        0,
    );
}

fn one_free_result(placement: SpatialPlacementV2) -> Result<(), SpatialResolveErrorV2> {
    phase_four_result(
        vec![root(), node(1, placement)],
        SpatialViewportV2::new(0, 0),
        generous_limits(),
    )
}

fn phase_four_result(
    nodes: Vec<SpatialNodeV2>,
    viewport: SpatialViewportV2,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    let fixture = RawInputFixture::with_nodes(nodes);
    let direct_counts = prepare_direct_counts(fixture.input_with_viewport(viewport), limits)?;
    let topology = prepare_topology(direct_counts)?;
    let topology_limits = prepare_topology_limits(topology)?;
    prepare_placement_input(topology_limits).map(|_| ())
}

fn generous_limits() -> SpatialLimitsV2 {
    limits(usize::MAX, usize::MAX)
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
        panic!("expected remaining-placement success, got {error:?}");
    }
}

fn expect_input(
    result: Result<(), SpatialResolveErrorV2>,
    kind: SpatialInputErrorKindV2,
    location: SpatialErrorLocationV2,
) {
    let error = match result {
        Ok(()) => panic!("expected remaining-placement input failure"),
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

fn expect_limit(
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
}

fn node_field(index: u32, field: SpatialNodeFieldV2) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::NodeField { index, field }
}

fn root() -> SpatialNodeV2 {
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(0),
        None,
        SpatialPlacementV2::Root,
        invalid_container(),
    )
}

fn node(key: u32, placement: SpatialPlacementV2) -> SpatialNodeV2 {
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(key),
        Some(SpatialNodeKeyV2::new(0)),
        placement,
        invalid_container(),
    )
}

fn free(width: i32, height: i32, x: i64, y: i64) -> SpatialPlacementV2 {
    let anchor = SpatialAnchorV2::new(
        SpatialAnchorComponentV2::Start,
        SpatialAnchorComponentV2::End,
    );
    SpatialPlacementV2::Free(SpatialFreePlacementV2::new(
        width,
        height,
        anchor,
        SpatialAnchorTargetV2::Node(SpatialNodeKeyV2::new(u32::MAX)),
        anchor,
        SpatialOffsetV2::new(SpatialScalarV2::new(x), SpatialScalarV2::new(y)),
        invalid_transform(),
    ))
}

fn invalid_layout() -> SpatialPlacementV2 {
    SpatialPlacementV2::Layout(SpatialLayoutPlacementV2::new(
        LayoutDimensionV1::new(-1, -2, -3),
        LayoutDimensionV1::new(-4, -5, -6),
        invalid_transform(),
    ))
}

fn invalid_container() -> SpatialContainerV2 {
    SpatialContainerV2::new(
        LayoutAxisV1::Column,
        LayoutPaddingV1::new(-1, -2, -3, -4),
        -5,
    )
}

fn invalid_transform() -> SpatialLocalTransformV2 {
    let scalar = SpatialScalarV2::new(i64::MAX);
    SpatialLocalTransformV2::new(
        Affine2V2::new(scalar, scalar, scalar, scalar, scalar, scalar),
        SpatialPointV2::new(scalar, scalar),
    )
}

const fn invalid_low() -> i64 {
    SpatialScalarV2::MIN_RAW - 1
}

const fn invalid_high() -> i64 {
    SpatialScalarV2::MAX_RAW + 1
}
