#[path = "support/layout_board/mod.rs"]
mod support;

use fenestra_ui_authoring::prototype::{CompiledAuthoringV1, FenSourceV1, compile_fen_v1};
use fenestra_ui_ir::prototype::{ConstructionProgram, SchemaManifest, StyleProgram};
use fenestra_ui_macros::ui;

#[test]
fn ui_macro_fixture_matches_the_independent_oracle_and_fen_compilation() {
    let generated = macro_fixture();
    let fen = compile_fixture();

    assert_eq!(generated.0, support::expected_schema());
    assert_eq!(generated.1, support::expected_construction());
    assert_eq!(generated.2, support::expected_style());
    assert_eq!(generated.0, *fen.schema());
    assert_eq!(generated.1, *fen.construction());
    assert_eq!(generated.2, *fen.style());
    assert_eq!(
        support::EXPECTED_LOGICAL_CATALOG.len(),
        support::EXPECTED_ANCHORS.len()
    );
    for anchor in support::EXPECTED_ANCHORS {
        assert!(fenestra_ui_authoring::prototype::AnchorKindV1::ALL.contains(&anchor.kind));
        assert_eq!(
            &support::FIXTURE[anchor.start as usize..anchor.end as usize],
            anchor.label.as_bytes()
        );
    }

    let repeated = macro_fixture();
    assert_eq!(repeated.0, generated.0);
    assert_eq!(repeated.1, generated.1);
    assert_eq!(repeated.2, generated.2);
}

fn macro_fixture() -> (SchemaManifest, ConstructionProgram, StyleProgram) {
    include!("../fixtures/layout-board.ui")
}

fn compile_fixture() -> CompiledAuthoringV1 {
    compile_fen_v1(
        FenSourceV1::new(support::SOURCE, support::FIXTURE),
        support::REGISTERED_LIMITS,
    )
    .expect("the registered FEN fixture should compile")
}
