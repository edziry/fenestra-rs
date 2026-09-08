//! Compile nested format-3 views to the public `fenestra-ui` facade.
//!
//! The generated expression constructs a `fenestra_ui::View`. This host-side
//! crate belongs in build dependencies; generated target code never invokes it.

mod build;
mod comments;
mod diagnostic;
mod emit;
mod lex;
mod limits;
mod parser;
mod strings;
mod token;

use std::fmt;

use fenestra_ui_ir::prototype::SourceId;
use proc_macro2::TokenStream;

use crate::source_v2::PhysicalOriginV2;

pub use build::{BuildError, build_file};
pub use diagnostic::Diagnostic;
pub use limits::Limits;

/// A compiled public `View` expression with no compiler dependency at runtime.
pub struct CompiledView {
    source: String,
    tokens: TokenStream,
}

impl CompiledView {
    /// Returns deterministic Rust expression source, including a final line feed.
    #[must_use]
    pub fn rust_source(&self) -> &str {
        &self.source
    }

    /// Returns the expression as Rust tokens for procedural-macro expansion.
    #[must_use]
    pub fn tokens(&self) -> TokenStream {
        self.tokens.clone()
    }
}

impl fmt::Debug for CompiledView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompiledView")
            .field("generated_bytes", &self.source.len())
            .finish()
    }
}

/// Compiles a UTF-8 `.fen` view using the default resource limits.
///
/// # Errors
/// Returns the first lexical, grammar, property, name, or resource diagnostic.
pub fn compile_fen(bytes: &[u8]) -> Result<CompiledView, Diagnostic> {
    compile_fen_with_limits(bytes, Limits::default())
}

/// Compiles a UTF-8 `.fen` view with explicit inclusive resource limits.
///
/// # Errors
/// Returns the first lexical, grammar, property, name, or resource diagnostic.
/// Source size is checked before UTF-8 validation or token allocation.
pub fn compile_fen_with_limits(bytes: &[u8], limits: Limits) -> Result<CompiledView, Diagnostic> {
    let maximum = limits.source_bytes.min(u32::MAX as usize);
    if bytes.len() > maximum {
        return Err(Diagnostic::new(
            "authoring limit exceeded: source bytes",
            origin(maximum, maximum.saturating_add(1)),
        ));
    }
    let text = std::str::from_utf8(bytes).map_err(|error| {
        let start = error.valid_up_to();
        let width = error.error_len().unwrap_or(bytes.len() - start);
        Diagnostic::new("source is not valid UTF-8", origin(start, start + width))
    })?;
    let tokens = lex::fen(text, limits)?;
    let document = parser::parse(tokens, origin(bytes.len(), bytes.len()), limits)?;
    emit::emit(&document, limits)
}

/// Compiles a `ui!` token stream using the default resource limits.
///
/// # Errors
/// Returns a diagnostic retaining the offending token's Rust source span.
pub fn compile_ui(input: TokenStream) -> Result<CompiledView, Diagnostic> {
    compile_ui_with_limits(input, Limits::default())
}

/// Compiles a `ui!` stream with explicit inclusive resource limits.
///
/// # Errors
/// Returns the first lexical, grammar, property, name, or resource diagnostic.
/// Source byte limits apply only to `.fen`; token and nesting limits apply here.
pub fn compile_ui_with_limits(
    input: TokenStream,
    limits: Limits,
) -> Result<CompiledView, Diagnostic> {
    let (tokens, eof) = lex::ui(input, limits)?;
    let document = parser::parse(tokens, eof, limits)?;
    emit::emit(&document, limits)
}

/// Recognizes an exact `format 3;` header after ordinary comments and whitespace.
///
/// Only the prefix is examined. An invalid body, including invalid UTF-8 or an
/// unterminated body comment, still dispatches to the format-3 compiler.
#[must_use]
pub fn has_format_header(bytes: &[u8]) -> bool {
    fn header(bytes: &[u8]) -> Option<()> {
        let mut offset = comments::skip_trivia(bytes, 0).ok()?;
        let start = offset;
        while bytes
            .get(offset)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        {
            offset += 1;
        }
        if bytes.get(start..offset)? != b"format" {
            return None;
        }
        offset = comments::skip_trivia(bytes, offset).ok()?;
        if bytes.get(offset) != Some(&b'3') {
            return None;
        }
        offset = comments::skip_trivia(bytes, offset + 1).ok()?;
        (bytes.get(offset) == Some(&b';')).then_some(())
    }
    header(bytes).is_some()
}

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    match compile_ui(input) {
        Ok(compiled) => compiled.tokens(),
        Err(error) => error.tokens(),
    }
}

fn origin(start: usize, end: usize) -> PhysicalOriginV2 {
    PhysicalOriginV2::fen_bytes(
        SourceId::new(0),
        u32::try_from(start).unwrap_or(u32::MAX),
        u32::try_from(end).unwrap_or(u32::MAX),
    )
}
