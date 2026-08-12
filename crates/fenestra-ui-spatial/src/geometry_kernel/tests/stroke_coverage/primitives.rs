use super::*;

#[test]
fn rect_stroke_is_its_round_outline_not_its_fill_or_expanded_aabb() {
    for query in [
        point(50, 0),
        point(50, -5),
        point(100, 40),
        point(50, 80),
        point(-5, 40),
        point(-3, -4),
    ] {
        assert!(rect_stroke_contains(point(0, 0), 100, 80, 10, query));
    }
    for query in [point(50, 40), point(-4, -4), point(104, 84)] {
        assert!(!rect_stroke_contains(point(0, 0), 100, 80, 10, query));
    }
}

#[test]
fn translated_rect_stroke_uses_its_authored_origin() {
    let origin = point(10, -20);

    assert!(rect_stroke_contains(origin, 20, 10, 2, point(15, -21)));
    assert!(rect_stroke_contains(origin, 20, 10, 2, point(9, -15)));
    assert!(!rect_stroke_contains(origin, 20, 10, 2, point(20, -15)));
    assert!(!rect_stroke_contains(origin, 20, 10, 2, point(9, -21)));
}

#[test]
fn odd_rect_width_uses_the_exact_round_corner_inequality() {
    assert!(rect_stroke_contains(point(0, 0), 20, 20, 3, point(-1, -1)));
    assert!(!rect_stroke_contains(point(0, 0), 20, 20, 3, point(-2, 0)));
}

#[test]
fn long_rect_edge_uses_the_quantized_closest_point() {
    assert!(!rect_stroke_contains(
        point(0, 0),
        200_000,
        20,
        1,
        point(1, 0)
    ));
}

#[test]
fn rect_projection_rounds_nearest_instead_of_floor_or_ceil() {
    assert!(rect_stroke_contains(
        point(0, 0),
        65_537,
        100,
        20,
        point(1, -10)
    ));
    assert!(rect_stroke_contains(
        point(0, 0),
        65_535,
        100,
        20,
        point(1, -10)
    ));
}

#[test]
fn degenerate_rect_lines_and_point_retain_round_stroke_disks() {
    for query in [point(3, 10), point(3, -4)] {
        assert!(rect_stroke_contains(point(0, 0), 0, 20, 10, query));
    }
    assert!(!rect_stroke_contains(point(0, 0), 0, 20, 10, point(4, -4)));

    assert!(rect_stroke_contains(point(0, 0), 20, 0, 10, point(10, 5)));
    assert!(!rect_stroke_contains(point(0, 0), 20, 0, 10, point(10, 6)));

    assert!(rect_stroke_contains(point(0, 0), 0, 0, 10, point(3, 4)));
    assert!(!rect_stroke_contains(point(0, 0), 0, 0, 10, point(4, 4)));
}

#[test]
fn circle_annulus_uses_exact_even_and_odd_width_inequalities() {
    for distance in [4, 5, 6] {
        assert!(circle_stroke_contains(
            point(0, 0),
            5,
            2,
            point(distance, 0)
        ));
    }
    assert!(!circle_stroke_contains(point(0, 0), 5, 2, point(0, 0)));
    assert!(!circle_stroke_contains(point(0, 0), 5, 2, point(3, 0)));
    assert!(!circle_stroke_contains(point(0, 0), 5, 2, point(6, 6)));

    for distance in [4, 5, 6] {
        assert!(circle_stroke_contains(
            point(0, 0),
            5,
            3,
            point(distance, 0)
        ));
    }
    for distance in [3, 7] {
        assert!(!circle_stroke_contains(
            point(0, 0),
            5,
            3,
            point(distance, 0)
        ));
    }
    assert!(!circle_stroke_contains(point(0, 0), 5, 3, point(0, 0)));
    assert!(!circle_stroke_contains(point(0, 0), 5, 3, point(1, 3)));
    assert!(circle_stroke_contains(point(0, 0), 5, 3, point(2, 3)));
    assert!(circle_stroke_contains(point(0, 0), 5, 3, point(2, 6)));
    assert!(!circle_stroke_contains(point(0, 0), 5, 3, point(3, 6)));
}

#[test]
fn translated_circle_annulus_uses_signed_center_deltas() {
    let center = point(10, -20);

    assert!(!circle_stroke_contains(center, 5, 2, point(13, -20)));
    assert!(circle_stroke_contains(center, 5, 2, point(13, -24)));
    assert!(circle_stroke_contains(center, 5, 2, point(16, -20)));
    assert!(!circle_stroke_contains(center, 5, 2, point(16, -16)));
}

#[test]
fn circle_width_at_least_diameter_fills_center_but_keeps_exact_outer_radius() {
    assert!(circle_stroke_contains(point(0, 0), 3, 7, point(0, 0)));
    assert!(circle_stroke_contains(point(0, 0), 3, 7, point(6, 0)));
    assert!(!circle_stroke_contains(point(0, 0), 3, 7, point(7, 0)));
}

#[test]
fn zero_radius_circle_is_a_disk_with_widened_maximum_width_math() {
    assert!(circle_stroke_contains(point(7, -9), 0, 10, point(10, -5)));
    assert!(!circle_stroke_contains(point(7, -9), 0, 10, point(11, -5)));
    assert!(circle_stroke_contains(point(0, 0), 0, 2, point(1, 0)));
    assert!(!circle_stroke_contains(point(0, 0), 0, 2, point(1, 1)));
    assert!(circle_stroke_contains(point(0, 0), 0, 3, point(1, 1)));
    assert!(!circle_stroke_contains(point(0, 0), 0, 3, point(2, 0)));

    let width = SpatialScalarV2::MAX_RAW;
    let radius = width / 2;
    assert!(circle_stroke_contains(
        point(0, 0),
        0,
        width,
        point(radius, 0)
    ));
    assert!(!circle_stroke_contains(
        point(0, 0),
        0,
        width,
        point(radius, 1)
    ));
}
