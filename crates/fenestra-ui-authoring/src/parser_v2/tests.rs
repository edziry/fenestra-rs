use fenestra_ui_ir::prototype::SourceId;

use crate::fen_v2::parse_fen_document_v2;
use crate::limits_v2::AuthoringLimitsV2;
use crate::ui_v2::parse_ui_document_v2;
use crate::vocabulary_v2::{AnchorKindV2, AuthoringFrontendV2};

const FIXTURE: &str = include_str!("../../tests/fixtures/hybrid_spatial_v2.fen");

#[test]
fn reference_frontends_parse_the_exact_anchor_and_field_counts() {
    let fen = parse_fen_document_v2(SourceId::new(13), FIXTURE, limits())
        .expect("reference FEN should parse");
    let ui = parse_ui_document_v2(FIXTURE.parse().expect("fixture should tokenize"), limits())
        .expect("reference UI stream should parse");

    assert_eq!(fen.frontend, AuthoringFrontendV2::Fen);
    assert_eq!(ui.frontend, AuthoringFrontendV2::UiMacro);
    assert_eq!(fen.spatial.field_count, 264);
    assert_eq!(ui.spatial.field_count, 264);
    assert_eq!(fen.anchors.len(), 380);
    assert_eq!(ui.anchors.len(), 380);
    for (fen, ui) in fen.anchors.iter().zip(&ui.anchors) {
        assert_eq!(fen.kind, ui.kind);
        assert_eq!(fen.label, ui.label);
    }
}

#[test]
fn reference_anchor_vocabulary_counts_are_exact() {
    let parsed = parse_fen_document_v2(SourceId::new(13), FIXTURE, limits())
        .expect("reference FEN should parse");
    let mut counts = [0; 30];
    for anchor in parsed.anchors {
        let index = AnchorKindV2::ALL
            .iter()
            .position(|kind| *kind == anchor.kind)
            .expect("closed anchor vocabulary");
        counts[index] += 1;
    }
    assert_eq!(
        counts,
        [
            1, 1, 1, 8, 1, 7, 19, 5, 1, 1, 2, 1, 3, 1, 1, 1, 7, 8, 7, 7, 264, 5, 5, 3, 3, 3, 3, 4,
            4, 3,
        ]
    );
}

#[test]
fn reference_key_origins_and_shared_node_origins_are_exact() {
    let parsed = parse_fen_document_v2(SourceId::new(13), FIXTURE, limits())
        .expect("reference FEN should parse");
    let cases = [
        (0, AnchorKindV2::Document, "format", 0),
        (51, AnchorKindV2::Spatial, "spatial", 0),
        (52, AnchorKindV2::SpatialContainer, "container", 0),
        (58, AnchorKindV2::Resources, "resources", 0),
        (59, AnchorKindV2::Image, "checker", 0),
        (60, AnchorKindV2::SpatialField, "checker", 0),
        (64, AnchorKindV2::SpatialNode, "scene", 0),
        (65, AnchorKindV2::SpatialField, "scene", 0),
        (330, AnchorKindV2::SpatialNode, "guide", 1),
        (331, AnchorKindV2::SpatialField, "guide", 1),
        (332, AnchorKindV2::SpatialField, "guide", 1),
        (377, AnchorKindV2::SpatialField, "-32768", 1),
    ];
    for (ordinal, kind, culprit, occurrence) in cases {
        let anchor = &parsed.anchors[ordinal];
        assert_eq!(anchor.kind, kind, "ordinal {ordinal}");
        let expected = nth_range(FIXTURE, culprit, occurrence);
        assert_eq!(anchor.physical.fen_byte_range(), Some(expected));
    }
    assert_eq!(
        parsed.anchors[330].physical.fen_byte_range(),
        parsed.anchors[331].physical.fen_byte_range()
    );
    assert_eq!(
        parsed.anchors[330].physical.fen_byte_range(),
        parsed.anchors[332].physical.fen_byte_range()
    );
}

fn limits() -> AuthoringLimitsV2 {
    AuthoringLimitsV2::new([
        8_192,
        2_048,
        15,
        12,
        1,
        8,
        7,
        1,
        6,
        19,
        2,
        3,
        1,
        16,
        7,
        264,
        5,
        1,
        5,
        3,
        3,
        3,
        3,
        4,
        4,
        3,
        512,
        usize::MAX,
    ])
}

fn nth_range(source: &str, needle: &str, occurrence: usize) -> (u32, u32) {
    let start = source
        .match_indices(needle)
        .nth(occurrence)
        .expect("fixture culprit should exist")
        .0;
    (
        u32::try_from(start).expect("fixture start should fit"),
        u32::try_from(start + needle.len()).expect("fixture end should fit"),
    )
}
