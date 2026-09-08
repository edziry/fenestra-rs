/// Fractional physical pixel coordinates in unscrolled text layout space.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextPoint {
    x: f64,
    y: f64,
}

impl TextPoint {
    pub(super) fn validate(self) -> Result<(), super::TextError> {
        validate_coordinate(self.x)?;
        validate_coordinate(self.y)
    }
    /// Describes a point; geometry requests reject invalid coordinates.
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    /// Returns the horizontal coordinate.
    #[must_use]
    pub const fn x(self) -> f64 {
        self.x
    }
    /// Returns the vertical coordinate.
    #[must_use]
    pub const fn y(self) -> f64 {
        self.y
    }
}

/// Fractional rectangle edges in unscrolled text layout space.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextRect {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

impl TextRect {
    pub(super) fn validate(self) -> Result<(), super::TextError> {
        for value in [self.x0, self.y0, self.x1, self.y1] {
            validate_coordinate(value)?;
        }
        if self.x0 > self.x1 || self.y0 > self.y1 {
            return Err(super::TextError::InvalidGeometry);
        }
        Ok(())
    }
    /// Describes edges; geometry output validation rejects invalid rectangles.
    #[must_use]
    pub const fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        Self { x0, y0, x1, y1 }
    }
    /// Returns the left edge.
    #[must_use]
    pub const fn x0(self) -> f64 {
        self.x0
    }
    /// Returns the top edge.
    #[must_use]
    pub const fn y0(self) -> f64 {
        self.y0
    }
    /// Returns the right edge.
    #[must_use]
    pub const fn x1(self) -> f64 {
        self.x1
    }
    /// Returns the bottom edge.
    #[must_use]
    pub const fn y1(self) -> f64 {
        self.y1
    }
    /// Returns the distance between horizontal edges.
    #[must_use]
    pub fn width(self) -> f64 {
        self.x1 - self.x0
    }
    /// Returns the distance between vertical edges.
    #[must_use]
    pub fn height(self) -> f64 {
        self.y1 - self.y0
    }
}

pub(super) fn validate_coordinate(value: f64) -> Result<(), super::TextError> {
    if !value.is_finite() || value.abs() > f64::from(i32::MAX) {
        return Err(super::TextError::InvalidGeometry);
    }
    Ok(())
}

/// One contiguous visual selection fragment on a shaped line.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextHighlight {
    rect: TextRect,
    line: usize,
}

impl TextHighlight {
    /// Describes a fragment to be validated as part of geometry output.
    #[must_use]
    pub const fn new(rect: TextRect, line: usize) -> Self {
        Self { rect, line }
    }
    /// Returns the fragment's fractional layout-space rectangle.
    #[must_use]
    pub const fn rect(self) -> TextRect {
        self.rect
    }
    /// Returns the zero-based shaped line index.
    #[must_use]
    pub const fn line(self) -> usize {
        self.line
    }
}
