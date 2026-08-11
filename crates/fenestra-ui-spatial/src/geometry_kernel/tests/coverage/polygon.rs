use super::*;

#[test]
fn polygon_boundary_is_inclusive_and_horizontal_edges_do_not_add_crossings() {
    let square = [point(0, 0), point(10, 0), point(10, 10), point(0, 10)];

    for rule in SpatialFillRuleV2::ALL {
        for query in [
            point(0, 0),
            point(5, 0),
            point(10, 5),
            point(5, 10),
            point(0, 5),
            point(5, 5),
        ] {
            assert!(polygon_contains(&square, rule, query));
        }
        for query in [
            point(5, -1),
            point(-1, 0),
            point(11, 0),
            point(11, 5),
            point(5, 11),
        ] {
            assert!(!polygon_contains(&square, rule, query));
        }
    }
}

#[test]
fn polygon_ray_counts_a_shared_vertex_with_half_open_y_intervals() {
    let triangle = [point(0, 0), point(10, 5), point(0, 10)];

    assert!(polygon_contains(
        &triangle,
        SpatialFillRuleV2::EvenOdd,
        point(1, 5)
    ));
    assert!(!polygon_contains(
        &triangle,
        SpatialFillRuleV2::EvenOdd,
        point(-1, 5)
    ));
}

#[test]
fn polygon_boundary_requires_collinearity_inside_the_segment_range() {
    let polygon = [point(0, 0), point(4, 0), point(8, 8), point(0, 8)];

    for rule in SpatialFillRuleV2::ALL {
        assert!(!polygon_contains(&polygon, rule, point(6, 0)));
    }
}

#[test]
fn concave_polygon_rejects_its_aabb_notch_in_either_direction() {
    let polygon = [
        point(0, 0),
        point(8, 0),
        point(8, 3),
        point(3, 3),
        point(3, 8),
        point(0, 8),
    ];
    let reversed = [
        point(0, 8),
        point(3, 8),
        point(3, 3),
        point(8, 3),
        point(8, 0),
        point(0, 0),
    ];

    for points in [&polygon[..], &reversed[..]] {
        for rule in SpatialFillRuleV2::ALL {
            assert!(polygon_contains(points, rule, point(1, 7)));
            assert!(polygon_contains(points, rule, point(6, 3)));
            assert!(!polygon_contains(points, rule, point(6, 6)));
        }
    }
}

#[test]
fn self_intersection_retains_boundary_lobes_and_empty_regions() {
    let bow_tie = [point(0, 0), point(8, 8), point(0, 8), point(8, 0)];

    for rule in SpatialFillRuleV2::ALL {
        assert!(polygon_contains(&bow_tie, rule, point(4, 4)));
        assert!(polygon_contains(&bow_tie, rule, point(4, 1)));
        assert!(!polygon_contains(&bow_tie, rule, point(1, 4)));
    }
}

#[test]
fn overlapping_polygon_winding_distinguishes_both_fill_rules() {
    let twice = [
        point(0, 0),
        point(10, 0),
        point(10, 10),
        point(0, 10),
        point(0, 0),
        point(10, 0),
        point(10, 10),
        point(0, 10),
    ];

    assert!(polygon_contains(
        &twice,
        SpatialFillRuleV2::NonZero,
        point(5, 5)
    ));
    assert!(!polygon_contains(
        &twice,
        SpatialFillRuleV2::EvenOdd,
        point(5, 5)
    ));
    assert!(polygon_contains(
        &twice,
        SpatialFillRuleV2::EvenOdd,
        point(0, 5)
    ));
}

#[test]
fn collinear_polygon_has_only_its_nonzero_boundary_line() {
    let line = [point(-5, 0), point(0, 0), point(5, 0)];

    for rule in SpatialFillRuleV2::ALL {
        assert!(polygon_contains(&line, rule, point(3, 0)));
        assert!(!polygon_contains(&line, rule, point(3, 1)));
    }
}

#[test]
fn polygon_cross_and_boundary_math_widen_across_canonical_extremes() {
    let minimum = SpatialScalarV2::MIN_RAW;
    let maximum = SpatialScalarV2::MAX_RAW;
    let triangle = [
        point(minimum, minimum),
        point(maximum, maximum),
        point(minimum, maximum),
    ];

    for rule in SpatialFillRuleV2::ALL {
        assert!(polygon_contains(&triangle, rule, point(0, 0)));
        assert!(polygon_contains(&triangle, rule, point(0, 1)));
        assert!(!polygon_contains(&triangle, rule, point(1, 0)));
    }
}
