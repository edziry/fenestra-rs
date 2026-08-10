#![forbid(unsafe_code)]

//! Unpublished host-side typed authoring experiment for Fenestra.
//!
//! The current slice parses and lowers the closed bounded `.fen` grammar and
//! equivalent `ui!` input tokens into the existing raw schema, construction,
//! and style programs. Canonical text generation and the procedural macro
//! remain absent.

mod canonical;
mod compiled;
mod compiler;
mod diagnostic;
mod emitter;
mod fen;
mod limits;
mod lower;
mod parsed;
mod parser;
mod resolved;
mod source;
mod token;
mod ui;
mod version;
mod vocabulary;

/// Reserved unstable surface for the typed authoring experiment.
#[doc(hidden)]
pub mod prototype {
    pub use crate::canonical::{GeneratedRustV1, canonical_rust_v1};
    pub use crate::compiled::{CompiledAuthoringV1, SourceMapEntryV1, SourceMapV1};
    pub use crate::compiler::{compile_fen_v1, compile_ui_v1};
    pub use crate::diagnostic::{AuthoringDiagnosticKindV1, AuthoringDiagnosticV1};
    pub use crate::emitter::emit_tokens_v1;
    pub use crate::limits::{AuthoringLimitKindV1, AuthoringLimitsV1};
    pub use crate::source::{DiagnosticLocationV1, FenSourceV1, PhysicalOriginV1};
    pub use crate::version::{AuthoringFormatVersion, SUPPORTED_AUTHORING_FORMAT};
    pub use crate::vocabulary::{AnchorKindV1, AuthoringFrontendV1};
}
