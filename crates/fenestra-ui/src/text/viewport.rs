use super::{TextError, TextMeasureRequest, TextRequest};

/// Explicit shaping width and integral scroll offset for a bounded text raster.
#[derive(Clone, Copy, Debug)]
pub struct TextViewportRequest<'a> {
    request: TextRequest<'a>,
    wrap_width: Option<u32>,
    offset_x: u32,
    offset_y: u32,
}

impl<'a> TextViewportRequest<'a> {
    /// Validates wrap width and the viewport's far edges in signed pixels.
    ///
    /// No wrap width disables soft wrapping. The existing request supplies
    /// exact raster dimensions and pixel limits, independently of shaping width.
    pub fn new(
        request: TextRequest<'a>,
        wrap_width: Option<u32>,
        offset_x: u32,
        offset_y: u32,
    ) -> Result<Self, TextError> {
        TextMeasureRequest::new(
            request.text(),
            request.style(),
            wrap_width,
            request.limits(),
        )?;
        for (offset, edge) in [
            (offset_x, request.size().width()),
            (offset_y, request.size().height()),
        ] {
            if offset
                .checked_add(edge)
                .is_none_or(|value| value > i32::MAX as u32)
            {
                return Err(TextError::InvalidViewportOffset);
            }
        }
        Ok(Self {
            request,
            wrap_width,
            offset_x,
            offset_y,
        })
    }
    /// Returns the validated text raster request.
    #[must_use]
    pub const fn request(self) -> TextRequest<'a> {
        self.request
    }
    /// Returns the independent wrap width, or no soft wrapping.
    #[must_use]
    pub const fn wrap_width(self) -> Option<u32> {
        self.wrap_width
    }
    /// Returns the horizontal distance scrolled from the layout origin.
    #[must_use]
    pub const fn offset_x(self) -> u32 {
        self.offset_x
    }
    /// Returns the vertical distance scrolled from the layout origin.
    #[must_use]
    pub const fn offset_y(self) -> u32 {
        self.offset_y
    }
    /// Returns an equivalent raster-free measurement request.
    pub fn measurement(self) -> Result<TextMeasureRequest<'a>, TextError> {
        TextMeasureRequest::new(
            self.request.text(),
            self.request.style(),
            self.wrap_width,
            self.request.limits(),
        )
    }
}
