#![forbid(unsafe_code)]

//! Unpublished host-side typed authoring experiment for Fenestra.
//!
//! The current slice parses and lowers the closed bounded `.fen` grammar and
//! equivalent `ui!` input tokens into the existing raw schema, construction,
//! and style programs, emits target tokens, and generates canonical Rust. A
//! separate thin procedural-macro package delegates to this compiler.

mod canonical;
mod compiled;
mod compiler;
mod diagnostic;
mod diagnostic_v2;
mod emitter;
mod expansion;
mod fen;
mod limits;
mod limits_v2;
mod lower;
mod parsed;
mod parser;
mod resolved;
mod semantic;
mod semantic_v2;
mod source;
mod source_v2;
mod token;
mod ui;
mod version;
mod version_v2;
mod vocabulary;
mod vocabulary_v2;

/// Reserved unstable surface for the typed authoring experiment.
#[doc(hidden)]
pub mod prototype {
    pub use crate::canonical::{GeneratedRustV1, canonical_rust_v1};
    pub use crate::compiled::{CompiledAuthoringV1, SourceMapEntryV1, SourceMapV1};
    pub use crate::compiler::{compile_fen_v1, compile_ui_v1};
    pub use crate::diagnostic::{AuthoringDiagnosticKindV1, AuthoringDiagnosticV1};
    pub use crate::diagnostic_v2::{AuthoringDiagnosticKindV2, AuthoringDiagnosticV2};
    pub use crate::emitter::emit_tokens_v1;
    pub use crate::expansion::{diagnostic_tokens_v1, expand_ui_v1};
    pub use crate::limits::{
        AuthoringLimitKindV1, AuthoringLimitsV1, REFERENCE_AUTHORING_LIMITS_V1,
    };
    pub use crate::limits_v2::{AuthoringLimitKindV2, AuthoringLimitsV2};
    pub use crate::semantic::{
        REFERENCE_SEMANTIC_ARTIFACT_LIMITS_V1, SemanticArtifactErrorKindV1,
        SemanticArtifactErrorV1, SemanticArtifactLimitKindV1, SemanticArtifactLimitsV1,
        SemanticArtifactV1, canonical_semantics_v1,
    };
    pub use crate::semantic_v2::{
        SemanticArtifactErrorKindV2, SemanticArtifactErrorV2, SemanticArtifactLimitKindV2,
        SemanticArtifactLimitsV2, SemanticArtifactV2,
    };
    pub use crate::source::{DiagnosticLocationV1, FenSourceV1, PhysicalOriginV1};
    pub use crate::source_v2::{DiagnosticLocationV2, FenSourceV2, PhysicalOriginV2};
    pub use crate::version::{AuthoringFormatVersion, SUPPORTED_AUTHORING_FORMAT};
    pub use crate::version_v2::SUPPORTED_AUTHORING_FORMAT_V2;
    pub use crate::vocabulary::{AnchorKindV1, AuthoringFrontendV1};
    pub use crate::vocabulary_v2::{AnchorKindV2, AuthoringFrontendV2};
}
