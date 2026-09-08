use super::TextError;

/// Inclusive aggregate application text bounds and per-request glyph bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextLimits {
    max_bytes: usize,
    max_pixels: usize,
    max_glyphs: usize,
}

impl TextLimits {
    /// Bounds UTF-8 bytes, logical text raster pixels, and glyphs per request.
    ///
    /// Pixel bounds count one bitmap per text element. Prepared snapshots and
    /// engine work buffers can retain additional copies; this is not a heap cap.
    #[must_use]
    pub const fn new(max_bytes: usize, max_pixels: usize, max_glyphs: usize) -> Self {
        Self {
            max_bytes,
            max_pixels,
            max_glyphs,
        }
    }

    /// Returns the total accepted text byte budget.
    #[must_use]
    pub const fn max_bytes(self) -> usize {
        self.max_bytes
    }

    /// Returns the total logical text raster pixel budget.
    #[must_use]
    pub const fn max_pixels(self) -> usize {
        self.max_pixels
    }

    /// Returns the glyph bound for one shaping request, including clipped glyphs.
    #[must_use]
    pub const fn max_glyphs(self) -> usize {
        self.max_glyphs
    }

    pub(crate) const fn defaults() -> Self {
        Self::new(262_144, 4_194_304, 32_768)
    }

    pub(crate) fn check(
        self,
        resource: &'static str,
        actual: usize,
        limit: usize,
    ) -> Result<(), TextError> {
        if actual > limit {
            Err(TextError::LimitExceeded {
                resource,
                actual,
                limit,
            })
        } else {
            Ok(())
        }
    }
}

impl Default for TextLimits {
    fn default() -> Self {
        Self::defaults()
    }
}
