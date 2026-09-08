use std::error::Error;
use std::fmt;

use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;

use crate::diagnostic_v2::{AuthoringDiagnosticKindV2, AuthoringDiagnosticV2};
use crate::limits_v2::AuthoringLimitKindV2;
use crate::source_v2::{DiagnosticLocationV2, PhysicalOriginV2};

/// A format-3 authoring error at a source byte range or Rust macro span.
#[derive(Clone)]
pub struct Diagnostic {
    message: String,
    origin: PhysicalOriginV2,
}

impl Diagnostic {
    pub(super) fn new(message: impl Into<String>, origin: PhysicalOriginV2) -> Self {
        Self {
            message: message.into(),
            origin,
        }
    }

    /// Returns the half-open byte range for `.fen` input, or `None` for `ui!`.
    #[must_use]
    pub fn byte_range(&self) -> Option<(usize, usize)> {
        self.origin
            .fen_byte_range()
            .map(|(start, end)| (start as usize, end as usize))
    }

    pub(super) fn tokens(&self) -> TokenStream {
        let span = self.origin.ui_span().unwrap_or_else(Span::call_site);
        let message = self.to_string();
        quote_spanned!(span=> compile_error!(#message))
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl fmt::Debug for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Diagnostic")
            .field("message", &self.message)
            .field("byte_range", &self.byte_range())
            .finish()
    }
}

impl Error for Diagnostic {}

impl From<AuthoringDiagnosticV2> for Diagnostic {
    fn from(error: AuthoringDiagnosticV2) -> Self {
        let origin = match *error.location() {
            DiagnosticLocationV2::Physical(origin)
            | DiagnosticLocationV2::Anchored {
                physical: origin, ..
            } => origin,
        };
        let message = match error.kind() {
            AuthoringDiagnosticKindV2::UnsupportedToken => {
                "unsupported token; use ASCII identifiers and unsuffixed decimal integers"
            }
            AuthoringDiagnosticKindV2::LimitExceeded(AuthoringLimitKindV2::Tokens) => {
                "authoring limit exceeded: tokens"
            }
            AuthoringDiagnosticKindV2::LimitExceeded(AuthoringLimitKindV2::NestingDepth) => {
                "authoring limit exceeded: delimiter nesting depth"
            }
            AuthoringDiagnosticKindV2::LimitExceeded(AuthoringLimitKindV2::IdentifierBytes) => {
                "authoring limit exceeded: identifier bytes"
            }
            _ => "invalid format-3 token input",
        };
        Self::new(message, origin)
    }
}
