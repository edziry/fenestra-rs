use super::*;

#[test]
fn rect_uses_closed_bounds_only_to_reject_then_applies_half_open_far_edges() {
    let origin = point(10, 20);

    for query in [point(10, 20), point(10, 26), point(14, 20), point(14, 26)] {
        assert!(rect_contains(origin, 5, 7, query));
    }
    for query in [
        point(15, 20),
        point(15, 26),
        point(10, 27),
        point(14, 27),
        point(9, 20),
        point(10, 19),
    ] {
        assert!(!rect_contains(origin, 5, 7, query));
    }
}

#[test]
fn zero_rect_axis_is_empty_even_at_its_closed_base_boundary() {
    for (width, height, boundary) in [
        (0, 5, point(2, 5)),
        (5, 0, point(5, 3)),
        (0, 0, point(2, 3)),
    ] {
        let rect = expect_valid(validate_rect_k1(
            SHAPE_INDEX,
            point(2, 3),
            scalar(width),
            scalar(height),
        ));
        let derived = match derive_rect_bounds_k3(SHAPE_INDEX, rect) {
            Ok(derived) => derived,
            Err(error) => panic!("expected rect K3 success, got {error:?}"),
        };
        let bounds = fill_bounds_k3(&derived);

        assert!(bounds.is_empty());
        assert!(!rect_fill_contains_k4(rect, bounds, boundary));
    }
}

#[test]
fn circle_uses_exact_closed_distance_after_its_square_aabb() {
    for query in [
        point(0, 0),
        point(1, 1),
        point(0, -5),
        point(3, 4),
        point(5, 0),
    ] {
        assert!(circle_contains(point(0, 0), 5, query));
    }
    for query in [point(4, 4), point(5, 1), point(6, 0)] {
        assert!(!circle_contains(point(0, 0), 5, query));
    }
}

#[test]
fn translated_circle_applies_signed_deltas_from_its_center() {
    let center = point(10, -20);

    for query in [point(10, -20), point(9, -19), point(7, -24), point(15, -20)] {
        assert!(circle_contains(center, 5, query));
    }
    for query in [point(6, -16), point(14, -24), point(10, -26)] {
        assert!(!circle_contains(center, 5, query));
    }
}

#[test]
fn zero_radius_circle_is_empty_even_at_its_center() {
    let circle = expect_valid(validate_circle_k1(SHAPE_INDEX, point(7, -9), scalar(0)));
    let derived = match derive_circle_bounds_k3(SHAPE_INDEX, circle) {
        Ok(derived) => derived,
        Err(error) => panic!("expected circle K3 success, got {error:?}"),
    };
    let bounds = fill_bounds_k3(&derived);

    assert!(bounds.is_empty());
    assert!(!circle_fill_contains_k4(circle, bounds, point(7, -9)));
}

#[test]
fn circle_distance_widens_at_the_canonical_maximum() {
    let radius = SpatialScalarV2::MAX_RAW;

    assert!(circle_contains(point(0, 0), radius, point(radius, 0)));
    assert!(!circle_contains(point(0, 0), radius, point(radius, 1)));
}
