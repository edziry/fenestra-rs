use fenestra_ui_ir::prototype::SourceId;

use crate::diagnostic::{AuthoringDiagnosticKindV1, AuthoringDiagnosticV1};
use crate::limits::{AuthoringLimitKindV1, AuthoringLimitsV1};
use crate::parsed::ParsedDocumentV1;
use crate::parser::parse_document_v1;
use crate::source::{DiagnosticLocationV1, PhysicalOriginV1};
use crate::token::{AbstractTokenKindV1, AbstractTokenV1, PunctuationV1};
use crate::vocabulary::AuthoringFrontendV1;

pub(crate) fn parse_fen_document_v1(
    source: SourceId,
    text: &str,
    limits: AuthoringLimitsV1,
) -> Result<ParsedDocumentV1, AuthoringDiagnosticV1> {
    let source_limit = limits.limit(AuthoringLimitKindV1::FenSourceBytes);
    if text.len() > source_limit {
        return Err(physical_failure(
            source,
            AuthoringDiagnosticKindV1::LimitExceeded(AuthoringLimitKindV1::FenSourceBytes),
            source_limit,
            source_limit.saturating_add(1),
        ));
    }
    let tokens = lex_fen_v1(source, text, limits)?;
    let eof = bounded_offset(text.len());
    parse_document_v1(
        AuthoringFrontendV1::Fen,
        PhysicalOriginV1::fen_bytes(source, eof, eof),
        tokens,
        limits,
    )
}

fn lex_fen_v1(
    source: SourceId,
    text: &str,
    limits: AuthoringLimitsV1,
) -> Result<Vec<AbstractTokenV1>, AuthoringDiagnosticV1> {
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
            let end = unsupported_lexeme_end(text, start);
            return Err(physical_failure(
                source,
                AuthoringDiagnosticKindV1::UnsupportedToken,
                start,
                end,
            ));
        } else if is_identifier_start(bytes[offset]) {
            offset += 1;
            while offset < bytes.len() && is_identifier_continue(bytes[offset]) {
                offset += 1;
            }
            AbstractTokenKindV1::Identifier(text[start..offset].into())
        } else if bytes[offset].is_ascii_digit() {
            offset += 1;
            while offset < bytes.len() && bytes[offset].is_ascii_digit() {
                offset += 1;
            }
            if offset < bytes.len() && is_numeric_continuation(bytes[offset]) {
                let end = unsupported_lexeme_end(text, start);
                return Err(physical_failure(
                    source,
                    AuthoringDiagnosticKindV1::UnsupportedToken,
                    start,
                    end,
                ));
            }
            AbstractTokenKindV1::UnsignedDecimal(text[start..offset].into())
        } else if let Some(punctuation) = punctuation(bytes[offset]) {
            offset += 1;
            AbstractTokenKindV1::Punctuation(punctuation)
        } else {
            let end = unsupported_lexeme_end(text, start);
            return Err(physical_failure(
                source,
                AuthoringDiagnosticKindV1::UnsupportedToken,
                start,
                end,
            ));
        };

        if tokens.len() >= limits.limit(AuthoringLimitKindV1::Tokens) {
            return Err(physical_failure(
                source,
                AuthoringDiagnosticKindV1::LimitExceeded(AuthoringLimitKindV1::Tokens),
                start,
                offset,
            ));
        }
        if matches!(&kind, AbstractTokenKindV1::Identifier(_))
            && offset - start > limits.limit(AuthoringLimitKindV1::IdentifierBytes)
        {
            return Err(physical_failure(
                source,
                AuthoringDiagnosticKindV1::LimitExceeded(AuthoringLimitKindV1::IdentifierBytes),
                start,
                offset,
            ));
        }

        if let AbstractTokenKindV1::Punctuation(punctuation) = &kind {
            let punctuation = *punctuation;
            if is_opening(punctuation) {
                let next_depth = depth.checked_add(1).ok_or_else(|| {
                    physical_failure(
                        source,
                        AuthoringDiagnosticKindV1::LimitExceeded(
                            AuthoringLimitKindV1::NestingDepth,
                        ),
                        start,
                        offset,
                    )
                })?;
                if next_depth > limits.limit(AuthoringLimitKindV1::NestingDepth) {
                    return Err(physical_failure(
                        source,
                        AuthoringDiagnosticKindV1::LimitExceeded(
                            AuthoringLimitKindV1::NestingDepth,
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

        tokens.push(AbstractTokenV1 {
            kind,
            physical: PhysicalOriginV1::fen_bytes(
                source,
                bounded_offset(start),
                bounded_offset(offset),
            ),
        });
    }

    Ok(tokens)
}

fn starts_raw_identifier(bytes: &[u8], offset: usize) -> bool {
    bytes[offset] == b'r' && bytes.get(offset + 1) == Some(&b'#')
}

const fn is_numeric_continuation(byte: u8) -> bool {
    is_identifier_start(byte) || byte == b'.'
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

fn punctuation(byte: u8) -> Option<PunctuationV1> {
    Some(match byte {
        b'{' => PunctuationV1::OpenBrace,
        b'}' => PunctuationV1::CloseBrace,
        b'[' => PunctuationV1::OpenBracket,
        b']' => PunctuationV1::CloseBracket,
        b'(' => PunctuationV1::OpenParenthesis,
        b')' => PunctuationV1::CloseParenthesis,
        b':' => PunctuationV1::Colon,
        b';' => PunctuationV1::Semicolon,
        b',' => PunctuationV1::Comma,
        b'=' => PunctuationV1::Equals,
        b'.' => PunctuationV1::Dot,
        b'-' => PunctuationV1::Minus,
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

const fn is_opening(punctuation: PunctuationV1) -> bool {
    matches!(
        punctuation,
        PunctuationV1::OpenBrace | PunctuationV1::OpenBracket | PunctuationV1::OpenParenthesis
    )
}

const fn is_closing(punctuation: PunctuationV1) -> bool {
    matches!(
        punctuation,
        PunctuationV1::CloseBrace | PunctuationV1::CloseBracket | PunctuationV1::CloseParenthesis
    )
}

fn physical_failure(
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
