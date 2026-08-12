mod api {
    pub use fenestra_ui_authoring::prototype::{
        AnchorKindV2, AuthoringDiagnosticKindV2, AuthoringDiagnosticV2, AuthoringFrontendV2,
        AuthoringLimitKindV2, AuthoringLimitsV2, CompiledAuthoringV2, DiagnosticLocationV2,
        FenSourceV2, GeneratedRustV2, PhysicalOriginV2, REFERENCE_AUTHORING_LIMITS_V2,
        REFERENCE_SEMANTIC_ARTIFACT_LIMITS_V2, SUPPORTED_AUTHORING_FORMAT_V2,
        SemanticArtifactErrorKindV2, SemanticArtifactErrorV2, SemanticArtifactLimitKindV2,
        SemanticArtifactLimitsV2, SemanticArtifactV2, SourceMapEntryV2, SourceMapV2,
        canonical_rust_v2, canonical_semantics_v2, compile_fen_v2, compile_ui_v2,
        diagnostic_tokens_v2, emit_tokens_v2, expand_ui, expand_ui_v2,
    };
}

#[path = "authoring_v2_surface/dependency.rs"]
mod dependency;
#[path = "authoring_v2_surface/registry.rs"]
mod registry;
#[path = "authoring_v2_surface/signatures.rs"]
mod signatures;
#[path = "authoring_v2_surface/source.rs"]
mod source;
#[path = "authoring_v2_surface/storage.rs"]
mod storage;
#[path = "authoring_v2_surface/support.rs"]
mod support;
#[path = "authoring_v2_surface/traits.rs"]
mod traits;
#[path = "authoring_v2_surface/vocabulary.rs"]
mod vocabulary;
