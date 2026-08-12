use super::local_transform_support::{
    VIEWPORT, fixed, free_node, identity, input, layout_node, root,
};
use super::shape_structure_support::{
    circle, expect_valid, fixture, fixture_with_paths, limits, path_shape, point, polygon, rect,
    scalar, shape, shape_k1_poison_limits, validate,
};
use super::validated_path_support::{line_to, move_to, path};
use crate::model::{SpatialPointV2, SpatialScalarV2};
use crate::shape::SpatialShapeGeometryV2;

#[test]
fn empty_shapes_and_points_retain_empty_structure_facts() {
    let fixture = fixture(Vec::new(), Vec::new());
    let proof = expect_valid(prepare_shape_structure!(&fixture, VIEWPORT, limits()));

    assert!(proof.polygon_range_facts().is_empty());
    assert!(proof.validated_path_facts().is_empty());
    assert_eq!(proof.subpath_total(), 0);
}

#[test]
fn zero_length_polygons_are_structurally_valid_and_retained() {
    let fixture = fixture(vec![polygon(0, 1, 0, 0), polygon(1, 2, 0, 0)], Vec::new());
    let proof = expect_valid(prepare_shape_structure!(&fixture, VIEWPORT, limits()));

    assert_eq!(
        proof.polygon_range_facts(),
        vec![(0, 0_u128, 0_u128), (1, 0, 0)]
    );
}

#[test]
fn mixed_shapes_retain_exact_polygon_ranges_and_all_prior_facts() {
    let transform = identity();
    let fixture = input(vec![
        root(),
        free_node(1, 0, 10, 10, transform),
        free_node(2, 0, 10, 10, transform),
        layout_node(3, 2, fixed(10), fixed(10), transform),
        free_node(4, 3, 10, 10, transform),
        layout_node(5, 4, fixed(10), fixed(10), transform),
        layout_node(6, 3, fixed(10), fixed(10), transform),
    ])
    .with_paths(
        vec![path(0, 0, 2), path(1, 2, 2)],
        vec![move_to(0, 0), line_to(1, 1), move_to(2, 2), line_to(3, 3)],
    )
    .with_shapes(
        vec![
            rect(0, 1),
            polygon(1, 2, 0, 2),
            circle(2, 1),
            path_shape(3, 2, 1),
            polygon(4, 1, 2, 0),
            path_shape(5, 2, 0),
        ],
        vec![point(4, 4), point(5, 5)],
    );
    let proof = expect_valid(prepare_shape_structure!(&fixture, VIEWPORT, limits()));

    assert_eq!(
        proof.polygon_range_facts(),
        vec![(1, 0_u128, 2_u128), (4, 2, 2)]
    );
    assert_eq!(proof.validated_path_facts(), vec![(0, 2, 1), (1, 2, 1)]);
    assert_eq!(proof.subpath_total(), 2);
    assert_eq!(
        proof.path_range_facts(),
        vec![(0, 0_u128, 2_u128), (1, 2, 4)]
    );
    assert_eq!(
        proof.prepared_island_facts(),
        vec![(0, vec![2, 3, 6]), (1, vec![4, 5])]
    );
}

#[test]
fn shape_k1_limits_and_later_content_stages_remain_deferred() {
    let outside = SpatialScalarV2::MAX_RAW + 1;
    let fixture = fixture_with_paths(
        vec![
            shape(
                0,
                1,
                SpatialShapeGeometryV2::Rect {
                    origin: SpatialPointV2::new(scalar(outside), scalar(0)),
                    width: scalar(-1),
                    height: scalar(-1),
                },
            ),
            shape(
                1,
                1,
                SpatialShapeGeometryV2::Circle {
                    center: SpatialPointV2::new(scalar(0), scalar(0)),
                    radius: scalar(-1),
                },
            ),
            polygon(2, 2, 0, 1),
            path_shape(3, 2, 0),
        ],
        vec![point(outside, SpatialScalarV2::MIN_RAW - 1)],
        vec![path(0, 0, 2)],
        vec![move_to(0, 0), line_to(1, 1)],
    );

    expect_valid(validate(&fixture, shape_k1_poison_limits()));
}
