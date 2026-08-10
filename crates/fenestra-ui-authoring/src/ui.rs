use proc_macro2::token_stream::IntoIter;
use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};

use crate::diagnostic::{AuthoringDiagnosticKindV1, AuthoringDiagnosticV1};
use crate::limits::{AuthoringLimitKindV1, AuthoringLimitsV1};
use crate::parsed::ParsedDocumentV1;
use crate::parser::parse_document_v1;
use crate::source::{DiagnosticLocationV1, PhysicalOriginV1};
use crate::token::{AbstractTokenKindV1, AbstractTokenV1, PunctuationV1};
use crate::vocabulary::AuthoringFrontendV1;

pub(crate) fn parse_ui_document_v1(
    stream: TokenStream,
    limits: AuthoringLimitsV1,
) -> Result<ParsedDocumentV1, AuthoringDiagnosticV1> {
    let (tokens, eof) = UiTokenAdapterV1::new(limits).adapt(stream)?;
    parse_document_v1(AuthoringFrontendV1::UiMacro, eof, tokens, limits)
}

struct UiTokenAdapterV1 {
    limits: AuthoringLimitsV1,
    tokens: Vec<AbstractTokenV1>,
    last_origin: Option<PhysicalOriginV1>,
}

impl UiTokenAdapterV1 {
    const fn new(limits: AuthoringLimitsV1) -> Self {
        Self {
            limits,
            tokens: Vec::new(),
            last_origin: None,
        }
    }

    fn adapt(
        mut self,
        stream: TokenStream,
    ) -> Result<(Vec<AbstractTokenV1>, PhysicalOriginV1), AuthoringDiagnosticV1> {
        let mut frames = vec![TokenFrameV1::root(stream)];
        while !frames.is_empty() {
            let next = frames.last_mut().and_then(|frame| frame.tokens.next());
            if let Some(tree) = next {
                let depth = frames.last().map_or(0, |frame| frame.depth);
                self.adapt_tree(tree, depth, &mut frames)?;
            } else {
                if let Some(frame) = frames.pop()
                    && let Some((punctuation, span)) = frame.closing
                {
                    self.push(
                        AbstractTokenKindV1::Punctuation(punctuation),
                        PhysicalOriginV1::ui_token(span),
                    )?;
                }
            }
        }

        let eof = self
            .last_origin
            .unwrap_or_else(|| PhysicalOriginV1::ui_token(Span::call_site()));
        Ok((self.tokens, eof))
    }

