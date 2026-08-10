use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;

use crate::compiler::compile_ui_v1;
use crate::diagnostic::AuthoringDiagnosticV1;
use crate::emitter::emit_tokens_v1;
use crate::limits::AuthoringLimitsV1;
use crate::source::DiagnosticLocationV1;

/// Converts one closed authoring diagnostic into a spanned compiler error.
#[must_use]
pub fn diagnostic_tokens_v1(error: AuthoringDiagnosticV1) -> TokenStream {
    let span = physical_span(&error).unwrap_or_else(Span::call_site);
    let message = error.to_string();
    quote_spanned!(span=> compile_error!(#message))
}

/// Compiles and emits one bounded `ui!` input through the shared pipeline.
#[must_use]
pub fn expand_ui_v1(input: TokenStream, limits: AuthoringLimitsV1) -> TokenStream {
    let result =
        compile_ui_v1(input, limits).and_then(|compiled| emit_tokens_v1(&compiled, limits));
    match result {
        Ok(tokens) => tokens,
        Err(error) => diagnostic_tokens_v1(error),
    }
}

fn physical_span(error: &AuthoringDiagnosticV1) -> Option<Span> {
    match error.location() {
        DiagnosticLocationV1::Physical(origin)
        | DiagnosticLocationV1::Anchored {
            physical: origin, ..
        } => origin.ui_span(),
    }
}
