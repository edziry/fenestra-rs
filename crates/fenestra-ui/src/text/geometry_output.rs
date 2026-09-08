use super::{TextError, TextGeometryRequest, TextHighlight, TextMetrics, TextRect, TextSelection};

/// Owned geometry derived from one complete shaped layout.
///
/// Carets have one pixel of width. Selection may contain disjoint rectangles
/// on the same bidi line. Component positions inside ligatures follow the
/// engine's layout policy and do not promise font-authored GDEF caret data.
#[derive(Clone, Debug, PartialEq)]
pub struct TextGeometry {
    metrics: TextMetrics,
    selection: TextSelection,
    anchor_caret: TextRect,
    focus_caret: TextRect,
    highlights: Vec<TextHighlight>,
    extent: TextRect,
    preferred_x: Option<f64>,
}

impl TextGeometry {
    /// Validates local geometry structure without borrowing source text.
    pub fn new(
        metrics: TextMetrics,
        selection: TextSelection,
        anchor_caret: TextRect,
        focus_caret: TextRect,
        highlights: Vec<TextHighlight>,
        extent: TextRect,
        preferred_x: Option<f64>,
    ) -> Result<Self, TextError> {
        metrics.ceil_size()?;
        if metrics.lines() == 0 {
            return Err(TextError::InvalidGeometry);
        }
        for caret in [anchor_caret, focus_caret] {
            caret.validate()?;
            if caret.width() != 1.0 || caret.height() <= 0.0 {
                return Err(TextError::InvalidGeometry);
            }
        }
        extent.validate()?;
        if let Some(x) = preferred_x {
            super::coordinates::validate_coordinate(x)?;
        }
        if selection.byte_selection().is_caret() && !highlights.is_empty() {
            return Err(TextError::InvalidGeometry);
        }
        for (index, highlight) in highlights.iter().enumerate() {
            highlight.rect().validate()?;
            if highlight.line() >= metrics.lines() {
                return Err(TextError::InvalidGeometry);
            }
            if let Some(previous) = index.checked_sub(1).map(|i| highlights[i])
                && (highlight.line() < previous.line()
                    || (highlight.line() == previous.line()
                        && highlight.rect().x0() < previous.rect().x1()))
            {
                return Err(TextError::InvalidGeometry);
            }
        }
        Ok(Self {
            metrics,
            selection,
            anchor_caret,
            focus_caret,
            highlights,
            extent,
            preferred_x,
        })
    }
    /// Validates source endpoints, query structure and request limits.
    pub fn validate_request(&self, request: TextGeometryRequest<'_>) -> Result<(), TextError> {
        let measure = request.measurement();
        self.selection.validate(measure.text())?;
        self.metrics.validate_measurement(measure)?;
        if self.metrics.lines() > measure.text().len().saturating_add(1)
            || (matches!(request.query(), super::TextGeometryQuery::Current)
                && self.selection.byte_selection() != request.selection().byte_selection())
        {
            return Err(TextError::InvalidGeometry);
        }
        use super::TextGeometryQuery as Query;
        let extend = match request.query() {
            Query::Current => None,
            Query::Hit { extend, .. }
            | Query::Left { extend }
            | Query::Right { extend }
            | Query::LineStart { extend }
            | Query::LineEnd { extend }
            | Query::Up { extend, .. }
            | Query::Down { extend, .. } => Some(extend),
        };
        if let Some(extend) = extend
            && ((extend && self.selection.anchor().byte() != request.selection().anchor().byte())
                || (!extend && !self.selection.byte_selection().is_caret()))
        {
            return Err(TextError::InvalidGeometry);
        }
        measure.limits().check(
            "text geometry rectangles",
            self.highlights.len(),
            request.max_rects(),
        )?;
        Ok(())
    }
    /// Returns the complete shaped metrics, including clipped text.
    #[must_use]
    pub const fn metrics(&self) -> TextMetrics {
        self.metrics
    }
    /// Returns the directed source selection after the query.
    #[must_use]
    pub const fn selection(&self) -> TextSelection {
        self.selection
    }
    /// Returns the fixed endpoint's caret rectangle.
    #[must_use]
    pub const fn anchor_caret(&self) -> TextRect {
        self.anchor_caret
    }
    /// Returns the moving endpoint's caret rectangle.
    #[must_use]
    pub const fn focus_caret(&self) -> TextRect {
        self.focus_caret
    }
    /// Borrows the selection's disjoint visual fragments in painting order.
    #[must_use]
    pub fn highlights(&self) -> &[TextHighlight] {
        &self.highlights
    }
    /// Returns the full text extent, including trailing whitespace advances.
    ///
    /// The extent excludes the caret's one-pixel thickness. Empty text has
    /// zero width and the height of its empty line box.
    #[must_use]
    pub const fn extent(&self) -> TextRect {
        self.extent
    }
    /// Returns the desired horizontal position to retain across vertical moves.
    #[must_use]
    pub const fn preferred_x(&self) -> Option<f64> {
        self.preferred_x
    }
}
