//! Owned text requests, output and resource bounds, independent of candidates.

mod error;
mod limits;
mod measurement;
mod output;
mod style;

use crate::{Error, Size};

pub use error::TextError;
pub use limits::TextLimits;
pub use measurement::TextMeasureRequest;
pub use output::{TextLayout, TextMetrics};
pub use style::TextStyle;

/// An application-owned shaping and raster adapter with private caches.
///
/// Implementations must honor request limits and return validated measurements
/// or owned premultiplied pixels. Cache changes on failure are allowed;
/// application state is committed only after output validation and spatial
/// preparation succeed.
pub trait TextEngine {
    /// Measures complete text without allocating a raster.
    ///
    /// The default preserves adapters that support only explicitly sized text.
    /// Implementations must validate their output with
    /// [`TextMetrics::validate_measurement`] before returning it.
    fn measure(&mut self, _request: TextMeasureRequest<'_>) -> Result<TextMetrics, TextError> {
        Err(TextError::MeasurementUnavailable)
    }

    /// Shapes complete text, wraps to the requested width and clips its raster.
    fn layout(&mut self, request: TextRequest<'_>) -> Result<TextLayout, TextError>;
}

/// A validated bounded text request in whole viewport pixels.
#[derive(Clone, Copy, Debug)]
pub struct TextRequest<'a> {
    text: &'a str,
    style: TextStyle,
    size: Size,
    limits: TextLimits,
}

impl<'a> TextRequest<'a> {
    /// Validates text, typography and a nonempty raster before adapter work.
    pub fn new(
        text: &'a str,
        style: TextStyle,
        size: Size,
        limits: TextLimits,
    ) -> Result<Self, TextError> {
        validate_content(text, style, limits)?;
        let pixels = size.pixel_count().map_err(|_| TextError::InvalidRaster)?;
        limits.check("text pixels", pixels, limits.max_pixels())?;
        Ok(Self {
            text,
            style,
            size,
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

    /// Returns the exact pixel dimensions required in the output raster.
    #[must_use]
    pub const fn size(self) -> Size {
        self.size
    }

    /// Returns the inclusive request bounds.
    #[must_use]
    pub const fn limits(self) -> TextLimits {
        self.limits
    }
}

fn validate_content(text: &str, style: TextStyle, limits: TextLimits) -> Result<(), TextError> {
    style.validate()?;
    limits.check("text bytes", text.len(), limits.max_bytes())
}

pub(crate) fn validate_budget<'a>(
    elements: impl Iterator<Item = (&'a str, Size)>,
    limits: TextLimits,
) -> Result<(), Error> {
    let mut bytes = 0usize;
    let mut pixels = 0usize;
    for (text, size) in elements {
        bytes = bytes
            .checked_add(text.len())
            .ok_or(Error::CapacityOverflow)?;
        let area = (size.width() as usize)
            .checked_mul(size.height() as usize)
            .ok_or(Error::CapacityOverflow)?;
        pixels = pixels.checked_add(area).ok_or(Error::CapacityOverflow)?;
        limits.check("text bytes", bytes, limits.max_bytes())?;
        limits.check("text pixels", pixels, limits.max_pixels())?;
    }
    Ok(())
}
