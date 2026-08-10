#[path = "support/layout_board/mod.rs"]
mod support;

use fenestra_ui_authoring::prototype::{
    CompiledAuthoringV1, FenSourceV1, compile_fen_v1, compile_ui_v1,
};
use proc_macro2::TokenStream;

#[test]
fn ui_fixture_matches_the_exact_registered_ir_triple_and_fen_output() {
    let (fen, ui) = compile_both();

    assert_eq!(ui.schema(), &support::expected_schema());
    assert_eq!(ui.construction(), &support::expected_construction());
    assert_eq!(ui.style(), &support::expected_style());
    assert_eq!(ui.schema(), fen.schema());
    assert_eq!(ui.construction(), fen.construction());
    assert_eq!(ui.style(), fen.style());
    assert_eq!(
        ui.logical_source_catalog(),
        support::EXPECTED_LOGICAL_CATALOG
    );
    assert_eq!(ui.logical_source_catalog(), fen.logical_source_catalog());
}

#[test]
fn ui_fixture_matches_all_fen_logical_anchors_with_opaque_origins() {
    let (fen, ui) = compile_both();
    let fen_entries = fen.source_map().entries();
    let ui_entries = ui.source_map().entries();

    assert_eq!(fen_entries.len(), support::EXPECTED_ANCHORS.len());
    assert_eq!(ui_entries.len(), support::EXPECTED_ANCHORS.len());
    for (ordinal, ((fen_entry, ui_entry), expected)) in fen_entries
        .iter()
        .zip(ui_entries)
        .zip(support::EXPECTED_ANCHORS)
        .enumerate()
    {
        let logical = support::logical_span(ordinal as u32);
        assert_eq!(ui_entry.logical_span(), logical);
        assert_eq!(ui_entry.logical_span(), fen_entry.logical_span());
        assert_eq!(ui_entry.anchor_kind(), expected.kind);
        assert_eq!(ui_entry.anchor_kind(), fen_entry.anchor_kind());
        assert_eq!(ui_entry.canonical_label(), expected.label);
        assert_eq!(ui_entry.canonical_label(), fen_entry.canonical_label());
        assert_eq!(ui_entry.physical_origin().source_id(), None);
        assert_eq!(ui_entry.physical_origin().fen_byte_range(), None);
        assert_eq!(
            fen_entry.physical_origin().source_id(),
            Some(support::SOURCE)
        );
        assert_eq!(
            fen_entry.physical_origin().fen_byte_range(),
            Some((expected.start, expected.end))
        );
    }
}

fn compile_both() -> (CompiledAuthoringV1, CompiledAuthoringV1) {
    let fen = compile_fen_v1(
        FenSourceV1::new(support::SOURCE, support::FIXTURE),
        support::REGISTERED_LIMITS,
    )
    .expect("the registered FEN fixture should compile");
    let ui_source = str::from_utf8(support::FIXTURE).expect("the fixture should be UTF-8");
    let ui_tokens = ui_source
        .parse::<TokenStream>()
        .expect("the registered UI fixture should tokenize");
    let ui = compile_ui_v1(ui_tokens, support::REGISTERED_LIMITS)
        .expect("the registered UI fixture should compile");
    (fen, ui)
}
