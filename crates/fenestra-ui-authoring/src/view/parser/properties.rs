use super::super::token::Kind as AbstractTokenKind;
use crate::token::Punctuation;

use super::{Diagnostic, Element, Kind, Parser};

impl Parser {
    pub(super) fn property(&mut self, element: &mut Element) -> Result<(), Diagnostic> {
        let token = self.take()?;
        let property = token.label();
        let bit = match property {
            "width" => 1,
            "height" => 2,
            "padding" => 4,
            "gap" => 8,
            "background" => 16,
            "input" => 32,
            "content" => 64,
            "font_size" => 128,
            "line_height" => 256,
            "color" => 512,
            _ => {
                return Err(Diagnostic::new(
                    "unknown property; expected width, height, padding, gap, background, or input",
                    token.physical,
                ));
            }
        };
        if element.properties.seen & bit != 0 {
            return Err(Diagnostic::new("duplicate property", token.physical));
        }
        if matches!(element.kind, Kind::Rect | Kind::Text) && matches!(property, "padding" | "gap")
        {
            return Err(Diagnostic::new(
                "padding and gap require a row or column container",
                token.physical,
            ));
        }
        if element.kind != Kind::Text
            && matches!(property, "content" | "font_size" | "line_height" | "color")
        {
            return Err(Diagnostic::new(
                "property requires a text element",
                token.physical,
            ));
        }
        self.punctuation(Punctuation::Colon)?;
        let props = &mut element.properties;
        match property {
            "width" => props.width = Some(self.dimension()?),
            "height" => props.height = Some(self.dimension()?),
            "padding" => props.padding = Some(self.dimension()?),
            "gap" => props.gap = Some(self.dimension()?),
            "background" => props.background = Some(self.color()?),
            "input" => props.input = Some(self.input()?),
            "content" => props.content = Some(self.content()?),
            "font_size" => props.font_size = Some(self.positive(512)?),
            "line_height" => props.line_height = Some(self.positive(2048)?),
            "color" => props.color = Some(self.color()?),
            _ => unreachable!("property name was validated"),
        }
        props.seen |= bit;
        self.punctuation(Punctuation::Semicolon)?;
        Ok(())
    }

    fn content(&mut self) -> Result<Box<str>, Diagnostic> {
        let token = self.take()?;
        match token.kind {
            AbstractTokenKind::String(value) => Ok(value),
            _ => Err(Diagnostic::new(
                "expected a Rust string literal",
                token.physical,
            )),
        }
    }

    fn positive(&mut self, maximum: u32) -> Result<u32, Diagnostic> {
        let token = self.take()?;
        if let AbstractTokenKind::UnsignedDecimal(value) = &token.kind
            && let Ok(value) = value.parse::<u32>()
            && (1..=maximum).contains(&value)
        {
            return Ok(value);
        }
        Err(Diagnostic::new(
            format!("expected a positive integer from 1 through {maximum}"),
            token.physical,
        ))
    }

    fn dimension(&mut self) -> Result<i32, Diagnostic> {
        let token = self.take()?;
        if let AbstractTokenKind::UnsignedDecimal(value) = &token.kind
            && let Ok(value) = value.parse::<i32>()
        {
            return Ok(value);
        }
        Err(Diagnostic::new(
            "expected a nonnegative integer from 0 through 2147483647",
            token.physical,
        ))
    }

    fn channel(&mut self) -> Result<u8, Diagnostic> {
        let token = self.take()?;
        if let AbstractTokenKind::UnsignedDecimal(value) = &token.kind
            && let Ok(value) = value.parse::<u8>()
        {
            return Ok(value);
        }
        Err(Diagnostic::new(
            "expected an rgba8 channel from 0 through 255",
            token.physical,
        ))
    }

    fn color(&mut self) -> Result<[u8; 4], Diagnostic> {
        self.keyword("rgba8")?;
        self.punctuation(Punctuation::OpenParenthesis)?;
        let red = self.channel()?;
        self.punctuation(Punctuation::Comma)?;
        let green = self.channel()?;
        self.punctuation(Punctuation::Comma)?;
        let blue = self.channel()?;
        self.punctuation(Punctuation::Comma)?;
        let alpha = self.channel()?;
        self.punctuation(Punctuation::CloseParenthesis)?;
        Ok([red, green, blue, alpha])
    }

    fn input(&mut self) -> Result<bool, Diagnostic> {
        let token = self.take()?;
        match token.label() {
            "accept" => Ok(true),
            "ignore" => Ok(false),
            _ => Err(Diagnostic::new(
                "expected input policy accept or ignore",
                token.physical,
            )),
        }
    }
}
