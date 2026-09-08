use super::{TextError, TextMeasureRequest, TextPoint, TextSelection};

/// One selection query over the same complete shaped text layout.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum TextGeometryQuery {
    /// Return geometry for the supplied selection without moving its bytes.
    #[default]
    Current,
    /// Hit-test a layout-space point, optionally retaining the current anchor.
    Hit {
        /// Position in unscrolled text layout coordinates.
        point: TextPoint,
        /// Preserve the anchor when true; otherwise collapse at the hit.
        extend: bool,
    },
    /// Move one grapheme boundary toward the visual left.
    Left {
        /// Preserve the anchor when true.
        extend: bool,
    },
    /// Move one grapheme boundary toward the visual right.
    Right {
        /// Preserve the anchor when true.
        extend: bool,
    },
    /// Move to the logical start of the current visually wrapped line.
    LineStart {
        /// Preserve the anchor when true.
        extend: bool,
    },
    /// Move to the logical end of the current visually wrapped line.
    LineEnd {
        /// Preserve the anchor when true.
        extend: bool,
    },
    /// Move up one visual line while retaining a desired horizontal position.
    Up {
        /// Preserve the anchor when true.
        extend: bool,
        /// Previous vertical movement's horizontal position, if any.
        preferred_x: Option<f64>,
    },
    /// Move down one visual line while retaining a desired horizontal position.
    Down {
        /// Preserve the anchor when true.
        extend: bool,
        /// Previous vertical movement's horizontal position, if any.
        preferred_x: Option<f64>,
    },
}

/// A bounded geometry query without a text raster allocation.
#[derive(Clone, Copy, Debug)]
pub struct TextGeometryRequest<'a> {
    measurement: TextMeasureRequest<'a>,
    selection: TextSelection,
    query: TextGeometryQuery,
    max_rects: usize,
}

impl<'a> TextGeometryRequest<'a> {
    /// Validates source grapheme endpoints and bounded query coordinates.
    ///
    /// Highlight output is limited to the source byte length plus one,
    /// independently of glyph count. [`Self::with_max_rects`] can tighten it.
    pub fn new(
        measurement: TextMeasureRequest<'a>,
        selection: TextSelection,
        query: TextGeometryQuery,
    ) -> Result<Self, TextError> {
        selection.validate(measurement.text())?;
        match query {
            TextGeometryQuery::Hit { point, .. } => point.validate()?,
            TextGeometryQuery::Up {
                preferred_x: Some(x),
                ..
            }
            | TextGeometryQuery::Down {
                preferred_x: Some(x),
                ..
            } => {
                super::coordinates::validate_coordinate(x)?;
            }
            _ => {}
        }
        Ok(Self {
            measurement,
            selection,
            query,
            max_rects: measurement.text().len().saturating_add(1),
        })
    }
    /// Tightens the inclusive highlight rectangle limit, even to zero.
    #[must_use]
    pub fn with_max_rects(mut self, maximum: usize) -> Self {
        self.max_rects = self.max_rects.min(maximum);
        self
    }
    /// Returns the complete text, style, wrap width and shaping bounds.
    #[must_use]
    pub const fn measurement(self) -> TextMeasureRequest<'a> {
        self.measurement
    }
    /// Returns the directed source selection before the query.
    #[must_use]
    pub const fn selection(self) -> TextSelection {
        self.selection
    }
    /// Returns the operation to perform on the current selection.
    #[must_use]
    pub const fn query(self) -> TextGeometryQuery {
        self.query
    }
    /// Returns the inclusive highlight rectangle limit.
    #[must_use]
    pub const fn max_rects(self) -> usize {
        self.max_rects
    }
    /// Iterates the editor's exact extended grapheme boundaries, including end.
    ///
    /// This borrowed iterator lets adapters use the same segmentation policy
    /// as the owned text buffer without copying text or exposing dependencies.
    pub fn grapheme_boundaries(self) -> impl Iterator<Item = usize> + 'a {
        crate::editing::grapheme_boundaries(self.measurement.text())
    }
}
