use super::flattened_path_support::{
    DEPTH_16_NONFLAT_HEIGHT, expect_limit, expect_nonflat, fixture, kernel_error, limits, line_to,
    map_error, move_to, path, quadratic, validate,
};
use crate::limits::SpatialLimitKindV2;
use crate::path::SpatialPathVerbV2;

#[test]
fn mapper_preserves_each_k2_kind_location_and_exact_evidence() {
    let line = [move_to(0, 0), line_to(1, 1)];
    expect_limit::<()>(
        Err(map_error(kernel_error(7, &line, 0, 0, 0))),
        SpatialLimitKindV2::FlattenedSegmentsPerPath,
        7,
        1,
        1,
        0,
    );
    expect_limit::<()>(
        Err(map_error(kernel_error(7, &line, 10, 1, 10))),
        SpatialLimitKindV2::FlattenedSegmentsTotal,
        7,
        1,
        11,
        10,
    );

    let nonflat = quadratic(DEPTH_16_NONFLAT_HEIGHT);
    expect_nonflat::<()>(
        Err(map_error(kernel_error(
            7,
            &nonflat,
            0,
            usize::MAX,
            usize::MAX,
        ))),
        7,
        1,
    );
}

#[test]
fn mapper_preserves_a_real_cumulative_candidate_beyond_usize() {
    let line = [move_to(0, 0), line_to(1, 1)];
    expect_limit::<()>(
        Err(map_error(kernel_error(7, &line, usize::MAX, 1, usize::MAX))),
        SpatialLimitKindV2::FlattenedSegmentsTotal,
        7,
        1,
        usize::MAX as u128 + 1,
        usize::MAX as u128,
    );
}

#[test]
fn close_and_every_curve_leaf_keep_their_authored_source_verb() {
    let close = fixture(
        vec![path(0, 0, 3)],
        vec![move_to(0, 0), line_to(1, 0), SpatialPathVerbV2::Close],
    );
    expect_limit(
        validate(&close, limits(1, usize::MAX)),
        SpatialLimitKindV2::FlattenedSegmentsPerPath,
        0,
        2,
        2,
        1,
    );

    let curve = fixture(
        vec![path(0, 0, 3)],
        vec![
            move_to(0, 0),
            line_to(1, 0),
            SpatialPathVerbV2::QuadraticTo {
                control: super::flattened_path_support::point(1, 257),
                to: super::flattened_path_support::point(513, 0),
            },
        ],
    );
    expect_limit(
        validate(&curve, limits(2, usize::MAX)),
        SpatialLimitKindV2::FlattenedSegmentsPerPath,
        0,
        2,
        3,
        2,
    );
}

#[test]
fn a_later_path_uses_its_local_source_verb_instead_of_the_payload_ordinal() {
    let fixture = fixture(
        vec![path(0, 0, 2), path(1, 2, 3)],
        vec![
            move_to(0, 0),
            line_to(1, 0),
            move_to(2, 0),
            line_to(3, 0),
            line_to(4, 0),
        ],
    );
    expect_limit(
        validate(&fixture, limits(1, usize::MAX)),
        SpatialLimitKindV2::FlattenedSegmentsPerPath,
        1,
        2,
        2,
        1,
    );
}
