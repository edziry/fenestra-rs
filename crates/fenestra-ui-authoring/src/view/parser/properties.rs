use crate::token::{AbstractTokenKind, Punctuation};

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
        if element.kind == Kind::Rect && matches!(property, "padding" | "gap") {
            return Err(Diagnostic::new(
                "padding and gap require a row or column container",
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
            _ => unreachable!("property name was validated"),
        }
        props.seen |= bit;
        self.punctuation(Punctuation::Semicolon)?;
        Ok(())
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
