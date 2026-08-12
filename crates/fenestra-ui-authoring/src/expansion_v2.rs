use proc_macro2::{Spacing, Span, TokenStream, TokenTree};
use quote::quote_spanned;

use crate::compiler_v2::compile_ui_v2;
use crate::diagnostic_v2::AuthoringDiagnosticV2;
use crate::emitter_v2::emit_tokens_v2;
use crate::expansion::expand_ui_v1;
use crate::limits::AuthoringLimitsV1;
use crate::limits_v2::AuthoringLimitsV2;
use crate::source_v2::DiagnosticLocationV2;

/// Converts one closed format-2 diagnostic into a spanned compiler error.
#[must_use]
pub fn diagnostic_tokens_v2(error: AuthoringDiagnosticV2) -> TokenStream {
    let span = physical_span(&error).unwrap_or_else(Span::call_site);
    let message = error.to_string();
    quote_spanned!(span=> compile_error!(#message))
}

/// Compiles and emits one bounded format-2 `ui!` stream.
#[must_use]
pub fn expand_ui_v2(input: TokenStream, limits: AuthoringLimitsV2) -> TokenStream {
    match compile_ui_v2(input, limits).and_then(|compiled| emit_tokens_v2(&compiled, limits)) {
        Ok(tokens) => tokens,
        Err(error) => diagnostic_tokens_v2(error),
    }
}

/// Dispatches one `ui!` stream to its exact declared authoring format.
#[must_use]
pub fn expand_ui(
    input: TokenStream,
    v1_limits: AuthoringLimitsV1,
    v2_limits: AuthoringLimitsV2,
) -> TokenStream {
    if has_exact_v2_header(&input) {
        expand_ui_v2(input, v2_limits)
    } else {
        expand_ui_v1(input, v1_limits)
    }
}

fn has_exact_v2_header(input: &TokenStream) -> bool {
    let mut trees = input.clone().into_iter();
    matches!(trees.next(), Some(TokenTree::Ident(value)) if value == "format")
        && matches!(trees.next(), Some(TokenTree::Literal(value)) if value.to_string() == "2")
        && matches!(trees.next(), Some(TokenTree::Punct(value)) if value.as_char() == ';' && value.spacing() == Spacing::Alone)
}

fn physical_span(error: &AuthoringDiagnosticV2) -> Option<Span> {
    match error.location() {
        DiagnosticLocationV2::Physical(origin)
        | DiagnosticLocationV2::Anchored {
            physical: origin, ..
        } => origin.ui_span(),
    }
}
