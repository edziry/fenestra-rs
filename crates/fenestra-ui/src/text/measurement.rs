use super::{TextError, TextLimits, TextStyle};

/// A validated text measurement request without a raster or pixel budget.
///
/// An absent width measures max-content: hard line breaks are preserved but
/// soft wrapping is disabled. A supplied width enables wrapping, including
/// zero; a glyph wider than the available width may still extend beyond it.
#[derive(Clone, Copy, Debug)]
pub struct TextMeasureRequest<'a> {
    text: &'a str,
    style: TextStyle,
    width: Option<u32>,
    limits: TextLimits,
}

impl<'a> TextMeasureRequest<'a> {
    /// Validates UTF-8 byte count, typography and an optional layout width.
    ///
    /// Width must fit the signed pixel coordinate domain. Text and glyph
    /// limits apply; the raster pixel limit does not apply to measurement.
    pub fn new(
        text: &'a str,
        style: TextStyle,
        width: Option<u32>,
        limits: TextLimits,
    ) -> Result<Self, TextError> {
        super::validate_content(text, style, limits)?;
        if let Some(width) = width
            && width > i32::MAX as u32
        {
            return Err(TextError::InvalidMeasurementWidth { width });
        }
        Ok(Self {
            text,
            style,
            width,
            limits,
        })
    }

    /// Borrows the exact UTF-8 content without normalization.
    #[must_use]
    pub const fn text(self) -> &'a str {
        self.text
    }

    /// Returns the requested typography and foreground color.
    #[must_use]
    pub const fn style(self) -> TextStyle {
        self.style
    }

    /// Returns the wrap width, or no width for max-content measurement.
    #[must_use]
    pub const fn width(self) -> Option<u32> {
        self.width
    }

    /// Returns the inclusive text byte and glyph bounds.
    #[must_use]
    pub const fn limits(self) -> TextLimits {
        self.limits
    }
}
