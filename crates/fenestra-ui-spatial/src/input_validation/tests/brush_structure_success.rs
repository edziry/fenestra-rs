use super::brush_structure_support::{
    deferred_p2_limits, expect_valid, fixture, gradient, gradient_values, limits, point, solid,
    stop, validate,
};
use super::local_transform_support::{VIEWPORT, free_node, identity, input, root};
use super::validated_path_support::{line_to, move_to, path};
use super::validated_shape_support::{path_shape, polygon, rect};
use crate::shape::SpatialShapeKindV2;

#[test]
fn empty_brushes_and_stops_retain_empty_structure_facts() {
    let fixture = fixture(Vec::new(), Vec::new());
    let proof = expect_valid(prepare_brush_structure!(&fixture, VIEWPORT, limits()));

    assert!(proof.gradient_range_facts().is_empty());
    assert!(proof.validated_shape_facts().is_empty());
    assert!(proof.polygon_range_facts().is_empty());
}

#[test]
fn zero_length_gradients_are_structurally_valid_and_retained() {
    let fixture = fixture(
        vec![gradient(0, 0, 0), solid(1), gradient(2, 0, 0)],
        Vec::new(),
    );
    let proof = expect_valid(prepare_brush_structure!(&fixture, VIEWPORT, limits()));

    assert_eq!(
        proof.gradient_range_facts(),
        vec![(0, 0_u128, 0_u128), (2, 0, 0)]
    );
}

#[test]
fn mixed_brushes_retain_exact_ranges_slices_and_all_prior_facts() {
    let transform = identity();
    let fixture = input(vec![
        root(),
        free_node(1, 0, 10, 10, transform),
        free_node(2, 0, 10, 10, transform),
    ])
    .with_paths(vec![path(0, 0, 2)], vec![move_to(0, 0), line_to(1, 1)])
    .with_shapes(
        vec![rect(0, 1), polygon(1, 2, 0, 3), path_shape(2, 1, 0)],
        vec![point(0, 0), point(2, 0), point(0, 2)],
    )
    .with_brushes(
        vec![
            solid(0),
            gradient(1, 0, 2),
            solid(2),
            gradient(3, 2, 3),
            gradient(4, 5, 0),
        ],
        vec![
            stop(0),
            stop(u16::MAX),
            stop(0),
            stop(32_768),
            stop(u16::MAX),
        ],
    );
    let proof = expect_valid(prepare_brush_structure!(&fixture, VIEWPORT, limits()));
    let raw_stops = fixture
        .input_with_viewport(VIEWPORT)
        .resources()
        .gradient_stops();

    assert_eq!(
        proof.gradient_range_facts(),
        vec![(1, 0_u128, 2_u128), (3, 2, 5), (4, 5, 5)]
    );
    assert!(std::ptr::eq(proof.gradient_stops(0, 1), &raw_stops[..2]));
    assert!(std::ptr::eq(proof.gradient_stops(1, 3), &raw_stops[2..5]));
    assert_eq!(
        proof.validated_shape_facts(),
        vec![
            (0, SpatialShapeKindV2::Rect, 0),
            (1, SpatialShapeKindV2::Polygon, 3),
            (2, SpatialShapeKindV2::Path, 0),
        ]
    );
    assert_eq!(proof.polygon_range_facts(), vec![(1, 0_u128, 3_u128)]);
    assert_eq!(proof.validated_path_facts(), vec![(0, 2, 1)]);
    assert_eq!(proof.subpath_total(), 1);
    assert_eq!(proof.path_range_facts(), vec![(0, 0_u128, 2_u128)]);
    assert!(proof.prepared_island_facts().is_empty());
}

#[test]
fn p2_semantics_limits_and_later_content_tables_remain_deferred() {
    let outside = crate::model::SpatialScalarV2::MAX_RAW + 1;
    let poisoned = point(outside, outside);
    let fixture = fixture(
        vec![gradient_values(0, 0, 3, poisoned, poisoned)],
        vec![stop(7), stop(3), stop(4)],
    );

    expect_valid(validate(&fixture, deferred_p2_limits()));
}

#[test]
fn structurally_valid_gradients_do_not_run_p2_stop_minimums() {
    let fixture = fixture(
        vec![gradient(0, 0, 1)],
        vec![crate::brush::SpatialGradientStopV2::new(
            19,
            crate::brush::SpatialRgba8V2::new(1, 2, 3, 4),
        )],
    );

    expect_valid(validate(&fixture, limits()));
}
