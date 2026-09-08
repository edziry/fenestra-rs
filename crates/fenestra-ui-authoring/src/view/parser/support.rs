use super::super::token::Kind as AbstractTokenKind;
use crate::source_v2::PhysicalOriginV2;
use crate::token::Punctuation;

use super::{Diagnostic, Parser, Token};

impl Parser {
    pub(super) fn name(&mut self) -> Result<(Box<str>, PhysicalOriginV2), Diagnostic> {
        let token = self.take()?;
        match token.kind {
            AbstractTokenKind::Identifier(name) => Ok((name, token.physical)),
            _ => Err(Diagnostic::new(
                "expected an ASCII identifier",
                token.physical,
            )),
        }
    }

    pub(super) fn keyword(&mut self, expected: &str) -> Result<Token, Diagnostic> {
        let token = self.take()?;
        if matches!(&token.kind, AbstractTokenKind::Identifier(name) if &**name == expected) {
            Ok(token)
        } else {
            Err(Diagnostic::new(
                format!("expected {expected}"),
                token.physical,
            ))
        }
    }

    pub(super) fn punctuation(&mut self, expected: Punctuation) -> Result<Token, Diagnostic> {
        let token = self.take()?;
        if token.kind == AbstractTokenKind::Punctuation(expected) {
            Ok(token)
        } else {
            Err(Diagnostic::new(
                format!("expected '{}'", expected.label()),
                token.physical,
            ))
        }
    }

    pub(super) fn matches(&self, expected: Punctuation) -> bool {
        matches!(
            self.tokens.get(self.next).map(|token| &token.kind),
            Some(AbstractTokenKind::Punctuation(actual)) if *actual == expected
        )
    }

    pub(super) fn error(&self, message: &str) -> Diagnostic {
        let origin = self
            .tokens
            .get(self.next)
            .map_or(self.eof, |token| token.physical);
        Diagnostic::new(message, origin)
    }

    pub(super) fn take(&mut self) -> Result<Token, Diagnostic> {
        let Some(token) = self.tokens.get(self.next).cloned() else {
            return Err(self.error("unexpected end of source; the view is incomplete"));
        };
        self.next += 1;
        Ok(token)
    }
}
