use fenestra_ui_ir::prototype::SourceId;

use crate::diagnostic_v2::{AuthoringDiagnosticKindV2, AuthoringDiagnosticV2};
use crate::limits_v2::{AuthoringLimitKindV2, AuthoringLimitsV2};
use crate::parsed_v2::ParsedDocumentV2;
use crate::parser_v2::parse_document_v2;
use crate::source_v2::{DiagnosticLocationV2, PhysicalOriginV2};
use crate::token::{AbstractToken, AbstractTokenKind, Punctuation};
use crate::vocabulary_v2::AuthoringFrontendV2;

pub(crate) fn parse_fen_document_v2(
    source: SourceId,
    text: &str,
    limits: AuthoringLimitsV2,
) -> Result<ParsedDocumentV2, AuthoringDiagnosticV2> {
    let source_limit = limits
        .limit(AuthoringLimitKindV2::FenSourceBytes)
        .min(u32::MAX as usize);
    if text.len() > source_limit {
        return Err(physical_failure(
            source,
            AuthoringDiagnosticKindV2::LimitExceeded(AuthoringLimitKindV2::FenSourceBytes),
            source_limit,
            source_limit.saturating_add(1),
        ));
    }
    let tokens = lex_fen_v2(source, text, limits)?;
    let eof = bounded_offset(text.len());
    parse_document_v2(
        AuthoringFrontendV2::Fen,
        PhysicalOriginV2::fen_bytes(source, eof, eof),
        tokens,
        limits,
    )
}

fn lex_fen_v2(
    source: SourceId,
    text: &str,
    limits: AuthoringLimitsV2,
) -> Result<Vec<AbstractToken<PhysicalOriginV2>>, AuthoringDiagnosticV2> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut offset = 0;
    let mut depth = 0usize;
    while offset < bytes.len() {
        if is_whitespace(bytes[offset]) {
            offset += 1;
            continue;
        }
        let start = offset;
        let kind = if starts_raw_identifier(bytes, offset) {
            return Err(unsupported(source, text, start));
        } else if is_identifier_start(bytes[offset]) {
            offset += 1;
            while offset < bytes.len() && is_identifier_continue(bytes[offset]) {
                offset += 1;
            }
            AbstractTokenKind::Identifier(text[start..offset].into())
        } else if bytes[offset].is_ascii_digit() {
            offset += 1;
            while offset < bytes.len() && bytes[offset].is_ascii_digit() {
                offset += 1;
            }
            if offset < bytes.len() && is_numeric_continuation(bytes, offset) {
                return Err(unsupported(source, text, start));
            }
            AbstractTokenKind::UnsignedDecimal(text[start..offset].into())
        } else if let Some(punctuation) = punctuation(bytes[offset]) {
            offset += 1;
            AbstractTokenKind::Punctuation(punctuation)
        } else {
            return Err(unsupported(source, text, start));
        };
        if tokens.len() >= limits.limit(AuthoringLimitKindV2::Tokens) {
            return Err(physical_failure(
                source,
                AuthoringDiagnosticKindV2::LimitExceeded(AuthoringLimitKindV2::Tokens),
                start,
                offset,
            ));
        }
        if matches!(&kind, AbstractTokenKind::Identifier(_))
            && offset - start > limits.limit(AuthoringLimitKindV2::IdentifierBytes)
        {
            return Err(physical_failure(
                source,
                AuthoringDiagnosticKindV2::LimitExceeded(AuthoringLimitKindV2::IdentifierBytes),
                start,
                offset,
            ));
        }
        if let AbstractTokenKind::Punctuation(punctuation) = kind {
            if is_opening(punctuation) {
                let next_depth = depth.checked_add(1).ok_or_else(|| {
                    physical_failure(
                        source,
                        AuthoringDiagnosticKindV2::LimitExceeded(
                            AuthoringLimitKindV2::NestingDepth,
                        ),
                        start,
                        offset,
                    )
                })?;
                if next_depth > limits.limit(AuthoringLimitKindV2::NestingDepth) {
                    return Err(physical_failure(
                        source,
                        AuthoringDiagnosticKindV2::LimitExceeded(
                            AuthoringLimitKindV2::NestingDepth,
                        ),
                        start,
                        offset,
                    ));
                }
                depth = next_depth;
            } else if is_closing(punctuation) {
                depth = depth.saturating_sub(1);
            }
        }
        tokens.push(AbstractToken {
            kind,
            physical: PhysicalOriginV2::fen_bytes(
                source,
                bounded_offset(start),
                bounded_offset(offset),
            ),
        });
    }
    Ok(tokens)
}

