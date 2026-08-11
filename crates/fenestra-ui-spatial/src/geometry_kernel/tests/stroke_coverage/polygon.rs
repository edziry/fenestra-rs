use super::*;

#[test]
fn polygon_stroke_uses_each_exact_edge_and_the_last_first_closure() {
    let square = [point(0, 0), point(20, 0), point(20, 20), point(0, 20)];

    for query in [
        point(10, -5),
        point(25, 10),
        point(10, 25),
        point(-5, 10),
        point(-3, -4),
    ] {
        assert!(polygon_stroke_contains(&square, 10, query));
    }
    for query in [point(10, 10), point(-4, -4), point(24, 24)] {
        assert!(!polygon_stroke_contains(&square, 10, query));
    }
}

#[test]
fn collinear_polygon_strokes_only_its_boundary_line_and_round_ends() {
    let line = [point(-10, 0), point(0, 0), point(10, 0)];

    assert!(polygon_stroke_contains(&line, 10, point(0, 5)));
    assert!(polygon_stroke_contains(&line, 10, point(13, 4)));
    assert!(!polygon_stroke_contains(&line, 10, point(14, 4)));
}

#[test]
fn nonrectangular_polygon_uses_its_diagonal_instead_of_its_aabb_outline() {
    let triangle = [point(0, 0), point(20, 0), point(0, 20)];

    assert!(polygon_stroke_contains(&triangle, 2, point(10, 10)));
    assert!(!polygon_stroke_contains(&triangle, 2, point(20, 10)));
}

#[test]
fn concave_polygon_strokes_its_notch_instead_of_its_convex_hull() {
    let polygon = [
        point(0, 0),
        point(20, 0),
        point(20, 5),
        point(5, 5),
        point(5, 20),
        point(0, 20),
    ];

    assert!(polygon_stroke_contains(&polygon, 2, point(10, 5)));
    assert!(!polygon_stroke_contains(&polygon, 2, point(12, 13)));
}

#[test]
fn polygon_oblique_edge_uses_the_registered_segment_projection() {
    let polygon = [point(0, 0), point(-65_536, -3_019), point(0, 100_000)];

    assert!(polygon_stroke_contains(
        &polygon,
        20,
        point(-60_022, -2_755)
    ));
}

#[test]
fn polygon_oblique_edge_uses_registered_closest_coordinate_rounding() {
    let polygon = [
        point(0, 0),
        point(-8, -7),
        point(-100, -7),
        point(-100, 100),
        point(0, 100),
    ];

    assert!(!polygon_stroke_contains(&polygon, 12, point(-8, 1)));
}

#[test]
fn polygon_oblique_edge_rounds_to_the_nearest_lower_parameter() {
    let polygon = [
        point(0, 0),
        point(-18, -6),
        point(-100, -6),
        point(-100, 100),
        point(0, 100),
    ];

    assert!(polygon_stroke_contains(&polygon, 3, point(-11, -2)));
}

#[test]
fn long_polygon_edge_uses_the_quantized_closest_point() {
    let polygon = [
        point(0, 0),
        point(200_000, 0),
        point(200_000, 20),
        point(0, 20),
    ];

    assert!(!polygon_stroke_contains(&polygon, 1, point(1, 0)));
}

#[test]
fn polygon_projection_rounds_nearest_instead_of_floor_or_ceil() {
    let floor_case = [
        point(0, 0),
        point(65_537, 0),
        point(65_537, 100),
        point(0, 100),
    ];
    let ceil_case = [
        point(0, 0),
        point(65_535, 0),
        point(65_535, 100),
        point(0, 100),
    ];

    assert!(polygon_stroke_contains(&floor_case, 20, point(1, -10)));
    assert!(polygon_stroke_contains(&ceil_case, 20, point(1, -10)));
}
