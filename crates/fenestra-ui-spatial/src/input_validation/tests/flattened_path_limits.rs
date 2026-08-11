use super::flattened_path_support::{
    expect_limit, expect_valid, fixture, limits, line_to, move_to, path, quadratic, validate,
};
use super::local_transform_support::VIEWPORT;
use crate::limits::{REGISTERED_SPATIAL_LIMITS_V2, SpatialLimitKindV2};

#[test]
fn per_path_limit_precedes_total_on_the_same_emitted_segment() {
    let fixture = fixture(vec![path(0, 0, 2)], vec![move_to(0, 0), line_to(1, 1)]);
    expect_limit(
        validate(&fixture, limits(0, 0)),
        SpatialLimitKindV2::FlattenedSegmentsPerPath,
        0,
        1,
        1,
        0,
    );
}

#[test]
fn cumulative_total_uses_complete_prior_paths_and_equality_is_atomic() {
    let fixture = fixture(
        vec![path(0, 0, 3), path(1, 3, 2)],
        vec![
            move_to(0, 0),
            line_to(1, 0),
            line_to(2, 0),
            move_to(3, 0),
            line_to(4, 0),
        ],
    );
    expect_limit(
        validate(&fixture, limits(2, 2)),
        SpatialLimitKindV2::FlattenedSegmentsTotal,
        1,
        1,
        3,
        2,
    );

    let proof = expect_valid(prepare_flattened_paths!(&fixture, VIEWPORT, limits(2, 3)));
    assert_eq!(proof.accepted_flattened_segment_total(), 3);
}

#[test]
fn per_path_count_resets_for_each_declared_path() {
    let fixture = fixture(
        vec![path(0, 0, 2), path(1, 2, 2)],
        vec![move_to(0, 0), line_to(1, 0), move_to(2, 0), line_to(3, 0)],
    );
    let proof = expect_valid(prepare_flattened_paths!(&fixture, VIEWPORT, limits(1, 2)));
    assert_eq!(proof.accepted_flattened_segment_total(), 2);
}

#[test]
fn caller_per_path_limit_is_not_capped_by_the_registered_profile() {
    let registered =
        REGISTERED_SPATIAL_LIMITS_V2.limit(SpatialLimitKindV2::FlattenedSegmentsPerPath);
    assert_eq!(registered, 4_096);
    const HEIGHT: i64 = 256 * 67_108_864;
    let verbs = quadratic(HEIGHT).to_vec();
    let fixture = fixture(vec![path(0, 0, 2)], verbs);

    expect_limit(
        validate(&fixture, limits(8_191, usize::MAX)),
        SpatialLimitKindV2::FlattenedSegmentsPerPath,
        0,
        1,
        8_192,
        8_191,
    );
    let proof = expect_valid(prepare_flattened_paths!(
        &fixture,
        VIEWPORT,
        limits(8_192, usize::MAX)
    ));
    assert_eq!(proof.accepted_flattened_segment_total(), 8_192);
}

#[test]
fn caller_total_limit_is_not_capped_by_the_registered_profile() {
    let registered = REGISTERED_SPATIAL_LIMITS_V2.limit(SpatialLimitKindV2::FlattenedSegmentsTotal);
    assert_eq!(registered, 65_536);
    const HEIGHT: i64 = 256 * 4_294_967_296;
    let mut verbs = quadratic(HEIGHT).to_vec();
    verbs.extend(quadratic(HEIGHT));
    let fixture = fixture(vec![path(0, 0, 2), path(1, 2, 2)], verbs);

    expect_limit(
        validate(&fixture, limits(65_536, registered)),
        SpatialLimitKindV2::FlattenedSegmentsTotal,
        1,
        1,
        65_537,
        65_536,
    );
    let proof = expect_valid(prepare_flattened_paths!(
        &fixture,
        VIEWPORT,
        limits(65_536, 131_072)
    ));
    assert_eq!(proof.accepted_flattened_segment_total(), 131_072);
}
