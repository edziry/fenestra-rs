use super::*;

fn quadratic(control: SpatialPointV2, to: SpatialPointV2) -> [SpatialPathVerbV2; 2] {
    [
        move_to(0, 0),
        SpatialPathVerbV2::QuadraticTo { control, to },
    ]
}

fn cubic(control_y: i64) -> [SpatialPathVerbV2; 2] {
    [
        move_to(0, 0),
        SpatialPathVerbV2::CubicTo {
            control1: point(0, control_y),
            control2: point(512, control_y),
            to: point(512, 0),
        },
    ]
}

fn symmetric_quadratic(height: i64) -> [SpatialPathVerbV2; 2] {
    [
        move_to(-height, 0),
        SpatialPathVerbV2::QuadraticTo {
            control: point(0, height),
            to: point(height, 0),
        },
    ]
}

#[test]
fn nonzero_chord_flatness_accepts_256_and_splits_257() {
    let flat = quadratic(point(0, 256), point(512, 0));
    let flattened = expect_flattened(flatten(&flat, 0, 1, 1));
    assert_points(&flattened, &[point(0, 0), point(512, 0)]);
    assert_eq!(flattened.segment_count(), 1);

    let nonflat = quadratic(point(0, 257), point(512, 0));
    let flattened = expect_flattened(flatten(&nonflat, 0, 2, 2));
    assert_points(&flattened, &[point(0, 0), point(128, 129), point(512, 0)]);
    assert_eq!(flattened.segment_count(), 2);
}

#[test]
fn diagonal_chord_cross_accepts_128_and_splits_129_inside_the_bbox() {
    let flat = quadratic(point(384, 640), point(1_024, 1_024));
    let flattened = expect_flattened(flatten(&flat, 0, 1, 1));
    assert_points(&flattened, &[point(0, 0), point(1_024, 1_024)]);

    let nonflat = quadratic(point(383, 641), point(1_024, 1_024));
    let flattened = expect_flattened(flatten(&nonflat, 0, 2, 2));
    assert_points(
        &flattened,
        &[point(0, 0), point(448, 577), point(1_024, 1_024)],
    );
}

#[test]
fn flatness_scales_cross_by_the_larger_chord_axis() {
    let verbs = quadratic(point(1, 0), point(1, 512));
    let flattened = expect_flattened(flatten(&verbs, 0, 1, 1));

    assert_points(&flattened, &[point(0, 0), point(1, 512)]);
    assert_eq!(flattened.segment_count(), 1);
}

#[test]
fn endpoint_box_expansion_accepts_256_and_splits_257() {
    let flat = quadratic(point(-256, 0), point(512, 0));
    let flattened = expect_flattened(flatten(&flat, 0, 1, 1));
    assert_points(&flattened, &[point(0, 0), point(512, 0)]);

    let nonflat = quadratic(point(-257, 0), point(512, 0));
    let flattened = expect_flattened(flatten(&nonflat, 0, 2, 2));
    assert_points(&flattened, &[point(0, 0), point(-1, 0), point(512, 0)]);
}

#[test]
fn vertical_chord_bbox_y_accepts_256_and_splits_257_with_zero_cross() {
    let flat = quadratic(point(0, -256), point(0, 512));
    let flattened = expect_flattened(flatten(&flat, 0, 1, 1));
    assert_points(&flattened, &[point(0, 0), point(0, 512)]);

    let nonflat = quadratic(point(0, -257), point(0, 512));
    let flattened = expect_flattened(flatten(&nonflat, 0, 2, 2));
    assert_points(&flattened, &[point(0, 0), point(0, -1), point(0, 512)]);
}

#[test]
fn zero_chord_uses_the_per_coordinate_256_boundary() {
    let flat = quadratic(point(256, -256), point(0, 0));
    let flattened = expect_flattened(flatten(&flat, 0, 1, 1));
    assert_points(&flattened, &[point(0, 0), point(0, 0)]);
    assert_eq!(flattened.segment_count(), 1);

    for (control, midpoint) in [
        (point(257, 0), point(129, 0)),
        (point(0, 257), point(0, 129)),
    ] {
        let nonflat = quadratic(control, point(0, 0));
        let flattened = expect_flattened(flatten(&nonflat, 0, 2, 2));
        assert_points(&flattened, &[point(0, 0), midpoint, point(0, 0)]);
        assert_eq!(flattened.segment_count(), 2);
    }
}

#[test]
fn quadratic_casteljau_midpoints_round_half_ties_away_from_zero() {
    let cases = [(257, 129), (-257, -129)];

    for (control_y, midpoint_y) in cases {
        let verbs = quadratic(point(0, control_y), point(512, 0));
        let flattened = expect_flattened(flatten(&verbs, 0, 2, 2));
        assert_points(
            &flattened,
            &[point(0, 0), point(128, midpoint_y), point(512, 0)],
        );
    }
}

#[test]
fn quadratic_casteljau_rounds_each_midpoint_before_the_next() {
    let verbs = [
        move_to(-5, 0),
        SpatialPathVerbV2::QuadraticTo {
            control: point(-5, 257),
            to: point(-2, 0),
        },
    ];
    let flattened = expect_flattened(flatten(&verbs, 0, 2, 2));

    assert_points(&flattened, &[point(-5, 0), point(-5, 129), point(-2, 0)]);
    assert_eq!(flattened.segment_count(), 2);
}

