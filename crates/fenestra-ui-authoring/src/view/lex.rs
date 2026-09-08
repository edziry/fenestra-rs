use std::collections::BTreeMap;

use fenestra_ui_ir::prototype::SourceId;
use proc_macro2::{Delimiter, Span, TokenStream, TokenTree};

use crate::fen_v2::lex_fen_v2;
use crate::source_v2::PhysicalOriginV2;
use crate::token::Punctuation;
use crate::ui_v2::adapt_ui_tokens_v2;

use super::token::{Kind, Token};
use super::{Diagnostic, Limits, comments, origin, strings};

pub(super) fn fen(text: &str, limits: Limits) -> Result<Vec<Token>, Diagnostic> {
    let text = comments::normalize(text)?;
    let mut masked = text.as_bytes().to_vec();
    let mut values = BTreeMap::new();
    let mut offset = 0;
    while offset < text.len() {
        if !strings::starts(text.as_bytes(), offset) {
            offset += 1;
            continue;
        }
        let start = offset;
        let end = strings::end(&text, start)
            .map_err(|message| Diagnostic::new(message, origin(start, text.len())))?;
        if start > 0 && !boundary(text.as_bytes()[start - 1])
            || end < text.len() && !boundary(text.as_bytes()[end])
        {
            return Err(Diagnostic::new(
                "string literals require token boundaries and cannot have prefixes or suffixes",
                origin(start, end),
            ));
        }
        let value = strings::decode(&text[start..end])
            .map_err(|message| Diagnostic::new(message, origin(start, end)))?;
        // A numeric placeholder is one existing frontend token. All remaining
        // bytes become whitespace, retaining diagnostics after Unicode strings.
        masked[start] = b'0';
        masked[start + 1..end].fill(b' ');
        values.insert(start, (end, value));
        offset = end;
    }
    let masked = String::from_utf8(masked).expect("complete string bytes become ASCII");
    let tokens = lex_fen_v2(SourceId::new(0), &masked, limits.frontend()).map_err(|error| {
        let error = Diagnostic::from(error);
        if let Some((start, _)) = error.byte_range()
            && let Some((end, _)) = values.get(&start)
        {
            return Diagnostic::new(error.to_string(), origin(start, *end));
        }
        error
    })?;
    tokens
        .into_iter()
        .map(|token| {
            let start = token.physical.fen_byte_range().expect("file token").0 as usize;
            Ok(match values.remove(&start) {
                Some((end, value)) => Token {
                    kind: Kind::String(value),
                    physical: origin(start, end),
                },
                None => token.into(),
            })
        })
        .collect()
}

fn boundary(byte: u8) -> bool {
    byte.is_ascii_whitespace()
        || matches!(
            byte,
            b'{' | b'}'
                | b'['
                | b']'
                | b'('
                | b')'
                | b':'
                | b';'
                | b','
                | b'='
                | b'-'
                | b'.'
                | b'/'
        )
}

pub(super) fn ui(
    stream: TokenStream,
    limits: Limits,
) -> Result<(Vec<Token>, PhysicalOriginV2), Diagnostic> {
    let mut tokens = Vec::new();
    let mut frames = vec![(stream.into_iter(), None)];
    while let Some((input, _)) = frames.last_mut() {
        let Some(tree) = input.next() else {
            if let Some((_, Some(token))) = frames.pop() {
                push(&mut tokens, token, limits)?;
            }
            continue;
        };
        match tree {
            TokenTree::Group(group) => {
                let (opening, closing) = match group.delimiter() {
                    Delimiter::Brace => (Punctuation::OpenBrace, Punctuation::CloseBrace),
                    Delimiter::Bracket => (Punctuation::OpenBracket, Punctuation::CloseBracket),
                    Delimiter::Parenthesis => {
                        (Punctuation::OpenParenthesis, Punctuation::CloseParenthesis)
                    }
                    Delimiter::None => {
                        // Preserve the existing frontend diagnostic for opaque groups.
                        adapt_ui_tokens_v2(TokenTree::Group(group).into(), limits.frontend())?;
                        unreachable!("the shared frontend rejects opaque groups");
                    }
                };
                let physical = PhysicalOriginV2::ui_token(group.span_open());
                push(
                    &mut tokens,
                    Token {
                        kind: Kind::Punctuation(opening),
                        physical,
                    },
                    limits,
                )?;
                if frames.len() > limits.nesting_depth {
                    return Err(Diagnostic::new(
                        "authoring limit exceeded: delimiter nesting depth",
                        physical,
                    ));
                }
                let closing = Token {
                    kind: Kind::Punctuation(closing),
                    physical: PhysicalOriginV2::ui_token(group.span_close()),
                };
                frames.push((group.stream().into_iter(), Some(closing)));
            }
            TokenTree::Literal(ref literal)
                if strings::starts(literal.to_string().as_bytes(), 0) =>
            {
                let physical = PhysicalOriginV2::ui_token(literal.span());
                let value = strings::decode(&literal.to_string())
                    .map_err(|message| Diagnostic::new(message, physical))?;
                push(
                    &mut tokens,
                    Token {
                        kind: Kind::String(value),
                        physical,
                    },
                    limits,
                )?;
            }
            _ => {
                // Delegate unchanged identifiers, numbers, and punctuation to
                // the frozen frontend instead of duplicating its lexical rules.
                let (mut leaf, _) = adapt_ui_tokens_v2(tree.into(), limits.frontend())?;
                let token = leaf.pop().expect("one non-group Rust token");
                push(&mut tokens, token.into(), limits)?;
            }
        }
    }
    let eof = tokens.last().map_or_else(
        || PhysicalOriginV2::ui_token(Span::call_site()),
        |token| token.physical,
    );
    Ok((tokens, eof))
}

fn push(tokens: &mut Vec<Token>, token: Token, limits: Limits) -> Result<(), Diagnostic> {
    if tokens.len() >= limits.tokens {
        return Err(Diagnostic::new(
            "authoring limit exceeded: tokens",
            token.physical,
        ));
    }
    tokens.push(token);
    Ok(())
}
