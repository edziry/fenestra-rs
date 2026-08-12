#[path = "support/hybrid_spatial_v2/mod.rs"]
mod support;

use fenestra_ui_authoring::prototype::{
    FenSourceV2, REFERENCE_AUTHORING_LIMITS_V2, canonical_rust_v2, compile_fen_v2,
};
use fenestra_ui_exp_0007_typed_authoring::{
    HYBRID_SPATIAL_FEN_V2, HYBRID_SPATIAL_GENERATED_RUST_V2,
    generated_hybrid_spatial_v2, macro_hybrid_spatial_v2,
};
use fenestra_ui_ir::prototype::{
    ConstructionProgram, SchemaManifest, SourceId, SpatialProgramV2,
    SpatialValidationLimitsV2, StyleProgram, StyleValidationLimits, ValidatedSpatialProgramV2,
    ValidationLimits, validate_construction, validate_schema, validate_spatial, validate_style,
};
use proc_macro2::{TokenStream, TokenTree};

type RawProgramsV2 = (
    SchemaManifest,
    ConstructionProgram,
    StyleProgram,
    SpatialProgramV2,
);

const IR_LIMITS: ValidationLimits = ValidationLimits::new(1, 8, 7, 1, 6, 19, 2, 4, 8);
const STYLE_LIMITS: StyleValidationLimits = StyleValidationLimits::new(3);
const SPATIAL_LIMITS: SpatialValidationLimitsV2 =
    SpatialValidationLimitsV2::new([7, 5, 3, 3, 4, 4, 3, 1, 5, 3, 3, 1, 16]);
const FEN_SOURCE: SourceId = SourceId::new(13);
const UI_FIXTURE: &str = include_str!("../fixtures/hybrid-spatial-v2.ui");

#[test]
fn registered_format_two_fixtures_and_generated_rust_are_exact() {
    assert_eq!(HYBRID_SPATIAL_FEN_V2.len(), 7_714);
    assert_eq!(fnv1a64(HYBRID_SPATIAL_FEN_V2), 0x4a6f_15b1_b7bf_b015);
    assert_eq!(HYBRID_SPATIAL_FEN_V2.last(), Some(&b'\n'));
    assert!(!HYBRID_SPATIAL_FEN_V2.contains(&b'\r'));
    assert!(HYBRID_SPATIAL_FEN_V2.iter().all(u8::is_ascii));

    let fen_tokens = std::str::from_utf8(HYBRID_SPATIAL_FEN_V2)
        .expect("the registered FEN fixture should be UTF-8")
        .parse::<TokenStream>()
        .expect("the registered FEN fixture should tokenize");
    assert_eq!(macro_document_tokens(UI_FIXTURE), fen_tokens.to_string());

    let compiled = compile_fen_v2(
        FenSourceV2::new(FEN_SOURCE, HYBRID_SPATIAL_FEN_V2),
        REFERENCE_AUTHORING_LIMITS_V2,
    )
    .expect("the registered format-two fixture should compile");
    let fresh = canonical_rust_v2(&compiled, REFERENCE_AUTHORING_LIMITS_V2)
        .expect("the registered canonical Rust should fit its exact bound");
    assert_eq!(HYBRID_SPATIAL_GENERATED_RUST_V2, fresh.as_str());
    assert_eq!(HYBRID_SPATIAL_GENERATED_RUST_V2.len(), 107_789);
    assert!(HYBRID_SPATIAL_GENERATED_RUST_V2.is_ascii());
    assert!(HYBRID_SPATIAL_GENERATED_RUST_V2.ends_with('\n'));
    assert!(!HYBRID_SPATIAL_GENERATED_RUST_V2.contains('\r'));
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[test]
fn manual_fen_and_ui_lanes_produce_equal_raw_programs_and_validate_separately() {
    let manual = support::manual_hybrid_spatial_v2();
    let fen = generated_hybrid_spatial_v2();
    let ui = macro_hybrid_spatial_v2();

    assert_raw_eq(&manual, &fen);
    assert_raw_eq(&manual, &ui);
    assert_raw_eq(&fen, &ui);

    let manual = validate(manual);
    let fen = validate(fen);
    let ui = validate(ui);
    assert_eq!(manual.program(), fen.program());
    assert_eq!(manual.program(), ui.program());
    assert!(!manual.style().shares_domain_with(fen.style()));
    assert!(!manual.style().shares_domain_with(ui.style()));
    assert!(!fen.style().shares_domain_with(ui.style()));
    assert!(!manual.shares_domain_with(&fen));
    assert!(!manual.shares_domain_with(&ui));
    assert!(!fen.shares_domain_with(&ui));
}

fn macro_document_tokens(source: &str) -> String {
    let mut tokens = source
        .parse::<TokenStream>()
        .expect("the registered ui! fixture should tokenize")
        .into_iter();
    assert!(matches!(tokens.next(), Some(TokenTree::Ident(name)) if name == "ui"));
    assert!(matches!(tokens.next(), Some(TokenTree::Punct(mark)) if mark.as_char() == '!'));
    let Some(TokenTree::Group(document)) = tokens.next() else {
        panic!("the registered ui! fixture should contain one document group");
    };
    assert!(tokens.next().is_none());
    document.stream().to_string()
}

fn assert_raw_eq(left: &RawProgramsV2, right: &RawProgramsV2) {
    assert_eq!(left.0, right.0);
    assert_eq!(left.1, right.1);
    assert_eq!(left.2, right.2);
    assert_eq!(left.3, right.3);
}

fn validate(programs: RawProgramsV2) -> ValidatedSpatialProgramV2 {
    let schema = validate_schema(programs.0, IR_LIMITS)
        .expect("the registered schema should validate independently");
    let construction = validate_construction(&schema, programs.1, IR_LIMITS)
        .expect("the registered construction should validate independently");
    let style = validate_style(&construction, programs.2, STYLE_LIMITS)
        .expect("the registered style should validate independently");
    validate_spatial(&style, programs.3, SPATIAL_LIMITS)
        .expect("the registered spatial program should validate independently")
}
