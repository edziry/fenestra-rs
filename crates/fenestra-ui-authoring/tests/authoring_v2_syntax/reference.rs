use fenestra_ui_authoring::prototype::{
    AnchorKindV2, AuthoringLimitsV1, FenSourceV1, compile_fen_v1, compile_ui_v2,
};
use fenestra_ui_ir::prototype::{SourceId, SourceSpan};
use proc_macro2::TokenStream;

use super::support::{FIXTURE, GENEROUS_LIMITS, SOURCE, compile, nth_range, replace_once};

#[test]
fn reference_fixture_measurements_and_both_frontends_are_exact() {
    assert_eq!(FIXTURE.len(), 7_714);
    assert!(FIXTURE.is_ascii());
    assert!(FIXTURE.ends_with('\n'));

    let tokens = FIXTURE
        .parse::<TokenStream>()
        .expect("the token-equivalent UI fixture should tokenize");
    assert_eq!(lexical_measurements(FIXTURE), (1_610, 15, 8));

    let fen = compile(FIXTURE);
    let ui = compile_ui_v2(tokens, GENEROUS_LIMITS)
        .expect("the token-equivalent UI fixture should compile");
    assert_eq!(fen.logical_source_catalog(), &[b'@'; 380]);
    assert_eq!(ui.logical_source_catalog(), fen.logical_source_catalog());
    assert_eq!(fen.source_map().entries().len(), 380);
    assert_eq!(ui.source_map().entries().len(), 380);
    assert_eq!(fen.schema(), ui.schema());
    assert_eq!(fen.construction(), ui.construction());
    assert_eq!(fen.style(), ui.style());
    assert_eq!(fen.spatial(), ui.spatial());
    assert_eq!(fen.spatial().nodes().len(), 7);
    assert_eq!(fen.spatial().images().len(), 1);
    assert_eq!(fen.spatial().images()[0].bytes().len(), 16);

    for (ordinal, (fen_entry, ui_entry)) in fen
        .source_map()
        .entries()
        .iter()
        .zip(ui.source_map().entries())
        .enumerate()
    {
        let ordinal = u32::try_from(ordinal).expect("fixture ordinal should fit");
        let expected = SourceSpan::bytes(SourceId::new(0), ordinal, ordinal + 1);
        assert_eq!(fen_entry.logical_span(), expected);
        assert_eq!(ui_entry.logical_span(), expected);
        assert_eq!(fen_entry.anchor_kind(), ui_entry.anchor_kind());
        assert_eq!(fen_entry.canonical_label(), ui_entry.canonical_label());
        assert_eq!(fen_entry.physical_origin().source_id(), Some(SOURCE));
        assert_eq!(ui_entry.physical_origin().source_id(), None);
        assert_eq!(ui_entry.physical_origin().fen_byte_range(), None);
    }
}

fn lexical_measurements(source: &str) -> (usize, usize, usize) {
    let bytes = source.as_bytes();
    let mut index = 0;
    let mut tokens = 0;
    let mut longest_identifier = 0;
    let mut depth = 0;
    let mut maximum_depth = 0;
    while index < bytes.len() {
        if bytes[index].is_ascii_whitespace() {
            index += 1;
            continue;
        }
        tokens += 1;
        if bytes[index].is_ascii_alphabetic() || bytes[index] == b'_' {
            let start = index;
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            longest_identifier = longest_identifier.max(index - start);
            continue;
        }
        if bytes[index].is_ascii_digit() {
            index += 1;
            while index < bytes.len() && bytes[index].is_ascii_digit() {
                index += 1;
            }
            continue;
        }
        match bytes[index] {
            b'(' | b'[' | b'{' => {
                depth += 1;
                maximum_depth = maximum_depth.max(depth);
            }
            b')' | b']' | b'}' => depth -= 1,
            _ => {}
        }
        index += 1;
    }
    assert_eq!(depth, 0);
    (tokens, longest_identifier, maximum_depth)
}

