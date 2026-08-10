#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AbstractTokenV1 {
    pub(crate) kind: AbstractTokenKindV1,
    pub(crate) start: u32,
    pub(crate) end: u32,
}

impl AbstractTokenV1 {
    pub(crate) fn label(&self) -> &str {
        match &self.kind {
            AbstractTokenKindV1::Identifier(label)
            | AbstractTokenKindV1::UnsignedDecimal(label) => label,
            AbstractTokenKindV1::Punctuation(punctuation) => punctuation.label(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AbstractTokenKindV1 {
    Identifier(Box<str>),
    UnsignedDecimal(Box<str>),
    Punctuation(PunctuationV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PunctuationV1 {
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

impl PunctuationV1 {
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