fn unsupported(source: SourceId, text: &str, start: usize) -> AuthoringDiagnosticV2 {
    physical_failure(
        source,
        AuthoringDiagnosticKindV2::UnsupportedToken,
        start,
        unsupported_lexeme_end(text, start),
    )
}

fn starts_raw_identifier(bytes: &[u8], offset: usize) -> bool {
    bytes[offset] == b'r' && bytes.get(offset + 1) == Some(&b'#')
}

fn is_numeric_continuation(bytes: &[u8], offset: usize) -> bool {
    let byte = bytes[offset];
    if is_identifier_start(byte) {
        return true;
    }
    byte == b'.'
        && !matches!(
            bytes.get(offset + 1),
            Some(next) if *next == b'.' || is_identifier_start(*next) || !next.is_ascii()
        )
}

fn unsupported_lexeme_end(text: &str, start: usize) -> usize {
    let mut end = start;
    for (relative, character) in text[start..].char_indices() {
        if relative > 0 && is_lexeme_boundary(character) {
            break;
        }
        end = start + relative + character.len_utf8();
    }
    end
}

fn is_lexeme_boundary(character: char) -> bool {
    character.is_ascii_whitespace()
        || matches!(
            character,
            '{' | '}' | '[' | ']' | '(' | ')' | ':' | ';' | ',' | '=' | '-'
        )
}

fn punctuation(byte: u8) -> Option<Punctuation> {
    Some(match byte {
        b'{' => Punctuation::OpenBrace,
        b'}' => Punctuation::CloseBrace,
        b'[' => Punctuation::OpenBracket,
        b']' => Punctuation::CloseBracket,
        b'(' => Punctuation::OpenParenthesis,
        b')' => Punctuation::CloseParenthesis,
        b':' => Punctuation::Colon,
        b';' => Punctuation::Semicolon,
        b',' => Punctuation::Comma,
        b'=' => Punctuation::Equals,
        b'.' => Punctuation::Dot,
        b'-' => Punctuation::Minus,
        _ => return None,
    })
}

const fn is_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n')
}

const fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

const fn is_identifier_continue(byte: u8) -> bool {
    is_identifier_start(byte) || byte.is_ascii_digit()
}

const fn is_opening(punctuation: Punctuation) -> bool {
    matches!(
        punctuation,
        Punctuation::OpenBrace | Punctuation::OpenBracket | Punctuation::OpenParenthesis
    )
}

const fn is_closing(punctuation: Punctuation) -> bool {
    matches!(
        punctuation,
        Punctuation::CloseBrace | Punctuation::CloseBracket | Punctuation::CloseParenthesis
    )
}

fn physical_failure(
    source: SourceId,
    kind: AuthoringDiagnosticKindV2,
    start: usize,
    end: usize,
) -> AuthoringDiagnosticV2 {
    AuthoringDiagnosticV2::new(
        AuthoringFrontendV2::Fen,
        kind,
        DiagnosticLocationV2::Physical(PhysicalOriginV2::fen_bytes(
            source,
            bounded_offset(start),
            bounded_offset(end),
        )),
    )
}

fn bounded_offset(offset: usize) -> u32 {
    u32::try_from(offset).unwrap_or(u32::MAX)
}
