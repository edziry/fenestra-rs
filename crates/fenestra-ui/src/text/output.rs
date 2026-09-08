use super::{TextError, TextMeasureRequest, TextRequest};
use crate::{Raster, Size};

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
    /// Describes measurements; validate them before use in layout or painting.
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

    /// Checks counts, the glyph budget and dimensions before intrinsic layout.
    ///
    /// Rounded dimensions must fit signed pixel coordinates. The width can
    /// exceed the requested wrap width when an indivisible glyph overflows it.
    pub fn validate_measurement(self, request: TextMeasureRequest<'_>) -> Result<(), TextError> {
        self.ceil_size()?;
        request
            .limits()
            .check("text glyphs", self.glyphs, request.limits().max_glyphs())
    }

    /// Rounds dimensions up to whole pixels, allowing a zero width or height.
    ///
    /// Rejects invalid metrics and dimensions outside signed pixel coordinates
    /// before converting them, without applying any raster pixel budget.
    pub fn ceil_size(self) -> Result<Size, TextError> {
        self.validate()?;
        let width = f64::from(self.width).ceil();
        let height = f64::from(self.height).ceil();
        if width > f64::from(i32::MAX) || height > f64::from(i32::MAX) {
            return Err(TextError::InvalidMetrics);
        }
        Ok(Size::new(width as u32, height as u32))
    }

    fn validate(self) -> Result<(), TextError> {
        if !self.width.is_finite()
            || self.width < 0.0
            || !self.height.is_finite()
            || self.height < 0.0
            || self.missing_glyphs > self.glyphs
        {
            return Err(TextError::InvalidMetrics);
        }
        Ok(())
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
        metrics.validate()?;
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
