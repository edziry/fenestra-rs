use super::*;

#[test]
fn segment_projection_rounds_the_parameter_before_clamping() {
    let verbs = [move_to(0, 0), line_to(-65_536, -3_019)];

    assert!(path_stroke_contains(&verbs, 20, point(-60_022, -2_755)));
}

#[test]
fn segment_projection_rounds_to_the_nearest_lower_parameter() {
    let verbs = [move_to(0, 0), line_to(-18, -6)];

    assert!(path_stroke_contains(&verbs, 3, point(-11, -2)));
}

#[test]
fn segment_projection_clamps_to_round_endpoint_disks() {
    let verbs = [move_to(0, 0), line_to(20, 0)];

    for query in [point(-3, 4), point(23, 4)] {
        assert!(path_stroke_contains(&verbs, 10, query));
    }
    for query in [point(-4, 4), point(24, 4)] {
        assert!(!path_stroke_contains(&verbs, 10, query));
    }
}

#[test]
fn segment_projection_clamps_to_the_exact_zero_and_65536_endpoints() {
    let verbs = [move_to(0, 0), line_to(65_536, 0)];

    assert!(path_stroke_contains(&verbs, 2, point(-1, 0)));
    assert!(path_stroke_contains(&verbs, 2, point(65_537, 0)));
}

#[test]
fn closest_coordinates_round_signed_half_ties_away_from_zero() {
    let verbs = [move_to(0, 0), line_to(-1, -3)];

    assert!(path_stroke_contains(&verbs, 3, point(-2, -1)));
}

#[test]
fn closest_coordinates_each_use_registered_rounding() {
    let verbs = [move_to(0, 0), line_to(-8, -7)];

    assert!(!path_stroke_contains(&verbs, 12, point(-8, 1)));
}

#[test]
fn quantized_closest_point_is_not_replaced_by_continuous_distance() {
    let verbs = [move_to(0, 0), line_to(1, 2)];

    assert!(!path_stroke_contains(&verbs, 4, point(2, 0)));
}

#[test]
fn segment_projection_and_distance_widen_across_both_canonical_extremes() {
    let minimum = SpatialScalarV2::MIN_RAW + 1;
    let maximum = SpatialScalarV2::MAX_RAW - 1;
    let verbs = [move_to(minimum, minimum), line_to(maximum, maximum)];

    assert!(path_stroke_contains(&verbs, 1, point(0, 0)));
    assert!(!path_stroke_contains(&verbs, 1, point(0, 1)));
}
