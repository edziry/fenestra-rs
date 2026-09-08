use super::TextError;
use crate::Color;

/// Integer typography and foreground color for a fixed-size text element.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextStyle {
    font_size: u32,
    line_height: u32,
    color: Color,
}

impl TextStyle {
    /// Uses 16-pixel glyph size, 24-pixel line height, and opaque white text.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            font_size: 16,
            line_height: 24,
            color: Color::rgba8(255, 255, 255, 255),
        }
    }

    /// Sets glyph size in pixels; accepted values are 1 through 512.
    #[must_use]
    pub const fn font_size(mut self, value: u32) -> Self {
        self.font_size = value;
        self
    }

    /// Sets absolute line height in pixels; accepted values are 1 through 2048.
    #[must_use]
    pub const fn line_height(mut self, value: u32) -> Self {
        self.line_height = value;
        self
    }

    /// Sets the straight RGBA8 foreground color.
    #[must_use]
    pub const fn color(mut self, value: Color) -> Self {
        self.color = value;
        self
    }

    /// Returns glyph size in pixels.
    #[must_use]
    pub const fn font_size_value(self) -> u32 {
        self.font_size
    }

    /// Returns absolute line height in pixels.
    #[must_use]
    pub const fn line_height_value(self) -> u32 {
        self.line_height
    }

    /// Returns the straight foreground color.
    #[must_use]
    pub const fn color_value(self) -> Color {
        self.color
    }

    pub(crate) fn validate(self) -> Result<(), TextError> {
        for (property, value, maximum) in [
            ("font_size", self.font_size, 512),
            ("line_height", self.line_height, 2048),
        ] {
            if value == 0 || value > maximum {
                return Err(TextError::InvalidStyle { property, value });
            }
        }
        Ok(())
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self::new()
    }
}
