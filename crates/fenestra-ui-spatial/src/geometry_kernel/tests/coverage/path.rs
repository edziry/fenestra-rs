use super::*;

#[test]
fn open_subpath_closes_for_fill_without_mutating_k2_storage() {
    let verbs = [move_to(0, 0), line_to(10, 0), line_to(0, 10)];
    let (flattened, bounds) = flattened_path_with_bounds(&verbs);
    let snapshot_points = flattened.points().to_vec();
    let snapshot_subpaths: Vec<_> = flattened
        .subpaths()
        .iter()
        .map(|subpath| {
            (
                subpath.point_start(),
                subpath.point_length(),
                subpath.is_explicitly_closed(),
            )
        })
        .collect();
    let snapshot_segment_count = flattened.segment_count();

    assert_eq!(flattened.segment_count(), 2);
    assert!(!flattened.subpaths()[0].is_explicitly_closed());
    for rule in SpatialFillRuleV2::ALL {
        assert!(path_fill_contains_k4(&flattened, bounds, rule, point(0, 5)));
        assert!(path_fill_contains_k4(&flattened, bounds, rule, point(2, 2)));
        assert!(!path_fill_contains_k4(
            &flattened,
            bounds,
            rule,
            point(8, 8)
        ));
    }
    let observed_subpaths: Vec<_> = flattened
        .subpaths()
        .iter()
        .map(|subpath| {
            (
                subpath.point_start(),
                subpath.point_length(),
                subpath.is_explicitly_closed(),
            )
        })
        .collect();
    assert_eq!(flattened.points(), snapshot_points.as_slice());
    assert_eq!(observed_subpaths, snapshot_subpaths);
    assert_eq!(flattened.segment_count(), snapshot_segment_count);
}

#[test]
fn quadratic_fill_uses_every_flattened_vertex() {
    let verbs = [
        move_to(0, 0),
        SpatialPathVerbV2::QuadraticTo {
            control: point(0, 257),
            to: point(512, 0),
        },
    ];
    let (flattened, bounds) = flattened_path_with_bounds(&verbs);

    assert_eq!(
        flattened.points(),
        &[point(0, 0), point(128, 129), point(512, 0)]
    );
    for rule in SpatialFillRuleV2::ALL {
        assert!(path_fill_contains_k4(
            &flattened,
            bounds,
            rule,
            point(128, 129)
        ));
        assert!(path_fill_contains_k4(
            &flattened,
            bounds,
            rule,
            point(128, 64)
        ));
        assert!(!path_fill_contains_k4(
            &flattened,
            bounds,
            rule,
            point(128, 200)
        ));
    }
}

#[test]
fn path_ray_counts_a_shared_vertex_with_half_open_y_intervals() {
    let verbs = [move_to(0, 0), line_to(10, 5), line_to(0, 10)];

    assert!(path_contains(
        &verbs,
        SpatialFillRuleV2::EvenOdd,
        point(1, 5)
    ));
    assert!(!path_contains(
        &verbs,
        SpatialFillRuleV2::EvenOdd,
        point(-1, 5)
    ));
}

#[test]
fn path_boundary_requires_collinearity_inside_the_segment_range() {
    let verbs = [
        move_to(0, 0),
        line_to(4, 0),
        line_to(8, 8),
        line_to(0, 8),
        SpatialPathVerbV2::Close,
    ];

    for rule in SpatialFillRuleV2::ALL {
        assert!(!path_contains(&verbs, rule, point(6, 0)));
    }
}

#[test]
fn path_segments_never_bridge_adjacent_subpath_descriptors() {
    let verbs = [
        move_to(0, 0),
        line_to(4, 0),
        line_to(4, 4),
        line_to(0, 4),
        SpatialPathVerbV2::Close,
        move_to(10, 0),
        line_to(14, 0),
        line_to(14, 4),
        line_to(10, 4),
        SpatialPathVerbV2::Close,
    ];

    for rule in SpatialFillRuleV2::ALL {
        assert!(!path_contains(&verbs, rule, point(7, 0)));
    }
}

