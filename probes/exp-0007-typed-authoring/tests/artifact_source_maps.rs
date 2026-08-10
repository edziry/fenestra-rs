#[path = "support/artifacts/mod.rs"]
mod artifacts;
#[allow(dead_code, unused_imports)]
#[path = "support/layout_board/mod.rs"]
mod support;

use artifacts::{
    MapArtifactLimitKindV1, MapArtifactLimitsV1, REGISTERED_MAP_ARTIFACT_LIMITS_V1,
    encode_fen_map_v1, encode_ui_map_v1,
};
use fenestra_ui_authoring::prototype::{
    CompiledAuthoringV1, FenSourceV1, compile_fen_v1, compile_ui_v1,
};
use proc_macro2::TokenStream;

const FEN_GOLDEN: &str = include_str!("artifacts/layout-board-fen-map-v1.txt");
const UI_GOLDEN: &str = include_str!("artifacts/layout-board-ui-map-v1.txt");

#[test]
fn committed_maps_match_both_compiled_frontends_exactly() {
    let (fen, ui) = compile_both();
    let fen_map = encode_fen_map_v1(&fen, REGISTERED_MAP_ARTIFACT_LIMITS_V1)
        .expect("the registered FEN map should encode");
    let ui_map = encode_ui_map_v1(&ui, REGISTERED_MAP_ARTIFACT_LIMITS_V1)
        .expect("the registered UI map should encode");

    assert_eq!(fen_map, FEN_GOLDEN);
    assert_eq!(ui_map, UI_GOLDEN);
    assert_artifact_bytes(FEN_GOLDEN);
    assert_artifact_bytes(UI_GOLDEN);
    assert_logical_rows_match(&fen_map, &ui_map);
}

#[test]
fn fen_rows_slice_all_tokens_and_ui_rows_never_serialize_origins() {
    let (fen, ui) = compile_both();
    assert_eq!(
        fen.logical_source_catalog(),
        support::EXPECTED_LOGICAL_CATALOG
    );
    assert_eq!(
        ui.logical_source_catalog(),
        support::EXPECTED_LOGICAL_CATALOG
    );
    for (ordinal, ((fen_entry, ui_entry), expected)) in fen
        .source_map()
        .entries()
        .iter()
        .zip(ui.source_map().entries())
        .zip(support::EXPECTED_ANCHORS)
        .enumerate()
    {
        let logical = support::logical_span(ordinal as u32);
        assert_eq!(fen_entry.logical_span(), logical);
        assert_eq!(ui_entry.logical_span(), logical);
        let range = fen_entry
            .physical_origin()
            .fen_byte_range()
            .expect("FEN entries should retain byte ranges");
        assert_eq!(range, (expected.start, expected.end));
        assert_eq!(
            &support::FIXTURE[range.0 as usize..range.1 as usize],
            expected.label.as_bytes()
        );
        assert_eq!(ui_entry.physical_origin().source_id(), None);
        assert_eq!(ui_entry.physical_origin().fen_byte_range(), None);
    }

    for row in FEN_GOLDEN
        .lines()
        .filter(|line| line.starts_with("anchor|"))
    {
        assert_eq!(row.split('|').count(), 9);
    }
    for row in UI_GOLDEN.lines().filter(|line| line.starts_with("anchor|")) {
        assert_eq!(row.split('|').count(), 6);
    }
    for forbidden in ["opaque", "Span", "SourceId", "/home/", "C:\\"] {
        assert!(!UI_GOLDEN.contains(forbidden));
    }
}

#[test]
fn map_bounds_are_inclusive_and_report_the_first_crossing() {
    assert_eq!(
        MapArtifactLimitKindV1::ALL,
        [
            MapArtifactLimitKindV1::Records,
            MapArtifactLimitKindV1::LineBytes,
            MapArtifactLimitKindV1::ArtifactBytes,
        ]
    );
    let (fen, _) = compile_both();
    let artifact_bytes = FEN_GOLDEN.len();
    let line_bytes = FEN_GOLDEN.lines().map(str::len).max().unwrap_or(0);
    let records = FEN_GOLDEN.lines().count();
    let exact = MapArtifactLimitsV1::new(artifact_bytes, line_bytes, records);
    assert_eq!(
        encode_fen_map_v1(&fen, exact).expect("exact map limits should be inclusive"),
        FEN_GOLDEN
    );

    let cases = [
        (
            MapArtifactLimitsV1::new(artifact_bytes - 1, line_bytes, records),
            MapArtifactLimitKindV1::ArtifactBytes,
        ),
        (
            MapArtifactLimitsV1::new(artifact_bytes, line_bytes - 1, records),
            MapArtifactLimitKindV1::LineBytes,
        ),
        (
            MapArtifactLimitsV1::new(artifact_bytes, line_bytes, records - 1),
            MapArtifactLimitKindV1::Records,
        ),
    ];
    for (limits, expected) in cases {
        let error = encode_fen_map_v1(&fen, limits).expect_err("one-under should fail");
        assert_eq!(error.limit_kind(), Some(expected));
    }

    let all_cross = encode_fen_map_v1(&fen, MapArtifactLimitsV1::new(0, 0, 0))
        .expect_err("simultaneous crossings should fail");
    assert_eq!(
        all_cross.limit_kind(),
        Some(MapArtifactLimitKindV1::Records)
    );
    let line_and_bytes_cross = encode_fen_map_v1(&fen, MapArtifactLimitsV1::new(0, 0, records))
        .expect_err("line crossing should precede artifact bytes");
    assert_eq!(
        line_and_bytes_cross.limit_kind(),
        Some(MapArtifactLimitKindV1::LineBytes)
    );
}

fn compile_both() -> (CompiledAuthoringV1, CompiledAuthoringV1) {
    let fen = compile_fen_v1(
        FenSourceV1::new(support::SOURCE, support::FIXTURE),
        support::REGISTERED_LIMITS,
    )
    .expect("the registered FEN fixture should compile");
    let source = str::from_utf8(support::FIXTURE).expect("the fixture should be UTF-8");
    let tokens = source
        .parse::<TokenStream>()
        .expect("the registered UI fixture should tokenize");
    let ui = compile_ui_v1(tokens, support::REGISTERED_LIMITS)
        .expect("the registered UI fixture should compile");
    (fen, ui)
}

fn assert_logical_rows_match(fen: &str, ui: &str) {
    let fen_rows = fen
        .lines()
        .skip(2)
        .map(|line| line.split('|').take(6).collect::<Vec<_>>().join("|"));
    assert_eq!(
        fen_rows.collect::<Vec<_>>(),
        ui.lines().skip(2).collect::<Vec<_>>()
    );
}

fn assert_artifact_bytes(value: &str) {
    assert!(value.is_ascii());
    assert!(!value.contains('\r'));
    assert!(value.ends_with('\n'));
    assert!(!value.ends_with("\n\n"));
    for forbidden in ["/home/", "C:\\", "OUT_DIR", "PhysicalOrigin", "SourceMapV1"] {
        assert!(!value.contains(forbidden));
    }
}
