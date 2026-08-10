#[allow(dead_code, unused_imports)]
#[path = "support/layout_board/mod.rs"]
mod support;

use fenestra_ui_authoring::prototype::{
    CompiledAuthoringV1, FenSourceV1, REFERENCE_SEMANTIC_ARTIFACT_LIMITS_V1, SemanticArtifactV1,
    canonical_semantics_v1, compile_fen_v1, compile_ui_v1,
};
use proc_macro2::TokenStream;

const GOLDEN: &str = include_str!("artifacts/layout-board-semantic-v1.txt");
const RECORD_KINDS: [&str; 13] = [
    "document",
    "schema",
    "component",
    "property",
    "construction",
    "template",
    "initial-property",
    "static-child",
    "region-child",
    "region",
    "initial-key",
    "style",
    "style-assignment",
];

#[test]
fn both_frontends_match_the_committed_semantic_artifact_exactly() {
    let (fen, ui) = compile_both();
    let outputs = [observe(&fen), observe(&fen), observe(&ui), observe(&ui)];

    for output in outputs {
        assert_eq!(output.as_str(), GOLDEN);
        assert_eq!(output.as_bytes(), GOLDEN.as_bytes());
        assert_eq!(
            format!("{output:?}"),
            format!("SemanticArtifactV1 {{ bytes: {} }}", GOLDEN.len())
        );
    }
}

#[test]
fn committed_semantics_are_closed_bounded_ascii_records_in_source_order() {
    assert!(GOLDEN.is_ascii());
    assert!(!GOLDEN.contains('\r'));
    assert!(GOLDEN.ends_with('\n'));
    assert!(!GOLDEN.ends_with("\n\n"));
    assert!(GOLDEN.len() <= 8_192);
    assert!(GOLDEN.lines().all(|line| line.len() <= 512));

    let mut lines = GOLDEN.lines();
    assert_eq!(
        lines.next(),
        Some(
            "fenestra-authoring-semantics|1|authoring-format=1|schema-format=1|construction-format=1|style-format=1|records=34"
        )
    );
    let records = lines.collect::<Vec<_>>();
    assert_eq!(records.len(), 34);
    assert!(records.len() <= 64);
    for (ordinal, record) in records.iter().enumerate() {
        let fields = record.split('|').collect::<Vec<_>>();
        assert!(fields.len() >= 5);
        assert_eq!(fields[0], "record");
        assert_eq!(fields[1], ordinal.to_string());
        assert!(RECORD_KINDS.contains(&fields[2]));
        assert_eq!(fields[3], format!("span={ordinal}:{}", ordinal + 1));
    }

    for forbidden in [
        "/home/",
        "C:\\",
        "OUT_DIR",
        "PhysicalOrigin",
        "SourceMap",
        "ResolvedDocument",
    ] {
        assert!(!GOLDEN.contains(forbidden));
    }
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

fn observe(compiled: &CompiledAuthoringV1) -> SemanticArtifactV1 {
    canonical_semantics_v1(compiled, REFERENCE_SEMANTIC_ARTIFACT_LIMITS_V1)
        .expect("the registered semantic artifact should fit")
}