#[test]
fn explicitly_closed_subpath_uses_its_stored_closing_segment() {
    let verbs = [
        move_to(0, 0),
        line_to(10, 0),
        line_to(0, 10),
        SpatialPathVerbV2::Close,
    ];
    let (flattened, bounds) = flattened_path_with_bounds(&verbs);

    assert_eq!(flattened.segment_count(), 3);
    assert!(flattened.subpaths()[0].is_explicitly_closed());
    assert_eq!(flattened.points().len(), 4);
    for rule in SpatialFillRuleV2::ALL {
        assert!(path_fill_contains_k4(&flattened, bounds, rule, point(0, 5)));
        assert!(path_fill_contains_k4(&flattened, bounds, rule, point(2, 2)));
        assert!(!path_fill_contains_k4(
            &flattened,
            bounds,
            rule,
            point(8, 8)
        ));
    }
}

#[test]
fn line_and_point_degenerates_ignore_only_zero_length_segments() {
    let line = [move_to(-5, 0), line_to(5, 0)];
    let point_path = [move_to(2, 2), line_to(2, 2)];
    let triangle_with_zero = [
        move_to(0, 0),
        line_to(10, 0),
        line_to(10, 0),
        line_to(0, 10),
    ];

    for rule in SpatialFillRuleV2::ALL {
        assert!(path_contains(&line, rule, point(0, 0)));
        assert!(!path_contains(&line, rule, point(0, 1)));
        assert!(!path_contains(&point_path, rule, point(2, 2)));
        assert!(path_contains(&triangle_with_zero, rule, point(2, 2)));
        assert!(!path_contains(&triangle_with_zero, rule, point(8, 8)));
    }
}

#[test]
fn same_winding_subpaths_distinguish_nonzero_from_even_odd() {
    let verbs = [
        move_to(0, 0),
        line_to(10, 0),
        line_to(10, 10),
        line_to(0, 10),
        SpatialPathVerbV2::Close,
        move_to(0, 0),
        line_to(10, 0),
        line_to(10, 10),
        line_to(0, 10),
        SpatialPathVerbV2::Close,
    ];

    assert!(path_contains(
        &verbs,
        SpatialFillRuleV2::NonZero,
        point(5, 5)
    ));
    assert!(!path_contains(
        &verbs,
        SpatialFillRuleV2::EvenOdd,
        point(5, 5)
    ));
    assert!(path_contains(
        &verbs,
        SpatialFillRuleV2::EvenOdd,
        point(0, 5)
    ));
}

#[test]
fn reversed_inner_subpath_cuts_a_hole_but_preserves_the_ring() {
    let verbs = [
        move_to(0, 0),
        line_to(12, 0),
        line_to(12, 12),
        line_to(0, 12),
        SpatialPathVerbV2::Close,
        move_to(3, 3),
        line_to(3, 9),
        line_to(9, 9),
        line_to(9, 3),
        SpatialPathVerbV2::Close,
    ];

    for rule in SpatialFillRuleV2::ALL {
        assert!(!path_contains(&verbs, rule, point(6, 6)));
        assert!(path_contains(&verbs, rule, point(1, 6)));
        assert!(path_contains(&verbs, rule, point(3, 6)));
    }
}

#[test]
fn path_boundary_math_widens_across_canonical_extremes() {
    let minimum = SpatialScalarV2::MIN_RAW;
    let maximum = SpatialScalarV2::MAX_RAW;
    let verbs = [move_to(minimum, minimum), line_to(maximum, maximum)];

    for rule in SpatialFillRuleV2::ALL {
        assert!(path_contains(&verbs, rule, point(0, 0)));
        assert!(!path_contains(&verbs, rule, point(0, 1)));
    }
}
