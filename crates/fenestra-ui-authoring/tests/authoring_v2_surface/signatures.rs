use fenestra_ui_ir::prototype::{
    ConstructionProgram, SchemaManifest, SourceId, SourceSpan, SpatialProgramV2, StyleProgram,
};
use proc_macro2::TokenStream;

use crate::api::{
    AnchorKindV2, AuthoringDiagnosticKindV2, AuthoringDiagnosticV2, AuthoringFrontendV2,
    AuthoringLimitKindV2, AuthoringLimitsV2, CompiledAuthoringV2, DiagnosticLocationV2,
    FenSourceV2, GeneratedRustV2, PhysicalOriginV2, SemanticArtifactErrorKindV2,
    SemanticArtifactErrorV2, SemanticArtifactLimitKindV2, SemanticArtifactLimitsV2,
    SemanticArtifactV2, SourceMapEntryV2, SourceMapV2, canonical_rust_v2, canonical_semantics_v2,
    compile_fen_v2, compile_ui_v2, diagnostic_tokens_v2, emit_tokens_v2, expand_ui, expand_ui_v2,
};

use super::source::all_source;
use super::support::{item_attributes, names};

#[test]
fn two_frontends_emission_observation_and_expansion_signatures_are_exact() {
    let _: for<'a> fn(
        FenSourceV2<'a>,
        AuthoringLimitsV2,
    ) -> Result<CompiledAuthoringV2, AuthoringDiagnosticV2> = compile_fen_v2;
    let _: fn(
        TokenStream,
        AuthoringLimitsV2,
    ) -> Result<CompiledAuthoringV2, AuthoringDiagnosticV2> = compile_ui_v2;
    let _: fn(
        &CompiledAuthoringV2,
        AuthoringLimitsV2,
    ) -> Result<TokenStream, AuthoringDiagnosticV2> = emit_tokens_v2;
    let _: fn(
        &CompiledAuthoringV2,
        AuthoringLimitsV2,
    ) -> Result<GeneratedRustV2, AuthoringDiagnosticV2> = canonical_rust_v2;
    let _: fn(
        &CompiledAuthoringV2,
        SemanticArtifactLimitsV2,
    ) -> Result<SemanticArtifactV2, SemanticArtifactErrorV2> = canonical_semantics_v2;
    let _: fn(AuthoringDiagnosticV2) -> TokenStream = diagnostic_tokens_v2;
    let _: fn(TokenStream, AuthoringLimitsV2) -> TokenStream = expand_ui_v2;
    let _: fn(
        TokenStream,
        fenestra_ui_authoring::prototype::AuthoringLimitsV1,
        AuthoringLimitsV2,
    ) -> TokenStream = expand_ui;
}

#[test]
fn constructors_getters_and_observations_have_exact_types() {
    let _: for<'a> fn(SourceId, &'a [u8]) -> FenSourceV2<'a> = FenSourceV2::new;
    let _: fn([usize; 28]) -> AuthoringLimitsV2 = AuthoringLimitsV2::new;
    let _: fn(AuthoringLimitsV2, AuthoringLimitKindV2) -> usize = AuthoringLimitsV2::limit;
    let _: fn(usize, usize, usize) -> SemanticArtifactLimitsV2 = SemanticArtifactLimitsV2::new;
    let _: fn(SemanticArtifactLimitsV2, SemanticArtifactLimitKindV2) -> usize =
        SemanticArtifactLimitsV2::limit;

    let _: fn(&CompiledAuthoringV2) -> &SchemaManifest = CompiledAuthoringV2::schema;
    let _: fn(&CompiledAuthoringV2) -> &ConstructionProgram = CompiledAuthoringV2::construction;
    let _: fn(&CompiledAuthoringV2) -> &StyleProgram = CompiledAuthoringV2::style;
    let _: fn(&CompiledAuthoringV2) -> &SpatialProgramV2 = CompiledAuthoringV2::spatial;
    let _: fn(&CompiledAuthoringV2) -> &[u8] = CompiledAuthoringV2::logical_source_catalog;
    let _: fn(&CompiledAuthoringV2) -> &SourceMapV2 = CompiledAuthoringV2::source_map;

    let _: fn(&SourceMapV2) -> &[SourceMapEntryV2] = SourceMapV2::entries;
    let _: fn(&SourceMapEntryV2) -> SourceSpan = SourceMapEntryV2::logical_span;
    let _: fn(&SourceMapEntryV2) -> AnchorKindV2 = SourceMapEntryV2::anchor_kind;
    let _: fn(&SourceMapEntryV2) -> &str = SourceMapEntryV2::canonical_label;
    let _: fn(&SourceMapEntryV2) -> &PhysicalOriginV2 = SourceMapEntryV2::physical_origin;
    let _: fn(&GeneratedRustV2) -> &str = GeneratedRustV2::as_str;

    let _: fn(&PhysicalOriginV2) -> Option<SourceId> = PhysicalOriginV2::source_id;
    let _: fn(&PhysicalOriginV2) -> Option<(u32, u32)> = PhysicalOriginV2::fen_byte_range;
    let _: fn(&AuthoringDiagnosticV2) -> AuthoringFrontendV2 = AuthoringDiagnosticV2::frontend;
    let _: fn(&AuthoringDiagnosticV2) -> AuthoringDiagnosticKindV2 = AuthoringDiagnosticV2::kind;
    let _: fn(&AuthoringDiagnosticV2) -> &DiagnosticLocationV2 = AuthoringDiagnosticV2::location;

    let _: fn(&SemanticArtifactErrorV2) -> SemanticArtifactErrorKindV2 =
        SemanticArtifactErrorV2::kind;
    let _: fn(&SemanticArtifactV2) -> &str = SemanticArtifactV2::as_str;
    let _: fn(&SemanticArtifactV2) -> &[u8] = SemanticArtifactV2::as_bytes;
}

#[test]
fn free_function_attributes_match_the_existing_lane_policy() {
    let source = all_source();
    for name in ["diagnostic_tokens_v2", "expand_ui_v2", "expand_ui"] {
        assert_eq!(
            item_attributes(&source, &format!("pub fn {name}(")),
            names(&["#[must_use]"])
        );
    }
    for name in [
        "canonical_rust_v2",
        "canonical_semantics_v2",
        "compile_fen_v2",
        "compile_ui_v2",
        "emit_tokens_v2",
    ] {
        assert!(
            item_attributes(&source, &format!("pub fn {name}(")).is_empty(),
            "unexpected attributes on {name}"
        );
    }
}
