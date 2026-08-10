/// Closed stack direction for version-1 child placement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutAxisV1 {
    /// Places children from left to right.
    Row,
    /// Places children from top to bottom.
    Column,
}

impl LayoutAxisV1 {
    /// Every stack direction in deterministic vocabulary order.
    pub const ALL: [Self; 2] = [Self::Row, Self::Column];
}

/// Closed logical extent vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutExtentV1 {
    /// Horizontal extent.
    Width,
    /// Vertical extent.
    Height,
}

impl LayoutExtentV1 {
    /// Every logical extent in deterministic vocabulary order.
    pub const ALL: [Self; 2] = [Self::Width, Self::Height];
}

/// Closed field vocabulary for one dimension constraint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutConstraintFieldV1 {
    /// Inclusive lower bound.
    Minimum,
    /// Authored fixed preference.
    Preferred,
    /// Inclusive upper bound.
    Maximum,
}

impl LayoutConstraintFieldV1 {
    /// Every constraint field in deterministic validation order.
    pub const ALL: [Self; 3] = [Self::Minimum, Self::Preferred, Self::Maximum];
}

/// Closed side vocabulary for border-box padding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutPaddingSideV1 {
    /// Left padding.
    Left,
    /// Right padding.
    Right,
    /// Top padding.
    Top,
    /// Bottom padding.
    Bottom,
}

impl LayoutPaddingSideV1 {
    /// Every padding side in deterministic validation order.
    pub const ALL: [Self; 4] = [Self::Left, Self::Right, Self::Top, Self::Bottom];
}

/// Closed checked-arithmetic operation vocabulary for layout engines.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutArithmeticOperationV1 {
    /// Computes the far edge of a border box.
    FarEdge,
    /// Computes a padded child origin.
    ContentOrigin,
    /// Advances a non-final sibling cursor by its gap.
    GapAdvance,
}

impl LayoutArithmeticOperationV1 {
    /// Every arithmetic operation in deterministic vocabulary order.
    pub const ALL: [Self; 3] = [Self::FarEdge, Self::ContentOrigin, Self::GapAdvance];
}

/// Closed scalar field vocabulary for one output rectangle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutOutputFieldV1 {
    /// Horizontal origin.
    X,
    /// Vertical origin.
    Y,
    /// Horizontal extent.
    Width,
    /// Vertical extent.
    Height,
}

impl LayoutOutputFieldV1 {
    /// Every output scalar in deterministic validation order.
    pub const ALL: [Self; 4] = [Self::X, Self::Y, Self::Width, Self::Height];
}
