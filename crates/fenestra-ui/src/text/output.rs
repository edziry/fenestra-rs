use super::{TextError, TextRequest};
use crate::Raster;

/// Unclipped shaped text measurements in viewport pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextMetrics {
    width: f32,
    height: f32,
    lines: usize,
    glyphs: usize,
    missing_glyphs: usize,
}

impl TextMetrics {
    /// Describes measurements; `TextLayout::new` validates them before use.
    #[must_use]
    pub const fn new(
        width: f32,
        height: f32,
        lines: usize,
        glyphs: usize,
        missing_glyphs: usize,
    ) -> Self {
        Self {
            width,
            height,
            lines,
            glyphs,
            missing_glyphs,
        }
    }
    /// Returns maximum shaped line width, excluding trailing whitespace.
    #[must_use]
    pub const fn width(self) -> f32 {
        self.width
    }
    /// Returns full laid-out height, including lines outside the raster.
    #[must_use]
    pub const fn height(self) -> f32 {
        self.height
    }
    /// Returns the number of shaped lines.
    #[must_use]
    pub const fn lines(self) -> usize {
        self.lines
    }
    /// Returns the complete glyph count, including clipped glyphs.
    #[must_use]
    pub const fn glyphs(self) -> usize {
        self.glyphs
    }
    /// Returns glyphs missing from the supplied font set.
    #[must_use]
    pub const fn missing_glyphs(self) -> usize {
        self.missing_glyphs
    }
    pub(crate) const fn empty() -> Self {
        Self::new(0.0, 0.0, 0, 0, 0)
    }
}

/// Owned, validated text pixels and measurements retained by an application.
#[derive(Clone, Debug, PartialEq)]
pub struct TextLayout {
    raster: Raster,
    metrics: TextMetrics,
}

impl TextLayout {
    /// Validates finite nonnegative metrics and premultiplied RGBA8 channels.
    pub fn new(raster: Raster, metrics: TextMetrics) -> Result<Self, TextError> {
        if !metrics.width.is_finite()
            || metrics.width < 0.0
            || !metrics.height.is_finite()
            || metrics.height < 0.0
            || metrics.missing_glyphs > metrics.glyphs
        {
            return Err(TextError::InvalidMetrics);
        }
        if raster
            .bytes()
            .chunks_exact(4)
            .any(|p| p[..3].iter().any(|c| *c > p[3]))
        {
            return Err(TextError::InvalidRaster);
        }
        Ok(Self { raster, metrics })
    }
    /// Borrows bounded premultiplied pixels; text outside this size is clipped.
    #[must_use]
    pub const fn raster(&self) -> &Raster {
        &self.raster
    }
    /// Returns complete layout measurements.
    #[must_use]
    pub const fn metrics(&self) -> TextMetrics {
        self.metrics
    }
    /// Checks exact request dimensions and the glyph budget before publication.
    pub fn validate_request(&self, request: TextRequest<'_>) -> Result<(), TextError> {
        if self.raster.size() != request.size() {
            return Err(TextError::InvalidRaster);
        }
        request.limits().check(
            "text glyphs",
            self.metrics.glyphs,
            request.limits().max_glyphs(),
        )
    }
}
