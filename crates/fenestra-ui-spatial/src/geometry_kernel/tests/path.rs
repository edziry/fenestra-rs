use super::*;

#[test]
fn every_path_scalar_field_is_preflighted_before_grammar() {
    let low = SpatialScalarV2::MIN_RAW - 1;
    let high = SpatialScalarV2::MAX_RAW + 1;
    let cases = [
        (
            SpatialPathVerbV2::MoveTo { to: point(low, 0) },
            GeometryK1Field::ToX,
        ),
        (
            SpatialPathVerbV2::MoveTo { to: point(0, high) },
            GeometryK1Field::ToY,
        ),
        (
            SpatialPathVerbV2::LineTo { to: point(high, 0) },
            GeometryK1Field::ToX,
        ),
        (
            SpatialPathVerbV2::LineTo { to: point(0, low) },
            GeometryK1Field::ToY,
        ),
        (
            SpatialPathVerbV2::QuadraticTo {
                control: point(low, 0),
                to: point(0, 0),
            },
            GeometryK1Field::ControlX,
        ),
        (
            SpatialPathVerbV2::QuadraticTo {
                control: point(0, high),
                to: point(0, 0),
            },
            GeometryK1Field::ControlY,
        ),
        (
            SpatialPathVerbV2::QuadraticTo {
                control: point(0, 0),
                to: point(low, 0),
            },
            GeometryK1Field::ToX,
        ),
        (
            SpatialPathVerbV2::QuadraticTo {
                control: point(0, 0),
                to: point(0, high),
            },
            GeometryK1Field::ToY,
        ),
        (
            SpatialPathVerbV2::CubicTo {
                control1: point(low, 0),
                control2: point(0, 0),
                to: point(0, 0),
            },
            GeometryK1Field::Control1X,
        ),
        (
            SpatialPathVerbV2::CubicTo {
                control1: point(0, high),
                control2: point(0, 0),
                to: point(0, 0),
            },
            GeometryK1Field::Control1Y,
        ),
        (
            SpatialPathVerbV2::CubicTo {
                control1: point(0, 0),
                control2: point(low, 0),
                to: point(0, 0),
            },
            GeometryK1Field::Control2X,
        ),
        (
            SpatialPathVerbV2::CubicTo {
                control1: point(0, 0),
                control2: point(0, high),
                to: point(0, 0),
            },
            GeometryK1Field::Control2Y,
        ),
        (
            SpatialPathVerbV2::CubicTo {
                control1: point(0, 0),
                control2: point(0, 0),
                to: point(low, 0),
            },
            GeometryK1Field::ToX,
        ),
        (
            SpatialPathVerbV2::CubicTo {
                control1: point(0, 0),
                control2: point(0, 0),
                to: point(0, high),
            },
            GeometryK1Field::ToY,
        ),
    ];

    for (verb, field) in cases {
        expect_error(
            validate_path_k1(PATH_INDEX, &[verb], 0, 8),
            GeometryK1ErrorKind::ScalarOutOfDomain,
            path_location(0, field),
        );
    }
}

#[test]
fn complete_scalar_pass_wins_over_an_earlier_grammar_fault() {
    let verbs = [line_to(1, 1), move_to(SpatialScalarV2::MAX_RAW + 1, 0)];

    expect_error(
        validate_path_k1(PATH_INDEX, &verbs, 0, 8),
        GeometryK1ErrorKind::ScalarOutOfDomain,
        path_location(1, GeometryK1Field::ToX),
    );
}

#[test]
fn path_scalar_priority_is_verb_then_applicable_field() {
    let invalid = SpatialScalarV2::MAX_RAW + 1;
    let verbs = [
        move_to(0, invalid),
        SpatialPathVerbV2::CubicTo {
            control1: point(invalid, invalid),
            control2: point(invalid, invalid),
            to: point(invalid, invalid),
        },
    ];

    expect_error(
        validate_path_k1(PATH_INDEX, &verbs, 0, 8),
        GeometryK1ErrorKind::ScalarOutOfDomain,
        path_location(0, GeometryK1Field::ToY),
    );
}

#[test]
fn grammar_errors_have_exact_locations() {
    let cases = [
        (
            Vec::new(),
            GeometryK1PathGrammarKind::Empty,
            GeometryK1Location::Path {
                index: PATH_INDEX,
                field: GeometryK1Field::VerbLength,
            },
        ),
        (
            vec![line_to(1, 1)],
            GeometryK1PathGrammarKind::FirstNotMove,
            path_location(0, GeometryK1Field::Kind),
        ),
        (
            vec![move_to(0, 0), move_to(1, 1), line_to(2, 2)],
            GeometryK1PathGrammarKind::EmptySubpath,
            path_location(1, GeometryK1Field::Kind),
        ),
        (
            vec![
                move_to(0, 0),
                line_to(1, 1),
                SpatialPathVerbV2::Close,
                line_to(2, 2),
            ],
            GeometryK1PathGrammarKind::DrawingWithoutSubpath,
            path_location(3, GeometryK1Field::Kind),
        ),
        (
            vec![move_to(0, 0), SpatialPathVerbV2::Close],
            GeometryK1PathGrammarKind::CloseWithoutSegment,
            path_location(1, GeometryK1Field::Kind),
        ),
        (
            vec![move_to(0, 0)],
            GeometryK1PathGrammarKind::TrailingMove,
            path_location(0, GeometryK1Field::Kind),
        ),
    ];

    for (verbs, grammar, location) in cases {
        expect_error(
            validate_path_k1(PATH_INDEX, &verbs, 0, 8),
            GeometryK1ErrorKind::InvalidPathGrammar(grammar),
            location,
        );
    }
}

