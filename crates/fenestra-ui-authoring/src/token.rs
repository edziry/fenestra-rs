use crate::source::PhysicalOriginV1;

#[derive(Clone)]
pub(crate) struct AbstractToken<O> {
    pub(crate) kind: AbstractTokenKind,
    pub(crate) physical: O,
}

impl<O> AbstractToken<O> {
    pub(crate) fn label(&self) -> &str {
        match &self.kind {
            AbstractTokenKind::Identifier(label) | AbstractTokenKind::UnsignedDecimal(label) => {
                label
            }
            AbstractTokenKind::Punctuation(punctuation) => punctuation.label(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AbstractTokenKind {
    Identifier(Box<str>),
    UnsignedDecimal(Box<str>),
    Punctuation(Punctuation),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Punctuation {
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    OpenParenthesis,
    CloseParenthesis,
    Colon,
    Semicolon,
    Comma,
    Equals,
    Dot,
    Minus,
}

impl Punctuation {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::OpenBrace => "{",
            Self::CloseBrace => "}",
            Self::OpenBracket => "[",
            Self::CloseBracket => "]",
            Self::OpenParenthesis => "(",
            Self::CloseParenthesis => ")",
            Self::Colon => ":",
            Self::Semicolon => ";",
            Self::Comma => ",",
            Self::Equals => "=",
            Self::Dot => ".",
            Self::Minus => "-",
        }
    }
}

pub(crate) type AbstractTokenV1 = AbstractToken<PhysicalOriginV1>;
pub(crate) type AbstractTokenKindV1 = AbstractTokenKind;
pub(crate) type PunctuationV1 = Punctuation;
