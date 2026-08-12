use crate::compiled_v2::{CompiledAuthoringV2, GeneratedRustV2};
use crate::diagnostic_v2::AuthoringDiagnosticV2;
use crate::emitter_v2::emit_tokens_v2;
use crate::limits_v2::AuthoringLimitsV2;

/// Generates deterministic canonical Rust for one format-2 document.
///
/// # Errors
///
/// Returns the document-anchored generated-byte limit diagnostic when the
/// emitted token spelling plus its required final line feed exceeds `limits`.
pub fn canonical_rust_v2(
    compiled: &CompiledAuthoringV2,
    limits: AuthoringLimitsV2,
) -> Result<GeneratedRustV2, AuthoringDiagnosticV2> {
    let tokens = emit_tokens_v2(compiled, limits)?;
    let mut source = tokens.to_string();
    source.push('\n');
    Ok(GeneratedRustV2::new(source.into_boxed_str()))
}
