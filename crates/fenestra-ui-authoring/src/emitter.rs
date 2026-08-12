use proc_macro2::TokenStream;

use crate::compiled::CompiledAuthoringV1;
use crate::diagnostic::AuthoringDiagnosticV1;
use crate::limits::{AuthoringLimitKindV1, AuthoringLimitsV1};

pub(crate) mod builder;
mod construction;
mod schema;
mod style;
mod value;

use builder::tuple;
use construction::construction;
use schema::schema;
use style::style;

pub(crate) fn logical_tokens(resolved: &crate::resolved::ResolvedDocumentV1) -> Vec<TokenStream> {
    let namespace = resolved.schema.namespace;
    let revision = resolved.schema.revision;
    vec![
        schema(&resolved.schema),
        construction(&resolved.construction, namespace, revision),
        style(&resolved.style, namespace, revision),
    ]
}

/// Emits one deterministic target expression for the compiled IR triple.
///
/// # Errors
///
/// Returns a document-anchored generated-byte limit diagnostic when canonical
/// token spelling plus its required final line feed exceeds the supplied
/// inclusive limit.
pub fn emit_tokens_v1(
    compiled: &CompiledAuthoringV1,
    limits: AuthoringLimitsV1,
) -> Result<TokenStream, AuthoringDiagnosticV1> {
    let resolved = compiled.resolved();
    let tokens = tuple(logical_tokens(resolved), true);
    let Some(bytes) = tokens.to_string().len().checked_add(1) else {
        return Err(compiled.generated_rust_limit_failure());
    };
    if bytes > limits.limit(AuthoringLimitKindV1::GeneratedRustBytes) {
        return Err(compiled.generated_rust_limit_failure());
    }
    Ok(tokens)
}
