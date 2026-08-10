use fenestra_ui_ir::prototype::SourceId;

use crate::diagnostic::{AuthoringDiagnosticKindV1, AuthoringDiagnosticV1};
use crate::limits::{AuthoringLimitKindV1, AuthoringLimitsV1};
use crate::source::{DiagnosticLocationV1, FenSourceV1, PhysicalOriginV1};
use crate::vocabulary::AuthoringFrontendV1;

/// Opaque result of a successful version-1 authoring compilation.
///
/// No value is produced until a later parser and lowering slice exists.
#[derive(Debug)]
pub struct CompiledAuthoringV1 {
    _private: (),
}

/// Performs the bounded version-1 `.fen` preflight.
///
/// Grammar parsing is intentionally outside this initial compiler slice, so a
/// valid nonempty source currently produces a typed placeholder parse failure.
///
/// # Errors
///
/// Returns the first source-byte, UTF-8, empty-input, or placeholder parse
/// diagnostic in deterministic validation order.
pub fn compile_fen_v1(
    source: FenSourceV1<'_>,
    limits: AuthoringLimitsV1,
) -> Result<CompiledAuthoringV1, AuthoringDiagnosticV1> {
    let source_id = source.source();
    let bytes = source.bytes();
    let source_limit = limits.limit(AuthoringLimitKindV1::FenSourceBytes);

    if bytes.len() > source_limit {
        return Err(fen_failure(
            source_id,
            AuthoringDiagnosticKindV1::LimitExceeded(AuthoringLimitKindV1::FenSourceBytes),
            source_limit,
            source_limit.saturating_add(1),
        ));
    }

    let text = std::str::from_utf8(bytes).map_err(|error| {
        let start = error.valid_up_to();
        let width = error.error_len().unwrap_or_else(|| bytes.len() - start);
        fen_failure(
            source_id,
            AuthoringDiagnosticKindV1::InvalidUtf8,
            start,
            start.saturating_add(width),
        )
    })?;

    if text.is_empty() {
        return Err(fen_failure(
            source_id,
            AuthoringDiagnosticKindV1::UnexpectedEof,
            0,
            0,
        ));
    }

    let first_token_end = text
        .chars()
        .next()
        .map_or(0, |character| character.len_utf8());
    Err(fen_failure(
        source_id,
        AuthoringDiagnosticKindV1::UnexpectedToken,
        0,
        first_token_end,
    ))
}

fn fen_failure(
    source: SourceId,
    kind: AuthoringDiagnosticKindV1,
    start: usize,
    end: usize,
) -> AuthoringDiagnosticV1 {
    AuthoringDiagnosticV1::new(
        AuthoringFrontendV1::Fen,
        kind,
        DiagnosticLocationV1::Physical(PhysicalOriginV1::fen_bytes(
            source,
            bounded_offset(start),
            bounded_offset(end),
        )),
    )
}

fn bounded_offset(offset: usize) -> u32 {
    u32::try_from(offset).unwrap_or(u32::MAX)
}