#[test]
fn reference_source_map_preserves_records_fields_and_shared_origins() {
    let compiled = compile(FIXTURE);
    let entries = compiled.source_map().entries();
    let cases = [
        (0, AnchorKindV2::Document, "format", "format", 0),
        (51, AnchorKindV2::Spatial, "spatial", "spatial", 0),
        (
            52,
            AnchorKindV2::SpatialContainer,
            "container",
            "container",
            0,
        ),
        (58, AnchorKindV2::Resources, "resources", "resources", 0),
        (59, AnchorKindV2::Image, "checker", "checker", 0),
        (60, AnchorKindV2::SpatialField, "checker", "checker", 0),
        (64, AnchorKindV2::SpatialNode, "scene", "scene", 0),
        (65, AnchorKindV2::SpatialField, "scene", "scene", 0),
        (89, AnchorKindV2::SpatialShape, "frame", "frame", 0),
        (132, AnchorKindV2::SpatialBrush, "flat", "flat", 0),
        (150, AnchorKindV2::SpatialClip, "outer", "outer", 0),
        (330, AnchorKindV2::SpatialNode, "guide", "guide", 1),
        (331, AnchorKindV2::SpatialField, "guide", "guide", 1),
        (332, AnchorKindV2::SpatialField, "guide", "guide", 1),
        (
            356,
            AnchorKindV2::SpatialNode,
            "viewport_layer",
            "viewport_layer",
            0,
        ),
        (
            357,
            AnchorKindV2::SpatialField,
            "viewport_layer",
            "viewport_layer",
            0,
        ),
        (
            358,
            AnchorKindV2::SpatialField,
            "viewport_layer",
            "viewport_layer",
            0,
        ),
        (377, AnchorKindV2::SpatialField, "-32768", "-32768", 1),
        (379, AnchorKindV2::SpatialField, "0", "0", 116),
    ];

    for (ordinal, kind, label, culprit, occurrence) in cases {
        let entry = &entries[ordinal];
        assert_eq!(entry.anchor_kind(), kind, "ordinal {ordinal}");
        assert_eq!(entry.canonical_label(), label, "ordinal {ordinal}");
        assert_eq!(
            entry.physical_origin().fen_byte_range(),
            Some(nth_range(FIXTURE, culprit, occurrence)),
            "ordinal {ordinal}"
        );
    }

    let guide_range = entries[330].physical_origin().fen_byte_range();
    assert_eq!(entries[331].physical_origin().fen_byte_range(), guide_range);
    assert_eq!(entries[332].physical_origin().fen_byte_range(), guide_range);

    let mut counts = [0; 30];
    for entry in entries {
        let index = AnchorKindV2::ALL
            .iter()
            .position(|kind| *kind == entry.anchor_kind())
            .expect("every source-map kind must belong to the closed vocabulary");
        counts[index] += 1;
        assert!(!entry.canonical_label().is_empty());
        assert!(entry.canonical_label().is_ascii());
        let (start, end) = entry
            .physical_origin()
            .fen_byte_range()
            .expect("every FEN map entry should carry a byte range");
        assert!(start < end);
        assert!(usize::try_from(end).expect("range should fit") <= FIXTURE.len());
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
fn format_two_words_remain_valid_format_one_names() {
    let v1 = "format 1;
schema namespace 1 revision 1 {
  component spatial = 0 {
    property viewport = 0: scalar_i32 = 0 invalidates [layout];
  }
}
construction {
  template node = 0: spatial { child region resources; }
  template viewport = 1: spatial {}
  region resources = 0 owner node repeat viewport keys [] invalidates [structure];
}
style {}
";
    let limits = AuthoringLimitsV1::new(4_096, 512, 32, 8, 1, 1, 2, 1, 1, 0, 0, 0, 10, 8_192);
    compile_fen_v1(FenSourceV1::new(SourceId::new(1), v1.as_bytes()), limits)
        .expect("format-2 keywords must not alter the format-1 reserved set");
}

#[test]
fn every_new_format_two_keyword_is_reserved_as_a_name() {
    let words = [
        "spatial",
        "viewport",
        "container",
        "row",
        "column",
        "padding",
        "gap",
        "resources",
        "image",
        "width",
        "height",
        "stride",
        "bytes",
        "node",
        "placement",
        "free",
        "dimension",
        "self_anchor",
        "anchor",
        "start",
        "center",
        "end",
        "target",
        "parent",
        "target_anchor",
        "offset",
        "transform",
        "identity",
        "translate",
        "scale",
        "quarter_turn",
        "affine",
        "origin",
        "point",
        "fixed",
        "shape",
        "rect",
        "circle",
        "polygon",
        "path",
        "radius",
        "move_to",
        "line_to",
        "quadratic_to",
        "cubic_to",
        "close",
        "brush",
        "solid",
        "color",
        "linear_gradient",
        "stop",
        "clip",
        "none",
        "fill_rule",
        "non_zero",
        "even_odd",
        "coverage",
        "fill",
        "rule",
        "round_stroke",
        "opacity",
        "source",
        "destination",
        "hit",
        "input",
        "semantic",
    ];
    for word in words {
        let source = replace_once(FIXTURE, "component fixture", &format!("component {word}"));
        let error = fenestra_ui_authoring::prototype::compile_fen_v2(
            fenestra_ui_authoring::prototype::FenSourceV2::new(SOURCE, source.as_bytes()),
            GENEROUS_LIMITS,
        )
        .expect_err("each new grammar keyword must be reserved in format 2");
        assert_eq!(
            error.kind(),
            fenestra_ui_authoring::prototype::AuthoringDiagnosticKindV2::InvalidIdentifier,
            "word {word}"
        );
    }
}
