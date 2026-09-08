use super::super::token::Kind as AbstractTokenKind;
use crate::token::Punctuation;

use super::{Diagnostic, Dimension, Element, Kind, Parser};

impl Parser {
    pub(super) fn property(
        &mut self,
        element: &mut Element,
        scope: Option<Kind>,
    ) -> Result<(), Diagnostic> {
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
            "min_width" => 1024,
            "max_width" => 2048,
            "min_height" => 4096,
            "max_height" => 8192,
            "label" => 1 << 14,
            "disabled" => 1 << 15,
            "checked" => 1 << 16,
            "hover_background" => 1 << 17,
            "pressed_background" => 1 << 18,
            "checked_background" => 1 << 19,
            "disabled_background" => 1 << 20,
            "checked_color" => 1 << 21,
            "disabled_color" => 1 << 22,
            "focus_color" => 1 << 23,
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
        self.control_property(element, property, scope, token.physical)?;
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
            "width" => props.width = Some(self.extent()?),
            "height" => props.height = Some(self.extent()?),
            "min_width" => props.min_width = Some(self.dimension()?),
            "max_width" => props.max_width = Some(self.dimension()?),
            "min_height" => props.min_height = Some(self.dimension()?),
            "max_height" => props.max_height = Some(self.dimension()?),
            "padding" => props.padding = Some(self.dimension()?),
            "gap" => props.gap = Some(self.dimension()?),
            "background" => props.background = Some(self.color()?),
            "input" => props.input = Some(self.input()?),
            "content" => props.content = Some(self.content()?),
            "font_size" => props.font_size = Some(self.positive(512)?),
            "line_height" => props.line_height = Some(self.positive(2048)?),
            "color" => props.color = Some(self.color()?),
            "label" => props.label = Some(self.label()?),
            "disabled" => props.disabled = Some(self.boolean()?),
            "checked" => props.checked = Some(self.boolean()?),
            "hover_background" => props.state_style.hover_background = Some(self.color()?),
            "pressed_background" => props.state_style.pressed_background = Some(self.color()?),
            "checked_background" => props.state_style.checked_background = Some(self.color()?),
            "disabled_background" => props.state_style.disabled_background = Some(self.color()?),
            "checked_color" => props.state_style.checked_color = Some(self.color()?),
            "disabled_color" => props.state_style.disabled_color = Some(self.color()?),
            "focus_color" => props.state_style.focus_color = Some(self.color()?),
            _ => unreachable!("property name was validated"),
        }
        if property == "input" && props.input == Some(true) && scope.is_some() {
            return Err(Diagnostic::new(
                "input accept is not supported inside a control",
                token.physical,
            ));
        }
        for (axis, minimum, maximum) in [
            ("width", props.min_width, props.max_width),
            ("height", props.min_height, props.max_height),
        ] {
            if let (Some(minimum), Some(maximum)) = (minimum, maximum)
                && minimum > maximum
            {
                return Err(Diagnostic::new(
                    format!("minimum {axis} exceeds maximum {axis}"),
                    token.physical,
                ));
            }
        }
        props.seen |= bit;
        self.punctuation(Punctuation::Semicolon)?;
        Ok(())
    }

    fn extent(&mut self) -> Result<Dimension, Diagnostic> {
        match self.tokens.get(self.next).map(|token| token.label()) {
            Some("auto") => {
                self.take()?;
                Ok(Dimension::Auto)
            }
            Some("fill") => {
                self.take()?;
                let weight = if self.matches(Punctuation::OpenParenthesis) {
                    self.take()?;
                    let weight = self.positive(65535)?;
                    self.punctuation(Punctuation::CloseParenthesis)?;
                    weight
                } else {
                    1
                };
                Ok(Dimension::Fill(weight))
            }
            _ => self.dimension().map(Dimension::Px),
        }
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

    fn label(&mut self) -> Result<Box<str>, Diagnostic> {
        let physical = self
            .tokens
            .get(self.next)
            .map_or(self.eof, |token| token.physical);
        let label = self.content()?;
        if label.trim().is_empty() {
            return Err(Diagnostic::new("control label must be nonempty", physical));
        }
        Ok(label)
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
