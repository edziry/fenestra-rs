mod properties;
mod support;

use std::collections::BTreeSet;

use crate::source_v2::PhysicalOriginV2;
use crate::token::{AbstractToken, AbstractTokenKind, Punctuation};

use super::{Diagnostic, Limits};

type Token = AbstractToken<PhysicalOriginV2>;

pub(super) struct Document {
    pub(super) name: Box<str>,
    pub(super) origin: PhysicalOriginV2,
    pub(super) elements: Vec<Element>,
}

pub(super) struct Element {
    pub(super) kind: Kind,
    pub(super) name: Box<str>,
    pub(super) properties: Properties,
    pub(super) children: Vec<usize>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Kind {
    Row,
    Column,
    Rect,
}

impl Kind {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Row => "row",
            Self::Column => "column",
            Self::Rect => "rect",
        }
    }
}

#[derive(Default)]
pub(super) struct Properties {
    pub(super) width: Option<i32>,
    pub(super) height: Option<i32>,
    pub(super) padding: Option<i32>,
    pub(super) gap: Option<i32>,
    pub(super) background: Option<[u8; 4]>,
    pub(super) input: Option<bool>,
    seen: u8,
}

pub(super) fn parse(
    tokens: Vec<Token>,
    eof: PhysicalOriginV2,
    limits: Limits,
) -> Result<Document, Diagnostic> {
    Parser {
        tokens,
        next: 0,
        eof,
        limits,
        names: BTreeSet::new(),
    }
    .document()
}

struct Parser {
    tokens: Vec<Token>,
    next: usize,
    eof: PhysicalOriginV2,
    limits: Limits,
    names: BTreeSet<Box<str>>,
}

impl Parser {
    fn document(mut self) -> Result<Document, Diagnostic> {
        let origin = self.keyword("format")?.physical;
        let format = self.take()?;
        if !matches!(&format.kind, AbstractTokenKind::UnsignedDecimal(value) if &**value == "3") {
            return Err(Diagnostic::new(
                "expected authoring format 3",
                format.physical,
            ));
        }
        self.punctuation(Punctuation::Semicolon)?;
        self.keyword("view")?;
        let (name, _) = self.name()?;
        self.punctuation(Punctuation::OpenBrace)?;
        let mut elements = vec![self.element(0)?];
        let mut pending = vec![0];
        while let Some(&current) = pending.last() {
            if self.matches(Punctuation::CloseBrace) {
                self.take()?;
                pending.pop();
            } else if self.starts_element() {
                if elements[current].kind == Kind::Rect {
                    return Err(self.error("rect cannot contain children; use row or column"));
                }
                let index = elements.len();
                let child = self.element(index)?;
                elements.push(child);
                elements[current].children.push(index);
                pending.push(index);
            } else {
                self.property(&mut elements[current])?;
            }
        }
        self.punctuation(Punctuation::CloseBrace)?;
        if self.next != self.tokens.len() {
            return Err(self.error("unexpected input after view; expected end of source"));
        }
        Ok(Document {
            name,
            origin,
            elements,
        })
    }

    fn element(&mut self, count: usize) -> Result<Element, Diagnostic> {
        if count >= self.limits.elements {
            return Err(self.error("authoring limit exceeded: elements"));
        }
        let token = self.take()?;
        let kind = match token.label() {
            "row" => Kind::Row,
            "column" => Kind::Column,
            "rect" => Kind::Rect,
            _ => {
                return Err(Diagnostic::new(
                    "unknown element; expected row, column, or rect",
                    token.physical,
                ));
            }
        };
        let (name, physical) = self.name()?;
        if !self.names.insert(name.clone()) {
            return Err(Diagnostic::new("duplicate element name", physical));
        }
        self.punctuation(Punctuation::OpenBrace)?;
        Ok(Element {
            kind,
            name,
            properties: Properties::default(),
            children: Vec::new(),
        })
    }

    fn starts_element(&self) -> bool {
        let Some(token) = self.tokens.get(self.next) else {
            return false;
        };
        if matches!(
            token.label(),
            "width" | "height" | "padding" | "gap" | "background" | "input"
        ) {
            return false;
        }
        matches!(
            self.tokens.get(self.next + 1).map(|token| &token.kind),
            Some(AbstractTokenKind::Identifier(_))
        )
    }
}
