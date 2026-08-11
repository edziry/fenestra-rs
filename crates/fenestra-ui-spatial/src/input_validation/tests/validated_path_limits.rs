use super::map_path_k1_error_stage;
use super::validated_path_support::{
    expect_limit, expect_valid, fixture, limits, line_to, move_to, path, validate,
};
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::{SpatialPathFieldV2, SpatialPathVerbFieldV2};
use crate::geometry_kernel::validate_path_k1;
use crate::limits::{REGISTERED_SPATIAL_LIMITS_V2, SpatialLimitKindV2};
use crate::model::SpatialScalarV2;

#[test]
fn path_verbs_per_path_is_one_complete_global_pass_before_k1() {
    let invalid = SpatialScalarV2::MAX_RAW + 1;
    let later_crossing = fixture(
        vec![path(0, 0, 2), path(1, 2, 3)],
        vec![
            move_to(invalid, 0),
            line_to(0, 0),
            move_to(0, 0),
            line_to(1, 1),
            crate::path::SpatialPathVerbV2::Close,
        ],
    );

    expect_limit(
        validate(&later_crossing, limits(2, 0)),
        SpatialLimitKindV2::PathVerbsPerPath,
        SpatialErrorLocationV2::Path {
            index: 1,
            field: SpatialPathFieldV2::VerbLength,
        },
        3,
        2,
    );

    let first = fixture(
        vec![path(0, 0, 2), path(1, 2, 2)],
        vec![move_to(0, 0), line_to(1, 1), move_to(2, 2), line_to(3, 3)],
    );
    expect_limit(
        validate(&first, limits(1, usize::MAX)),
        SpatialLimitKindV2::PathVerbsPerPath,
        SpatialErrorLocationV2::Path {
            index: 0,
            field: SpatialPathFieldV2::VerbLength,
        },
        2,
        1,
    );
}

#[test]
fn custom_per_path_limit_accepts_equality_and_rejects_one_over() {
    let verbs = vec![
        move_to(0, 0),
        line_to(1, 1),
        crate::path::SpatialPathVerbV2::Close,
    ];
    let fixture = fixture(vec![path(0, 0, 3)], verbs);

    expect_valid(validate(&fixture, limits(3, 1)));
    expect_limit(
        validate(&fixture, limits(2, 1)),
        SpatialLimitKindV2::PathVerbsPerPath,
        SpatialErrorLocationV2::Path {
            index: 0,
            field: SpatialPathFieldV2::VerbLength,
        },
        3,
        2,
    );
}

#[test]
fn caller_per_path_limit_is_not_replaced_by_the_registered_profile() {
    let registered = REGISTERED_SPATIAL_LIMITS_V2.limit(SpatialLimitKindV2::PathVerbsPerPath);
    assert_eq!(registered, 256);

    let edge = one_subpath(registered);
    let edge_fixture = fixture(vec![path(0, 0, registered as u32)], edge);
    expect_valid(validate(&edge_fixture, limits(registered, 1)));

    let one_over = registered + 1;
    let over_fixture = fixture(vec![path(0, 0, one_over as u32)], one_subpath(one_over));
    expect_limit(
        validate(&over_fixture, limits(registered, 1)),
        SpatialLimitKindV2::PathVerbsPerPath,
        SpatialErrorLocationV2::Path {
            index: 0,
            field: SpatialPathFieldV2::VerbLength,
        },
        one_over as u128,
        registered as u128,
    );
    expect_valid(validate(&over_fixture, limits(one_over, 1)));
}

#[test]
fn cumulative_subpaths_report_the_first_crossing_move_and_exact_evidence() {
    let verbs = subpaths(3);
    let fixture = fixture(vec![path(0, 0, 2), path(1, 2, 4)], verbs);

    expect_limit(
        validate(&fixture, limits(4, 2)),
        SpatialLimitKindV2::PathSubpathsTotal,
        SpatialErrorLocationV2::PathVerb {
            path: 1,
            verb: 2,
            field: SpatialPathVerbFieldV2::Kind,
        },
        3,
        2,
    );
    expect_valid(validate(&fixture, limits(4, 3)));
}

#[test]
fn caller_subpath_limit_is_not_replaced_by_the_registered_profile() {
    let registered = REGISTERED_SPATIAL_LIMITS_V2.limit(SpatialLimitKindV2::PathSubpathsTotal);
    assert_eq!(registered, 1_024);

    let edge = subpaths(registered);
    let edge_fixture = fixture(vec![path(0, 0, edge.len() as u32)], edge);
    expect_valid(validate(&edge_fixture, limits(registered * 2, registered)));

    let one_over = registered + 1;
    let over = subpaths(one_over);
    let over_fixture = fixture(vec![path(0, 0, over.len() as u32)], over);
    expect_limit(
        validate(&over_fixture, limits(one_over * 2, registered)),
        SpatialLimitKindV2::PathSubpathsTotal,
        SpatialErrorLocationV2::PathVerb {
            path: 0,
            verb: (registered * 2) as u32,
            field: SpatialPathVerbFieldV2::Kind,
        },
        one_over as u128,
        registered as u128,
    );
    expect_valid(validate(&over_fixture, limits(one_over * 2, one_over)));
}

#[test]
fn k1_evidence_above_usize_is_preserved_by_the_aggregate_mapper() {
    let verbs = [move_to(0, 0), line_to(1, 1)];
    let error = match validate_path_k1(u32::MAX, &verbs, usize::MAX, usize::MAX) {
        Ok(_) => panic!("expected the synthetic cumulative K1 crossing"),
        Err(error) => error,
    };
    let mapped = map_path_k1_error_stage(error);

    expect_limit(
        Err::<(), _>(mapped),
        SpatialLimitKindV2::PathSubpathsTotal,
        SpatialErrorLocationV2::PathVerb {
            path: u32::MAX,
            verb: 0,
            field: SpatialPathVerbFieldV2::Kind,
        },
        usize::MAX as u128 + 1,
        usize::MAX as u128,
    );
}

fn one_subpath(length: usize) -> Vec<crate::path::SpatialPathVerbV2> {
    let mut verbs = Vec::with_capacity(length);
    verbs.push(move_to(0, 0));
    verbs.extend((1..length).map(|_| line_to(1, 1)));
    verbs
}

fn subpaths(count: usize) -> Vec<crate::path::SpatialPathVerbV2> {
    let mut verbs = Vec::with_capacity(count * 2);
    for index in 0..count {
        verbs.push(move_to(index as i64, 0));
        verbs.push(line_to(index as i64, 1));
    }
    verbs
}