#[test]
fn first_verb_priority_precedes_later_grammar_rules() {
    expect_error(
        validate_path_k1(PATH_INDEX, &[SpatialPathVerbV2::Close], 0, 8),
        GeometryK1ErrorKind::InvalidPathGrammar(GeometryK1PathGrammarKind::FirstNotMove),
        path_location(0, GeometryK1Field::Kind),
    );
}

#[test]
fn a_second_close_reports_close_without_segment_at_its_own_verb() {
    let verbs = [
        move_to(0, 0),
        line_to(1, 1),
        SpatialPathVerbV2::Close,
        SpatialPathVerbV2::Close,
    ];
    expect_error(
        validate_path_k1(PATH_INDEX, &verbs, 0, 8),
        GeometryK1ErrorKind::InvalidPathGrammar(GeometryK1PathGrammarKind::CloseWithoutSegment),
        path_location(3, GeometryK1Field::Kind),
    );
}

#[test]
fn full_grammar_wins_over_a_subpath_crossing_in_its_valid_prefix() {
    let verbs = [move_to(0, 0), line_to(1, 1), move_to(2, 2)];

    expect_error(
        validate_path_k1(PATH_INDEX, &verbs, 1, 1),
        GeometryK1ErrorKind::InvalidPathGrammar(GeometryK1PathGrammarKind::TrailingMove),
        path_location(2, GeometryK1Field::Kind),
    );
}

#[test]
fn subpath_limit_accepts_the_edge_and_names_the_crossing_move() {
    let one = [move_to(0, 0), line_to(1, 1)];
    let proof: ValidatedPathK1<'_> = expect_valid(validate_path_k1(PATH_INDEX, &one, 1, 2));
    assert_eq!(proof.subpath_count(), 1);

    let two = [move_to(0, 0), line_to(1, 1), move_to(2, 2), line_to(3, 3)];
    expect_limit(
        validate_path_k1(PATH_INDEX, &two, 1, 2),
        GeometryK1LimitKind::PathSubpathsTotal,
        path_location(2, GeometryK1Field::Kind),
        3,
        2,
    );
}

#[test]
fn registered_subpath_total_accepts_1024_and_rejects_1025() {
    assert_eq!(PATH_SUBPATH_MAXIMUM, 1_024);
    let one = [move_to(0, 0), line_to(1, 1)];

    let proof: ValidatedPathK1<'_> = expect_valid(validate_path_k1(
        PATH_INDEX,
        &one,
        PATH_SUBPATH_MAXIMUM - 1,
        PATH_SUBPATH_MAXIMUM,
    ));
    assert_eq!(proof.subpath_count(), 1);

    expect_limit(
        validate_path_k1(PATH_INDEX, &one, PATH_SUBPATH_MAXIMUM, PATH_SUBPATH_MAXIMUM),
        GeometryK1LimitKind::PathSubpathsTotal,
        path_location(0, GeometryK1Field::Kind),
        PATH_SUBPATH_MAXIMUM + 1,
        PATH_SUBPATH_MAXIMUM,
    );
}

#[test]
fn zero_length_line_is_valid_geometry() {
    let verbs = [move_to(4, 5), line_to(4, 5)];
    let proof: ValidatedPathK1<'_> = expect_valid(validate_path_k1(PATH_INDEX, &verbs, 0, 1));
    assert_eq!(proof.verbs(), verbs.as_slice());
    assert_eq!(proof.subpath_count(), 1);
}

#[test]
fn valid_path_proof_retains_verbs_and_count_without_mutating_the_caller() {
    let accepted_subpaths = 5;
    let verbs = [
        move_to(SpatialScalarV2::MIN_RAW, SpatialScalarV2::MAX_RAW),
        line_to(0, 0),
        SpatialPathVerbV2::Close,
        move_to(2, 2),
        SpatialPathVerbV2::QuadraticTo {
            control: point(3, 4),
            to: point(5, 6),
        },
    ];

    let proof: ValidatedPathK1<'_> =
        expect_valid(validate_path_k1(PATH_INDEX, &verbs, accepted_subpaths, 7));
    assert_eq!(proof.verbs(), verbs.as_slice());
    assert!(std::ptr::eq(proof.verbs(), verbs.as_slice()));
    assert_eq!(proof.subpath_count(), 2);
    assert_eq!(accepted_subpaths, 5);
}
