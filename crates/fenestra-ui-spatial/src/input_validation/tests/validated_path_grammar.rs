use super::validated_path_support::{
    expect_content, fixture, line_to, move_to, path, permissive_limits, validate,
};
use crate::content_diagnostic::SpatialPathGrammarErrorV2;
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::{SpatialPathFieldV2, SpatialPathVerbFieldV2};
use crate::path::SpatialPathVerbV2;

#[test]
fn every_k1_path_grammar_failure_maps_its_exact_local_location() {
    let cases = [
        (
            Vec::new(),
            SpatialPathGrammarErrorV2::Empty,
            SpatialErrorLocationV2::Path {
                index: 1,
                field: SpatialPathFieldV2::VerbLength,
            },
        ),
        (
            vec![line_to(0, 0)],
            SpatialPathGrammarErrorV2::FirstNotMove,
            verb_location(0),
        ),
        (
            vec![move_to(0, 0), move_to(1, 1), line_to(2, 2)],
            SpatialPathGrammarErrorV2::EmptySubpath,
            verb_location(1),
        ),
        (
            vec![
                move_to(0, 0),
                line_to(1, 1),
                SpatialPathVerbV2::Close,
                line_to(2, 2),
            ],
            SpatialPathGrammarErrorV2::DrawingWithoutSubpath,
            verb_location(3),
        ),
        (
            vec![move_to(0, 0), SpatialPathVerbV2::Close],
            SpatialPathGrammarErrorV2::CloseWithoutSegment,
            verb_location(1),
        ),
        (
            vec![move_to(0, 0)],
            SpatialPathGrammarErrorV2::TrailingMove,
            verb_location(0),
        ),
    ];

    for (invalid, grammar, location) in cases {
        let prefix = vec![move_to(7, 7), line_to(8, 8)];
        let mut verbs = prefix;
        verbs.extend_from_slice(&invalid);
        let fixture = fixture(vec![path(0, 0, 2), path(1, 2, invalid.len() as u32)], verbs);

        expect_content(
            validate(&fixture, permissive_limits()),
            SpatialContentErrorKindV2::InvalidPathGrammar(grammar),
            location,
        );
    }
}

fn verb_location(verb: u32) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::PathVerb {
        path: 1,
        verb,
        field: SpatialPathVerbFieldV2::Kind,
    }
}
