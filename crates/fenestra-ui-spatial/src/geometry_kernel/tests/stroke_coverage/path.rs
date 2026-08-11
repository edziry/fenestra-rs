use super::*;

#[test]
fn open_path_has_round_ends_but_no_implicit_closing_stroke() {
    let open = [move_to(0, 0), line_to(20, 0), line_to(20, 20)];
    let closed = [
        move_to(0, 0),
        line_to(20, 0),
        line_to(20, 20),
        SpatialPathVerbV2::Close,
    ];

    assert!(!path_stroke_contains(&open, 2, point(10, 10)));
    assert!(path_stroke_contains(&closed, 2, point(10, 10)));
}

#[test]
fn open_path_endpoints_and_joins_contribute_round_disks() {
    let verbs = [move_to(0, 0), line_to(20, 0), line_to(20, 20)];

    for query in [point(-3, -4), point(23, -4), point(23, 24)] {
        assert!(path_stroke_contains(&verbs, 10, query));
    }
    for query in [point(-4, -4), point(24, -4), point(24, 24)] {
        assert!(!path_stroke_contains(&verbs, 10, query));
    }
}

#[test]
fn retained_zero_length_path_segment_is_one_exact_disk() {
    let verbs = [move_to(0, 0), line_to(0, 0)];
    let width = SpatialScalarV2::MAX_RAW;
    let radius = width / 2;

    assert!(path_stroke_contains(&verbs, width, point(radius, 0)));
    assert!(!path_stroke_contains(&verbs, width, point(radius, 1)));
}

#[test]
fn zero_length_segment_remains_a_disk_beside_a_normal_subpath() {
    let verbs = [
        move_to(0, 0),
        line_to(10, 0),
        move_to(100, 100),
        line_to(100, 100),
    ];

    assert!(path_stroke_contains(&verbs, 10, point(103, 104)));
    assert!(!path_stroke_contains(&verbs, 10, point(104, 104)));
}

#[test]
fn path_segments_never_bridge_adjacent_subpath_descriptors_or_mutate_them() {
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
    let (path, bounds, stroke) = flattened_path_with_stroke_bounds(&verbs, 2);
    let snapshot_points = path.points().to_vec();
    let snapshot_subpaths: Vec<_> = path
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
    let snapshot_segment_count = path.segment_count();

    assert!(!path_round_stroke_contains_k5(
        &path,
        bounds,
        stroke,
        point(7, 0)
    ));

    let observed_subpaths: Vec<_> = path
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
    assert_eq!(path.points(), snapshot_points.as_slice());
    assert_eq!(observed_subpaths, snapshot_subpaths);
    assert_eq!(path.segment_count(), snapshot_segment_count);
}

#[test]
fn path_stroke_uses_every_flattened_curve_vertex() {
    let verbs = [
        move_to(0, 0),
        SpatialPathVerbV2::QuadraticTo {
            control: point(0, 257),
            to: point(512, 0),
        },
    ];
    let (path, bounds, stroke) = flattened_path_with_stroke_bounds(&verbs, 10);

    assert_eq!(
        path.points(),
        &[point(0, 0), point(128, 129), point(512, 0)]
    );
    assert!(path_round_stroke_contains_k5(
        &path,
        bounds,
        stroke,
        point(128, 134)
    ));
    assert!(!path_round_stroke_contains_k5(
        &path,
        bounds,
        stroke,
        point(128, 135)
    ));
}
