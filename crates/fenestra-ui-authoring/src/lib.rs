#![forbid(unsafe_code)]

//! Unpublished host-side typed authoring experiment for Fenestra.
//!
//! The initial slice exposes only closed contract vocabulary and bounded
//! `.fen` preflight diagnostics. Parsing and emission remain intentionally
//! absent.

mod compiler;
mod diagnostic;
mod limits;
mod source;
mod version;
mod vocabulary;

/// Reserved unstable surface for the typed authoring experiment.
#[doc(hidden)]
pub mod prototype {
    pub use crate::compiler::{CompiledAuthoringV1, compile_fen_v1};
    pub use crate::diagnostic::{AuthoringDiagnosticKindV1, AuthoringDiagnosticV1};
    pub use crate::limits::{AuthoringLimitKindV1, AuthoringLimitsV1};
    pub use crate::source::{DiagnosticLocationV1, FenSourceV1, PhysicalOriginV1};
    pub use crate::version::{AuthoringFormatVersion, SUPPORTED_AUTHORING_FORMAT};
    pub use crate::vocabulary::{AnchorKindV1, AuthoringFrontendV1};
}
