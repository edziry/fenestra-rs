use super::local_transform_support::{
    VIEWPORT, fixed, free_node, identity, input, layout_node, root,
};
use super::validated_path_support::{move_to, path};
use super::validated_shape_support::{
    circle_values, deferred_k2_limits, expect_valid, fixture, fixture_with_paths, limits,
    path_shape, point, polygon, rect_values, validate,
};
use crate::model::SpatialScalarV2;
use crate::path::SpatialPathVerbV2;
use crate::shape::SpatialShapeKindV2;

#[test]
fn empty_shapes_retain_empty_validated_facts() {
    let fixture = fixture(Vec::new(), Vec::new());
    let proof = expect_valid(prepare_validated_shapes!(&fixture, VIEWPORT, limits(0)));

    assert!(proof.validated_shape_facts().is_empty());
    assert!(proof.polygon_range_facts().is_empty());
}

#[test]
fn mixed_k1_proofs_and_all_prior_facts_survive_the_stage() {
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
        vec![path(0, 0, 2)],
        vec![move_to(0, 0), SpatialPathVerbV2::LineTo { to: point(1, 1) }],
    )
    .with_shapes(
        vec![
            rect_values(0, 1, -2, 3, 4, 5),
            circle_values(1, 2, 6, 7, 8),
            polygon(2, 1, 0, 4),
            path_shape(3, 2, 0),
        ],
        vec![point(0, 0), point(4, 0), point(4, 4), point(0, 4)],
    );
    let proof = expect_valid(prepare_validated_shapes!(&fixture, VIEWPORT, limits(4)));
    let raw_points = fixture
        .input_with_viewport(VIEWPORT)
        .geometry()
        .polygon_points();

    assert_eq!(
        proof.validated_shape_facts(),
        vec![
            (0, SpatialShapeKindV2::Rect, 0),
            (1, SpatialShapeKindV2::Circle, 0),
            (2, SpatialShapeKindV2::Polygon, 4),
            (3, SpatialShapeKindV2::Path, 0),
        ]
    );
    assert_eq!(proof.polygon_range_facts(), vec![(2, 0_u128, 4_u128)]);
    assert!(std::ptr::eq(proof.validated_polygon_points(2), raw_points));
    assert_eq!(proof.validated_path_facts(), vec![(0, 2, 1)]);
    assert_eq!(proof.subpath_total(), 1);
    assert_eq!(proof.path_range_facts(), vec![(0, 0_u128, 2_u128)]);
    assert_eq!(
        proof.prepared_island_facts(),
        vec![(0, vec![2, 3, 6]), (1, vec![4, 5])]
    );
}

#[test]
fn canonical_edges_and_allowed_polygon_shapes_succeed() {
    let minimum = SpatialScalarV2::MIN_RAW;
    let maximum = SpatialScalarV2::MAX_RAW;
    let boundary = fixture(
        vec![
            rect_values(0, 1, minimum, maximum, maximum, 0),
            circle_values(1, 2, maximum, minimum, 0),
            polygon(2, 1, 0, 3),
        ],
        vec![
            point(minimum, maximum),
            point(maximum, maximum),
            point(minimum, minimum),
        ],
    );
    expect_valid(validate(&boundary, limits(3)));

    let zero = fixture(
        vec![rect_values(0, 1, 0, 0, 0, 0), circle_values(1, 2, 0, 0, 0)],
        Vec::new(),
    );
    expect_valid(validate(&zero, limits(0)));

    let allowed = [
        vec![point(0, 0), point(2, 0), point(0, 0), point(4, 0)],
        vec![point(0, 0), point(0, 4), point(4, 4), point(4, 0)],
        vec![point(0, 0), point(4, 4), point(0, 4), point(4, 0)],
    ];
    for points in allowed {
        let fixture = fixture(vec![polygon(0, 1, 0, points.len() as u32)], points);
        expect_valid(validate(&fixture, limits(4)));
    }
}

#[test]
fn flattening_bounds_and_later_content_tables_remain_deferred() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let fixture = fixture_with_paths(
        vec![
            rect_values(0, 1, maximum, 0, maximum, 0),
            circle_values(1, 2, maximum, 0, maximum),
            path_shape(2, 1, 0),
        ],
        Vec::new(),
        vec![path(0, 0, 2)],
        vec![
            move_to(0, 0),
            SpatialPathVerbV2::QuadraticTo {
                control: point(2, 4),
                to: point(4, 0),
            },
        ],
    );

    expect_valid(validate(&fixture, deferred_k2_limits(usize::MAX)));
}
