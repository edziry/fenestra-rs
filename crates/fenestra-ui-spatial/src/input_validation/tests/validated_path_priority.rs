use super::path_structure_support::expect_invalid_range;
use super::validated_path_support::{
    expect_content, expect_limit, fixture, limits, line_to, move_to, path, permissive_limits,
    validate,
};
use crate::content_diagnostic::SpatialPathGrammarErrorV2;
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::{SpatialPathFieldV2, SpatialPathVerbFieldV2};
use crate::limits::SpatialLimitKindV2;
use crate::model::SpatialScalarV2;
use crate::path::SpatialPathVerbV2;

#[test]
fn the_complete_scalar_pass_precedes_earlier_grammar() {
    let fixture = fixture(
        vec![path(0, 0, 2)],
        vec![
            SpatialPathVerbV2::Close,
            move_to(SpatialScalarV2::MAX_RAW + 1, 0),
        ],
    );

    expect_content(
        validate(&fixture, permissive_limits()),
        SpatialContentErrorKindV2::ScalarOutOfDomain,
        SpatialErrorLocationV2::PathVerb {
            path: 0,
            verb: 1,
            field: SpatialPathVerbFieldV2::ToX,
        },
    );
}

#[test]
fn k1_runs_complete_paths_in_record_order() {
    let fixture = fixture(
        vec![path(0, 0, 1), path(1, 1, 2)],
        vec![
            SpatialPathVerbV2::Close,
            move_to(SpatialScalarV2::MIN_RAW - 1, 0),
            line_to(0, 0),
        ],
    );

    expect_content(
        validate(&fixture, permissive_limits()),
        SpatialContentErrorKindV2::InvalidPathGrammar(SpatialPathGrammarErrorV2::FirstNotMove),
        SpatialErrorLocationV2::PathVerb {
            path: 0,
            verb: 0,
            field: SpatialPathVerbFieldV2::Kind,
        },
    );
}

#[test]
fn complete_grammar_precedes_a_subpath_crossing_in_the_same_path() {
    let fixture = fixture(
        vec![path(0, 0, 3)],
        vec![move_to(0, 0), line_to(1, 1), move_to(2, 2)],
    );

    expect_content(
        validate(&fixture, limits(3, 0)),
        SpatialContentErrorKindV2::InvalidPathGrammar(SpatialPathGrammarErrorV2::TrailingMove),
        SpatialErrorLocationV2::PathVerb {
            path: 0,
            verb: 2,
            field: SpatialPathVerbFieldV2::Kind,
        },
    );
}

#[test]
fn an_earlier_subpath_crossing_precedes_a_later_scalar_failure() {
    let fixture = fixture(
        vec![path(0, 0, 2), path(1, 2, 2)],
        vec![
            move_to(0, 0),
            line_to(1, 1),
            move_to(SpatialScalarV2::MAX_RAW + 1, 0),
            line_to(2, 2),
        ],
    );

    expect_limit(
        validate(&fixture, limits(2, 0)),
        SpatialLimitKindV2::PathSubpathsTotal,
        SpatialErrorLocationV2::PathVerb {
            path: 0,
            verb: 0,
            field: SpatialPathVerbFieldV2::Kind,
        },
        1,
        0,
    );
}

#[test]
fn a_later_global_per_path_failure_precedes_an_earlier_subpath_crossing() {
    let fixture = fixture(
        vec![path(0, 0, 2), path(1, 2, 3)],
        vec![
            move_to(0, 0),
            line_to(1, 1),
            move_to(2, 2),
            line_to(3, 3),
            SpatialPathVerbV2::Close,
        ],
    );

    expect_limit(
        validate(&fixture, limits(2, 0)),
        SpatialLimitKindV2::PathVerbsPerPath,
        SpatialErrorLocationV2::Path {
            index: 1,
            field: SpatialPathFieldV2::VerbLength,
        },
        3,
        2,
    );
}

#[test]
fn path_structure_failure_precedes_k1_and_derived_limits() {
    let fixture = fixture(
        vec![path(0, 1, 1)],
        vec![move_to(SpatialScalarV2::MAX_RAW + 1, 0)],
    );

    expect_invalid_range(
        prepare_validated_paths!(
            &fixture,
            super::local_transform_support::VIEWPORT,
            limits(0, 0)
        ),
        SpatialErrorLocationV2::Path {
            index: 0,
            field: SpatialPathFieldV2::VerbStart,
        },
    );
}