#[test]
fn cubic_casteljau_splits_in_source_order_with_signed_ties() {
    let cases = [(257, 193), (-257, -193)];

    for (control_y, midpoint_y) in cases {
        let verbs = cubic(control_y);
        let flattened = expect_flattened(flatten(&verbs, 0, 2, 2));
        assert_points(
            &flattened,
            &[point(0, 0), point(256, midpoint_y), point(512, 0)],
        );
        assert_eq!(flattened.segment_count(), 2);
    }
}

#[test]
fn each_cubic_control_participates_in_the_flatness_test() {
    let cases = [(257, 0), (0, 257)];

    for (control1_y, control2_y) in cases {
        let verbs = [
            move_to(0, 0),
            SpatialPathVerbV2::CubicTo {
                control1: point(0, control1_y),
                control2: point(512, control2_y),
                to: point(512, 0),
            },
        ];
        let flattened = expect_flattened(flatten(&verbs, 0, 2, 2));
        assert_points(&flattened, &[point(0, 0), point(256, 97), point(512, 0)]);
        assert_eq!(flattened.segment_count(), 2);
    }
}

#[test]
fn casteljau_midpoints_widen_canonical_maximum_and_minimum_sums() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let minimum = SpatialScalarV2::MIN_RAW;
    let cases = [
        (maximum, 257, maximum - 512, maximum - 128, 129),
        (minimum, -257, minimum + 512, minimum + 128, -129),
    ];

    for (start_x, control_y, end_x, midpoint_x, midpoint_y) in cases {
        let verbs = [
            move_to(start_x, 0),
            SpatialPathVerbV2::QuadraticTo {
                control: point(start_x, control_y),
                to: point(end_x, 0),
            },
        ];
        let flattened = expect_flattened(flatten(&verbs, 0, 2, 2));
        assert_points(
            &flattened,
            &[
                point(start_x, 0),
                point(midpoint_x, midpoint_y),
                point(end_x, 0),
            ],
        );
    }
}

#[test]
fn flatness_cross_widens_past_i64_while_control_stays_inside_the_bbox() {
    let magnitude = SpatialScalarV2::MAX_RAW;
    let perpendicular = 32_769_i64;
    let chord_extent = 2_i128 * i128::from(magnitude);
    let cross = 4_i128 * i128::from(magnitude) * i128::from(perpendicular);
    assert!(cross > i128::from(i64::MAX));
    assert!(cross > 256_i128 * chord_extent);
    assert!((-magnitude..=magnitude).contains(&-perpendicular));
    assert!((-magnitude..=magnitude).contains(&perpendicular));

    let verbs = [
        move_to(-magnitude, -magnitude),
        SpatialPathVerbV2::QuadraticTo {
            control: point(-perpendicular, perpendicular),
            to: point(magnitude, magnitude),
        },
    ];
    let flattened = expect_flattened(flatten(&verbs, 0, usize::MAX, usize::MAX));
    assert!(flattened.segment_count() > 1);
    assert_eq!(
        flattened.points().first(),
        Some(&point(-magnitude, -magnitude))
    );
    assert_eq!(
        flattened.points().last(),
        Some(&point(magnitude, magnitude))
    );
}

#[test]
fn widened_cross_and_registered_path_limit_accept_exactly_4096_leaves() {
    const FOUR_TO_12: i64 = 16_777_216;
    const HEIGHT: i64 = 256 * FOUR_TO_12;
    let cross = 2_i128 * i128::from(HEIGHT) * i128::from(HEIGHT);
    assert!(cross > i128::from(i64::MAX));
    assert_eq!(FLATTENED_PER_PATH_MAXIMUM, 4_096);

    let verbs = symmetric_quadratic(HEIGHT);
    let flattened = expect_flattened(flatten(&verbs, 0, FLATTENED_PER_PATH_MAXIMUM, usize::MAX));
    assert_eq!(flattened.segment_count(), FLATTENED_PER_PATH_MAXIMUM);
    assert_eq!(flattened.points().len(), FLATTENED_PER_PATH_MAXIMUM + 1);
}

#[test]
fn flatness_is_tested_before_rejecting_depth_16() {
    const FOUR_TO_16: i64 = 4_294_967_296;
    const HEIGHT: i64 = 256 * FOUR_TO_16;
    let verbs = symmetric_quadratic(HEIGHT);

    let flattened = expect_flattened(flatten(&verbs, 0, usize::MAX, usize::MAX));
    assert_eq!(flattened.segment_count(), 65_536);
    assert_eq!(flattened.points().first(), Some(&point(-HEIGHT, 0)));
    assert_eq!(flattened.points().last(), Some(&point(HEIGHT, 0)));
}

#[test]
fn a_curve_still_nonflat_at_depth_16_fails_without_an_approximation() {
    let verbs = symmetric_quadratic(DEPTH_16_NONFLAT_HEIGHT);

    expect_k2_error(
        flatten(&verbs, 0, usize::MAX, usize::MAX),
        GeometryK2ErrorKind::NonFlatAtMaximumDepth,
        1,
    );
}
