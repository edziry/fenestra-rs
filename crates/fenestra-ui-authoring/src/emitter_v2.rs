use proc_macro2::TokenStream;

use crate::compiled_v2::CompiledAuthoringV2;
use crate::diagnostic_v2::AuthoringDiagnosticV2;
use crate::emitter::builder::tuple;
use crate::limits_v2::{AuthoringLimitKindV2, AuthoringLimitsV2};

mod spatial;

/// Emits one deterministic target expression for the compiled IR quadruple.
///
/// # Errors
///
/// Returns a document-anchored generated-byte limit diagnostic when canonical
/// token spelling plus its required final line feed exceeds the supplied
/// inclusive limit.
pub fn emit_tokens_v2(
    compiled: &CompiledAuthoringV2,
    limits: AuthoringLimitsV2,
) -> Result<TokenStream, AuthoringDiagnosticV2> {
    let mut items = crate::emitter::logical_tokens(&compiled.resolved().core);
    items.push(spatial::spatial(compiled.spatial()));
    let tokens = tuple(items, true);
    let Some(bytes) = tokens.to_string().len().checked_add(1) else {
        return Err(compiled.generated_rust_limit_failure());
    };
    if bytes > limits.limit(AuthoringLimitKindV2::GeneratedRustBytes) {
        return Err(compiled.generated_rust_limit_failure());
    }
    Ok(tokens)
}
