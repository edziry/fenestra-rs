use proc_macro2::token_stream::IntoIter;
use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};

use crate::diagnostic_v2::{AuthoringDiagnosticKindV2, AuthoringDiagnosticV2};
use crate::limits_v2::{AuthoringLimitKindV2, AuthoringLimitsV2};
use crate::parsed_v2::ParsedDocumentV2;
use crate::parser_v2::parse_document_v2;
use crate::source_v2::{DiagnosticLocationV2, PhysicalOriginV2};
use crate::token::{AbstractToken, AbstractTokenKind, Punctuation};
use crate::vocabulary_v2::AuthoringFrontendV2;

pub(crate) fn parse_ui_document_v2(
    stream: TokenStream,
    limits: AuthoringLimitsV2,
) -> Result<ParsedDocumentV2, AuthoringDiagnosticV2> {
    let (tokens, eof) = UiTokenAdapterV2::new(limits).adapt(stream)?;
    parse_document_v2(AuthoringFrontendV2::UiMacro, eof, tokens, limits)
}

struct UiTokenAdapterV2 {
    limits: AuthoringLimitsV2,
    tokens: Vec<AbstractToken<PhysicalOriginV2>>,
    last_origin: Option<PhysicalOriginV2>,
}

impl UiTokenAdapterV2 {
    const fn new(limits: AuthoringLimitsV2) -> Self {
        Self {
            limits,
            tokens: Vec::new(),
            last_origin: None,
        }
    }

    fn adapt(
        mut self,
        stream: TokenStream,
    ) -> Result<(Vec<AbstractToken<PhysicalOriginV2>>, PhysicalOriginV2), AuthoringDiagnosticV2>
    {
        let mut frames = vec![TokenFrameV2::root(stream)];
        while !frames.is_empty() {
            let next = frames.last_mut().and_then(|frame| frame.tokens.next());
            if let Some(tree) = next {
                let depth = frames.last().map_or(0, |frame| frame.depth);
                self.adapt_tree(tree, depth, &mut frames)?;
            } else if let Some(frame) = frames.pop()
                && let Some((punctuation, span)) = frame.closing
            {
                self.push(
                    AbstractTokenKind::Punctuation(punctuation),
                    PhysicalOriginV2::ui_token(span),
                )?;
            }
        }
        let eof = self
            .last_origin
            .unwrap_or_else(|| PhysicalOriginV2::ui_token(Span::call_site()));
        Ok((self.tokens, eof))
    }

