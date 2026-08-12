#![allow(dead_code)]

use fenestra_ui_authoring::prototype::{
    AuthoringDiagnosticV2, AuthoringLimitKindV2, AuthoringLimitsV2, CompiledAuthoringV2,
    FenSourceV2, SemanticArtifactLimitsV2, compile_fen_v2, compile_ui_v2,
};
use fenestra_ui_ir::prototype::SourceId;
use proc_macro2::TokenStream;

pub const SOURCE_ID: SourceId = SourceId::new(2_013);
pub const FIXTURE: &str = include_str!("../fixtures/hybrid_spatial_v2.fen");

pub const V1_DOCUMENT: &str = "format 1;
schema namespace 9 revision 1 {
  component panel = 7 {
    property visible = 0: bool = true invalidates [paint];
    property width = 1: scalar_i32 = -2 invalidates [layout, semantics];
    property color = 2: rgba8 = rgba8(1, 2, 3, 255) invalidates [paint];
    property input = 3: input_policy = ignore invalidates [hit_test];
  }
}
construction {
  template root = 10: panel {
    set visible = false;
    set width = 3;
    set color = rgba8(4, 5, 6, 255);
    set input = accept;
    child template leaf;
    child region rows;
  }
  template leaf = 11: panel {}
  template cell = 12: panel {}
  region rows = 5 owner root repeat cell keys [42]
    invalidates [structure, layout];
}
style {
  set leaf.color = rgba8(7, 8, 9, 255);
}
";

pub const GENEROUS_VALUES: [usize; 28] = [
    16_384, 4_096, 64, 16, 8, 32, 16, 8, 32, 64, 16, 16, 8, 64, 16, 512, 16, 8, 32, 16, 16, 16, 16,
    16, 16, 16, 768, 1_000_000,
];

pub fn limits() -> AuthoringLimitsV2 {
    AuthoringLimitsV2::new(GENEROUS_VALUES)
}

pub fn limits_with(kind: AuthoringLimitKindV2, value: usize) -> AuthoringLimitsV2 {
    let mut values = GENEROUS_VALUES;
    let index = AuthoringLimitKindV2::ALL
        .iter()
        .position(|candidate| *candidate == kind)
        .expect("the requested V2 limit must be registered");
    values[index] = value;
    AuthoringLimitsV2::new(values)
}

pub fn limits_with_generated_rust(value: usize) -> AuthoringLimitsV2 {
    limits_with(AuthoringLimitKindV2::GeneratedRustBytes, value)
}

pub fn semantic_limits() -> SemanticArtifactLimitsV2 {
    SemanticArtifactLimitsV2::new(1_000_000, 16_384, 1_024)
}

pub fn compile_fen(source: &str) -> CompiledAuthoringV2 {
    compile_fen_with(source, limits())
        .unwrap_or_else(|error| panic!("format-2 FEN should compile: {error}"))
}

pub fn compile_fen_with(
    source: &str,
    limits: AuthoringLimitsV2,
) -> Result<CompiledAuthoringV2, AuthoringDiagnosticV2> {
    compile_fen_v2(FenSourceV2::new(SOURCE_ID, source.as_bytes()), limits)
}

pub fn compile_ui(source: &str) -> CompiledAuthoringV2 {
    compile_ui_v2(ui_tokens(source), limits())
        .unwrap_or_else(|error| panic!("format-2 UI tokens should compile: {error}"))
}

pub fn compile_both() -> (CompiledAuthoringV2, CompiledAuthoringV2) {
    (compile_fen(FIXTURE), compile_ui(FIXTURE))
}

pub fn ui_tokens(source: &str) -> TokenStream {
    source
        .parse()
        .unwrap_or_else(|error| panic!("test source should tokenize: {error}"))
}

pub fn replace_once(source: &str, from: &str, to: &str) -> String {
    assert_eq!(
        source.match_indices(from).count(),
        1,
        "replacement needle must be unique: {from}"
    );
    source.replacen(from, to, 1)
}

pub fn replace_occurrence(source: &str, from: &str, to: &str, occurrence: usize) -> String {
    let start = source
        .match_indices(from)
        .nth(occurrence)
        .map(|(start, _)| start)
        .unwrap_or_else(|| panic!("missing occurrence {occurrence} of {from}"));
    let mut changed = String::with_capacity(source.len() - from.len() + to.len());
    changed.push_str(&source[..start]);
    changed.push_str(to);
    changed.push_str(&source[start + from.len()..]);
    changed
}
