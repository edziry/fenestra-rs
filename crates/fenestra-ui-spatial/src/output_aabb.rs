//! Raw candidate output bounds.

use crate::model::SpatialScalarV2;

/// Unvalidated candidate output AABB fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialOutputAabbV2 {
    empty: bool,
    min_x: SpatialScalarV2,
    min_y: SpatialScalarV2,
    max_x: SpatialScalarV2,
    max_y: SpatialScalarV2,
}

impl SpatialOutputAabbV2 {
    /// Creates raw candidate bounds without validating or canonicalizing them.
    #[must_use]
    pub const fn new(
        empty: bool,
        min_x: SpatialScalarV2,
        min_y: SpatialScalarV2,
        max_x: SpatialScalarV2,
        max_y: SpatialScalarV2,
    ) -> Self {
        Self {
            empty,
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    /// Returns the raw empty marker.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.empty
    }

    /// Returns the raw minimum horizontal edge.
    #[must_use]
    pub const fn min_x(self) -> SpatialScalarV2 {
        self.min_x
    }

    /// Returns the raw minimum vertical edge.
    #[must_use]
    pub const fn min_y(self) -> SpatialScalarV2 {
        self.min_y
    }

    /// Returns the raw maximum horizontal edge.
    #[must_use]
    pub const fn max_x(self) -> SpatialScalarV2 {
        self.max_x
    }

    /// Returns the raw maximum vertical edge.
    #[must_use]
    pub const fn max_y(self) -> SpatialScalarV2 {
        self.max_y
    }
}
