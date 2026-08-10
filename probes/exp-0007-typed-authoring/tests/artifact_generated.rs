#[allow(dead_code)]
#[path = "support/layout_board/mod.rs"]
mod support;

use fenestra_ui_authoring::prototype::{
    CompiledAuthoringV1, FenSourceV1, canonical_rust_v1, compile_fen_v1, compile_ui_v1,
};
use fenestra_ui_exp_0007_typed_authoring::LAYOUT_BOARD_GENERATED_RUST_V1;
use fenestra_ui_ir::prototype::{ConstructionProgram, SchemaManifest, StyleProgram};
use proc_macro2::TokenStream;

const GOLDEN: &[u8] = include_bytes!("artifacts/layout-board-generated-v1.rs");

#[test]
fn committed_generated_rust_matches_both_frontends_and_out_dir_exactly() {
    let (fen, ui) = compile_both();
    let outputs = [
        canonical(&fen),
        canonical(&fen),
        canonical(&ui),
        canonical(&ui),
    ];

    for output in outputs {
        assert_eq!(output.as_bytes(), GOLDEN);
    }
    assert_eq!(LAYOUT_BOARD_GENERATED_RUST_V1.as_bytes(), GOLDEN);
    assert_canonical_bytes(GOLDEN);
}

#[test]
fn committed_generated_expression_constructs_the_manual_oracle() {
    let generated = golden_programs();

    assert_eq!(generated.0, support::expected_schema());
    assert_eq!(generated.1, support::expected_construction());
    assert_eq!(generated.2, support::expected_style());
    assert_eq!(support::EXPECTED_LOGICAL_CATALOG.len(), 34);
    assert_eq!(support::EXPECTED_ANCHORS.len(), 34);
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

fn canonical(compiled: &CompiledAuthoringV1) -> String {
    canonical_rust_v1(compiled, support::REGISTERED_LIMITS)
        .expect("the registered generated output should fit")
        .as_str()
        .to_owned()
}

fn golden_programs() -> (SchemaManifest, ConstructionProgram, StyleProgram) {
    include!("artifacts/layout-board-generated-v1.rs")
}

fn assert_canonical_bytes(bytes: &[u8]) {
    assert!(bytes.is_ascii());
    assert!(!bytes.contains(&b'\r'));
    assert_eq!(bytes.last(), Some(&b'\n'));
    assert_ne!(bytes.get(bytes.len().saturating_sub(2)), Some(&b'\n'));
    assert!(bytes.len() <= 32_768);

    let text = str::from_utf8(bytes).expect("ASCII generated Rust should be UTF-8");
    for forbidden in [
        "/home/",
        "C:\\",
        "OUT_DIR",
        "fenestra_ui_authoring",
        "SourceMap",
    ] {
        assert!(
            !text.contains(forbidden),
            "generated Rust leaked {forbidden}"
        );
    }
}
