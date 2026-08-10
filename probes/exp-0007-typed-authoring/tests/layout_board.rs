#[path = "support/layout_board/mod.rs"]
mod support;

use fenestra_ui_authoring::prototype::{
    AnchorKindV1, CompiledAuthoringV1, FenSourceV1, compile_fen_v1,
};

#[test]
fn fen_fixture_lowers_to_the_exact_registered_ir_triple_and_catalog() {
    let compiled = compile_fixture();

    assert_eq!(compiled.schema(), &support::expected_schema());
    assert_eq!(compiled.construction(), &support::expected_construction());
    assert_eq!(compiled.style(), &support::expected_style());
    assert_eq!(
        compiled.logical_source_catalog(),
        support::EXPECTED_LOGICAL_CATALOG
    );
}

#[test]
fn fen_fixture_maps_all_34_logical_anchors_to_exact_physical_tokens() {
    let compiled = compile_fixture();
    let entries = compiled.source_map().entries();

    assert_eq!(entries.len(), support::EXPECTED_ANCHORS.len());
    for (ordinal, (entry, expected)) in entries.iter().zip(support::EXPECTED_ANCHORS).enumerate() {
        assert_eq!(entry.logical_span(), support::logical_span(ordinal as u32));
        assert_eq!(entry.anchor_kind(), expected.kind);
        assert_eq!(entry.canonical_label(), expected.label);
        assert_eq!(entry.physical_origin().source_id(), Some(support::SOURCE));
        assert_eq!(
            entry.physical_origin().fen_byte_range(),
            Some((expected.start, expected.end))
        );
        assert_eq!(
            &support::FIXTURE[expected.start as usize..expected.end as usize],
            expected.label.as_bytes()
        );
    }

    assert_eq!(
        entries
            .iter()
            .map(|entry| entry.anchor_kind())
            .collect::<Vec<_>>(),
        expected_anchor_kinds()
    );
}

fn expected_anchor_kinds() -> Vec<AnchorKindV1> {
    support::EXPECTED_ANCHORS
        .iter()
        .map(|anchor| anchor.kind)
        .collect()
}

fn compile_fixture() -> CompiledAuthoringV1 {
    compile_fen_v1(
        FenSourceV1::new(support::SOURCE, support::FIXTURE),
        support::REGISTERED_LIMITS,
    )
    .expect("the registered format-1 fixture should compile")
}