    fn adapt_tree(
        &mut self,
        tree: TokenTree,
        depth: usize,
        frames: &mut Vec<TokenFrameV2>,
    ) -> Result<(), AuthoringDiagnosticV2> {
        match tree {
            TokenTree::Group(group) => self.open_group(group, depth, frames),
            TokenTree::Ident(ident) => {
                let span = ident.span();
                let label = ident.to_string();
                if !is_identifier(&label) {
                    return Err(unsupported(span));
                }
                self.push(
                    AbstractTokenKind::Identifier(label.into()),
                    PhysicalOriginV2::ui_token(span),
                )
            }
            TokenTree::Literal(literal) => {
                let span = literal.span();
                let label = literal.to_string();
                if label.is_empty() || !label.bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err(unsupported(span));
                }
                self.push(
                    AbstractTokenKind::UnsignedDecimal(label.into()),
                    PhysicalOriginV2::ui_token(span),
                )
            }
            TokenTree::Punct(punctuation) => {
                let span = punctuation.span();
                let Some(kind) = punctuation_v2(punctuation.as_char()) else {
                    return Err(unsupported(span));
                };
                self.push(
                    AbstractTokenKind::Punctuation(kind),
                    PhysicalOriginV2::ui_token(span),
                )
            }
        }
    }

    fn open_group(
        &mut self,
        group: Group,
        depth: usize,
        frames: &mut Vec<TokenFrameV2>,
    ) -> Result<(), AuthoringDiagnosticV2> {
        let Some((opening, closing)) = delimiters(group.delimiter()) else {
            return Err(unsupported(group.span()));
        };
        let next_depth = self.push_opening(
            AbstractTokenKind::Punctuation(opening),
            PhysicalOriginV2::ui_token(group.span_open()),
            depth,
        )?;
        frames.push(TokenFrameV2::group(group, closing, next_depth));
        Ok(())
    }

    fn push(
        &mut self,
        kind: AbstractTokenKind,
        physical: PhysicalOriginV2,
    ) -> Result<(), AuthoringDiagnosticV2> {
        self.preflight_token(&kind, physical)?;
        self.retain(kind, physical);
        Ok(())
    }

    fn push_opening(
        &mut self,
        kind: AbstractTokenKind,
        physical: PhysicalOriginV2,
        depth: usize,
    ) -> Result<usize, AuthoringDiagnosticV2> {
        self.preflight_token(&kind, physical)?;
        let next_depth = depth.checked_add(1).ok_or_else(|| {
            failure(
                AuthoringDiagnosticKindV2::LimitExceeded(AuthoringLimitKindV2::NestingDepth),
                physical,
            )
        })?;
        if next_depth > self.limits.limit(AuthoringLimitKindV2::NestingDepth) {
            return Err(failure(
                AuthoringDiagnosticKindV2::LimitExceeded(AuthoringLimitKindV2::NestingDepth),
                physical,
            ));
        }
        self.retain(kind, physical);
        Ok(next_depth)
    }

    fn preflight_token(
        &self,
        kind: &AbstractTokenKind,
        physical: PhysicalOriginV2,
    ) -> Result<(), AuthoringDiagnosticV2> {
        if self.tokens.len() >= self.limits.limit(AuthoringLimitKindV2::Tokens) {
            return Err(failure(
                AuthoringDiagnosticKindV2::LimitExceeded(AuthoringLimitKindV2::Tokens),
                physical,
            ));
        }
        if let AbstractTokenKind::Identifier(label) = kind
            && label.len() > self.limits.limit(AuthoringLimitKindV2::IdentifierBytes)
        {
            return Err(failure(
                AuthoringDiagnosticKindV2::LimitExceeded(AuthoringLimitKindV2::IdentifierBytes),
                physical,
            ));
        }
        Ok(())
    }

    fn retain(&mut self, kind: AbstractTokenKind, physical: PhysicalOriginV2) {
        self.last_origin = Some(physical);
        self.tokens.push(AbstractToken { kind, physical });
    }
}

struct TokenFrameV2 {
    tokens: IntoIter,
    closing: Option<(Punctuation, Span)>,
    depth: usize,
}

impl TokenFrameV2 {
    fn root(stream: TokenStream) -> Self {
        Self {
            tokens: stream.into_iter(),
            closing: None,
            depth: 0,
        }
    }

    fn group(group: Group, closing: Punctuation, depth: usize) -> Self {
        let span = group.span_close();
        Self {
            tokens: group.stream().into_iter(),
            closing: Some((closing, span)),
            depth,
        }
    }
}

const fn delimiters(delimiter: Delimiter) -> Option<(Punctuation, Punctuation)> {
    Some(match delimiter {
        Delimiter::Brace => (Punctuation::OpenBrace, Punctuation::CloseBrace),
        Delimiter::Bracket => (Punctuation::OpenBracket, Punctuation::CloseBracket),
        Delimiter::Parenthesis => (Punctuation::OpenParenthesis, Punctuation::CloseParenthesis),
        Delimiter::None => return None,
    })
}

const fn punctuation_v2(character: char) -> Option<Punctuation> {
    Some(match character {
        ':' => Punctuation::Colon,
        ';' => Punctuation::Semicolon,
        ',' => Punctuation::Comma,
        '=' => Punctuation::Equals,
        '.' => Punctuation::Dot,
        '-' => Punctuation::Minus,
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

fn unsupported(span: Span) -> AuthoringDiagnosticV2 {
    failure(
        AuthoringDiagnosticKindV2::UnsupportedToken,
        PhysicalOriginV2::ui_token(span),
    )
}

fn failure(kind: AuthoringDiagnosticKindV2, physical: PhysicalOriginV2) -> AuthoringDiagnosticV2 {
    AuthoringDiagnosticV2::new(
        AuthoringFrontendV2::UiMacro,
        kind,
        DiagnosticLocationV2::Physical(physical),
    )
}