    fn adapt_tree(
        &mut self,
        tree: TokenTree,
        depth: usize,
        frames: &mut Vec<TokenFrameV1>,
    ) -> Result<(), AuthoringDiagnosticV1> {
        match tree {
            TokenTree::Group(group) => self.open_group(group, depth, frames),
            TokenTree::Ident(ident) => {
                let span = ident.span();
                let label = ident.to_string();
                if !is_identifier(&label) {
                    return Err(unsupported(span));
                }
                self.push(
                    AbstractTokenKindV1::Identifier(label.into()),
                    PhysicalOriginV1::ui_token(span),
                )
            }
            TokenTree::Literal(literal) => {
                let span = literal.span();
                let label = literal.to_string();
                if label.is_empty() || !label.bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err(unsupported(span));
                }
                self.push(
                    AbstractTokenKindV1::UnsignedDecimal(label.into()),
                    PhysicalOriginV1::ui_token(span),
                )
            }
            TokenTree::Punct(punctuation) => {
                let span = punctuation.span();
                let Some(kind) = punctuation_v1(punctuation.as_char()) else {
                    return Err(unsupported(span));
                };
                self.push(
                    AbstractTokenKindV1::Punctuation(kind),
                    PhysicalOriginV1::ui_token(span),
                )
            }
        }
    }

    fn open_group(
        &mut self,
        group: Group,
        depth: usize,
        frames: &mut Vec<TokenFrameV1>,
    ) -> Result<(), AuthoringDiagnosticV1> {
        let Some((opening, closing)) = delimiters(group.delimiter()) else {
            return Err(unsupported(group.span()));
        };
        let next_depth = self.push_opening(
            AbstractTokenKindV1::Punctuation(opening),
            PhysicalOriginV1::ui_token(group.span_open()),
            depth,
        )?;
        frames.push(TokenFrameV1::group(group, closing, next_depth));
        Ok(())
    }

    fn push(
        &mut self,
        kind: AbstractTokenKindV1,
        physical: PhysicalOriginV1,
    ) -> Result<(), AuthoringDiagnosticV1> {
        self.preflight_token(&kind, physical)?;
        self.retain(kind, physical);
        Ok(())
    }

    fn push_opening(
        &mut self,
        kind: AbstractTokenKindV1,
        physical: PhysicalOriginV1,
        depth: usize,
    ) -> Result<usize, AuthoringDiagnosticV1> {
        self.preflight_token(&kind, physical)?;
        let next_depth = depth.checked_add(1).ok_or_else(|| {
            failure(
                AuthoringDiagnosticKindV1::LimitExceeded(AuthoringLimitKindV1::NestingDepth),
                physical,
            )
        })?;
        if next_depth > self.limits.limit(AuthoringLimitKindV1::NestingDepth) {
            return Err(failure(
                AuthoringDiagnosticKindV1::LimitExceeded(AuthoringLimitKindV1::NestingDepth),
                physical,
            ));
        }
        self.retain(kind, physical);
        Ok(next_depth)
    }

    fn preflight_token(
        &self,
        kind: &AbstractTokenKindV1,
        physical: PhysicalOriginV1,
    ) -> Result<(), AuthoringDiagnosticV1> {
        if self.tokens.len() >= self.limits.limit(AuthoringLimitKindV1::Tokens) {
            return Err(failure(
                AuthoringDiagnosticKindV1::LimitExceeded(AuthoringLimitKindV1::Tokens),
                physical,
            ));
        }
        if let AbstractTokenKindV1::Identifier(label) = &kind
            && label.len() > self.limits.limit(AuthoringLimitKindV1::IdentifierBytes)
        {
            return Err(failure(
                AuthoringDiagnosticKindV1::LimitExceeded(AuthoringLimitKindV1::IdentifierBytes),
                physical,
            ));
        }
        Ok(())
    }

    fn retain(&mut self, kind: AbstractTokenKindV1, physical: PhysicalOriginV1) {
        self.last_origin = Some(physical);
        self.tokens.push(AbstractTokenV1 { kind, physical });
    }
}

struct TokenFrameV1 {
    tokens: IntoIter,
    closing: Option<(PunctuationV1, Span)>,
    depth: usize,
}

impl TokenFrameV1 {
    fn root(stream: TokenStream) -> Self {
        Self {
            tokens: stream.into_iter(),
            closing: None,
            depth: 0,
        }
    }

    fn group(group: Group, closing: PunctuationV1, depth: usize) -> Self {
        let span = group.span_close();
        Self {
            tokens: group.stream().into_iter(),
            closing: Some((closing, span)),
            depth,
        }
    }
}

const fn delimiters(delimiter: Delimiter) -> Option<(PunctuationV1, PunctuationV1)> {
    Some(match delimiter {
        Delimiter::Brace => (PunctuationV1::OpenBrace, PunctuationV1::CloseBrace),
        Delimiter::Bracket => (PunctuationV1::OpenBracket, PunctuationV1::CloseBracket),
        Delimiter::Parenthesis => (
            PunctuationV1::OpenParenthesis,
            PunctuationV1::CloseParenthesis,
        ),
        Delimiter::None => return None,
    })
}

const fn punctuation_v1(character: char) -> Option<PunctuationV1> {
    Some(match character {
        ':' => PunctuationV1::Colon,
        ';' => PunctuationV1::Semicolon,
        ',' => PunctuationV1::Comma,
        '=' => PunctuationV1::Equals,
        '.' => PunctuationV1::Dot,
        '-' => PunctuationV1::Minus,
        _ => return None,
    })
}

fn is_identifier(label: &str) -> bool {
    let mut bytes = label.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn unsupported(span: Span) -> AuthoringDiagnosticV1 {
    failure(
        AuthoringDiagnosticKindV1::UnsupportedToken,
        PhysicalOriginV1::ui_token(span),
    )
}

fn failure(kind: AuthoringDiagnosticKindV1, physical: PhysicalOriginV1) -> AuthoringDiagnosticV1 {
    AuthoringDiagnosticV1::new(
        AuthoringFrontendV1::UiMacro,
        kind,
        DiagnosticLocationV1::Physical(physical),
    )
}
