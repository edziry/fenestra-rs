use super::*;

#[test]
fn per_path_limit_wins_when_the_same_line_would_cross_the_total() {
    let verbs = [move_to(0, 0), line_to(1, 1)];

    expect_k2_limit(
        flatten(&verbs, 4, 0, 4),
        GeometryK2LimitKind::FlattenedSegmentsPerPath,
        1,
        1,
        0,
    );
}

#[test]
fn explicit_close_limit_failure_names_the_close_verb() {
    let verbs = [move_to(0, 0), line_to(1, 1), SpatialPathVerbV2::Close];

    expect_k2_limit(
        flatten(&verbs, 0, 1, 8),
        GeometryK2LimitKind::FlattenedSegmentsPerPath,
        2,
        2,
        1,
    );
}

#[test]
fn every_curve_leaf_retains_its_authored_source_verb() {
    let verbs = [
        move_to(0, 0),
        line_to(1, 0),
        SpatialPathVerbV2::QuadraticTo {
            control: point(1, 257),
            to: point(513, 0),
        },
    ];

    expect_k2_limit(
        flatten(&verbs, 0, 2, 8),
        GeometryK2LimitKind::FlattenedSegmentsPerPath,
        2,
        3,
        2,
    );
}

#[test]
fn registered_path_limit_rejects_the_4097th_curve_leaf() {
    const FOUR_TO_12: i64 = 16_777_216;
    const HEIGHT: i64 = 257 * FOUR_TO_12;
    let verbs = [
        move_to(-HEIGHT, 0),
        SpatialPathVerbV2::QuadraticTo {
            control: point(0, HEIGHT),
            to: point(HEIGHT, 0),
        },
    ];

    expect_k2_limit(
        flatten(&verbs, 0, FLATTENED_PER_PATH_MAXIMUM, usize::MAX),
        GeometryK2LimitKind::FlattenedSegmentsPerPath,
        1,
        FLATTENED_PER_PATH_MAXIMUM + 1,
        FLATTENED_PER_PATH_MAXIMUM,
    );
}

#[test]
fn a_line_limit_failure_precedes_a_later_nonflat_curve() {
    let height = DEPTH_16_NONFLAT_HEIGHT;
    let verbs = [
        move_to(-height, 0),
        line_to(-height, 0),
        SpatialPathVerbV2::QuadraticTo {
            control: point(0, height),
            to: point(height, 0),
        },
    ];

    expect_k2_limit(
        flatten(&verbs, 0, 0, usize::MAX),
        GeometryK2LimitKind::FlattenedSegmentsPerPath,
        1,
        1,
        0,
    );
}

#[test]
fn a_nonflat_curve_after_an_accepted_line_names_the_curve_verb() {
    let height = DEPTH_16_NONFLAT_HEIGHT;
    let verbs = [
        move_to(-height, 0),
        line_to(-height, 0),
        SpatialPathVerbV2::QuadraticTo {
            control: point(0, height),
            to: point(height, 0),
        },
    ];

    expect_k2_error(
        flatten(&verbs, 0, usize::MAX, usize::MAX),
        GeometryK2ErrorKind::NonFlatAtMaximumDepth,
        2,
    );
}

#[test]
fn a_nonflat_curve_precedes_a_later_line_limit_failure() {
    let height = DEPTH_16_NONFLAT_HEIGHT;
    let verbs = [
        move_to(-height, 0),
        SpatialPathVerbV2::QuadraticTo {
            control: point(0, height),
            to: point(height, 0),
        },
        line_to(height, 0),
    ];

    expect_k2_error(
        flatten(&verbs, 0, 0, usize::MAX),
        GeometryK2ErrorKind::NonFlatAtMaximumDepth,
        1,
    );
}

#[test]
fn total_limit_crosses_on_the_first_curve_leaf_not_after_the_path() {
    let verbs = [
        move_to(0, 0),
        SpatialPathVerbV2::QuadraticTo {
            control: point(0, 257),
            to: point(512, 0),
        },
    ];

    expect_k2_limit(
        flatten(&verbs, 10, 2, 10),
        GeometryK2LimitKind::FlattenedSegmentsTotal,
        1,
        11,
        10,
    );
}

#[test]
fn failed_flattening_does_not_commit_a_total_or_publish_partial_points() {
    let accepted_total = 10;
    let verbs = [
        move_to(0, 0),
        SpatialPathVerbV2::QuadraticTo {
            control: point(0, 257),
            to: point(512, 0),
        },
    ];

    let failed = flatten(&verbs, accepted_total, 2, 11);
    assert!(failed.is_err());
    assert_eq!(accepted_total, 10);

    let flattened = expect_flattened(flatten(&verbs, accepted_total, 2, 12));
    assert_points(&flattened, &[point(0, 0), point(128, 129), point(512, 0)]);
    assert_eq!(flattened.segment_count(), 2);
    assert_eq!(accepted_total, 10);
}

#[test]
fn registered_total_limit_is_inclusive_and_reports_the_first_crossing() {
    assert_eq!(FLATTENED_TOTAL_MAXIMUM, 65_536);
    let verbs = [move_to(0, 0), line_to(1, 1)];

    let flattened = expect_flattened(flatten(
        &verbs,
        FLATTENED_TOTAL_MAXIMUM - 1,
        1,
        FLATTENED_TOTAL_MAXIMUM,
    ));
    assert_eq!(flattened.segment_count(), 1);

    expect_k2_limit(
        flatten(&verbs, FLATTENED_TOTAL_MAXIMUM, 1, FLATTENED_TOTAL_MAXIMUM),
        GeometryK2LimitKind::FlattenedSegmentsTotal,
        1,
        FLATTENED_TOTAL_MAXIMUM + 1,
        FLATTENED_TOTAL_MAXIMUM,
    );
}
