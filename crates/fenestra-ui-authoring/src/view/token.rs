use crate::source_v2::PhysicalOriginV2;
use crate::token::{AbstractToken, AbstractTokenKind, Punctuation};

#[derive(Clone)]
pub(super) struct Token {
    pub(super) kind: Kind,
    pub(super) physical: PhysicalOriginV2,
}

#[derive(Clone, Eq, PartialEq)]
pub(super) enum Kind {
    Identifier(Box<str>),
    UnsignedDecimal(Box<str>),
    Punctuation(Punctuation),
    String(Box<str>),
}

impl Token {
    pub(super) fn label(&self) -> &str {
        match &self.kind {
            Kind::Identifier(value) | Kind::UnsignedDecimal(value) => value,
            Kind::Punctuation(value) => value.label(),
            Kind::String(_) => "string literal",
        }
    }
}

impl From<AbstractToken<PhysicalOriginV2>> for Token {
    fn from(token: AbstractToken<PhysicalOriginV2>) -> Self {
        let kind = match token.kind {
            AbstractTokenKind::Identifier(value) => Kind::Identifier(value),
            AbstractTokenKind::UnsignedDecimal(value) => Kind::UnsignedDecimal(value),
            AbstractTokenKind::Punctuation(value) => Kind::Punctuation(value),
        };
        Self {
            kind,
            physical: token.physical,
        }
    }
}
