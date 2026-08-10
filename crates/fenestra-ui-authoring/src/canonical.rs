use std::fmt;

use crate::compiled::CompiledAuthoringV1;
use crate::diagnostic::AuthoringDiagnosticV1;
use crate::emitter::emit_tokens_v1;
use crate::limits::AuthoringLimitsV1;

/// Opaque canonical Rust source generated from one compiled document.
pub struct GeneratedRustV1 {
    source: Box<str>,
}

impl GeneratedRustV1 {
    /// Returns the canonical Rust expression with exactly one final line feed.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.source
    }
}

impl fmt::Debug for GeneratedRustV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GeneratedRustV1")
            .field("bytes", &self.source.len())
            .finish()
    }
}

/// Generates deterministic canonical Rust for one compiled authoring document.
///
/// # Errors
///
/// Returns the compiled document's generated-byte limit diagnostic when the
/// emitted token spelling plus its required final line feed exceeds `limits`.
pub fn canonical_rust_v1(
    compiled: &CompiledAuthoringV1,
    limits: AuthoringLimitsV1,
) -> Result<GeneratedRustV1, AuthoringDiagnosticV1> {
    let tokens = emit_tokens_v1(compiled, limits)?;
    let mut source = tokens.to_string();
    source.push('\n');
    Ok(GeneratedRustV1 {
        source: source.into_boxed_str(),
    })
}
