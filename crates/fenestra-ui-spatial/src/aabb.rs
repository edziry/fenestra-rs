use crate::model::{Affine2V2, SpatialScalarV2};
use crate::numeric::{ceil_ratio, floor_ratio, scalar_from_i128};
use crate::numeric_error::SpatialArithmeticOperationV2;

/// Closed conservative axis-aligned bounds in scene-logical coordinates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialAabbV2 {
    empty: bool,
    min_x: SpatialScalarV2,
    min_y: SpatialScalarV2,
    max_x: SpatialScalarV2,
    max_y: SpatialScalarV2,
}

impl SpatialAabbV2 {
    /// Returns the single canonical empty bound.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            empty: true,
            min_x: SpatialScalarV2::new(0),
            min_y: SpatialScalarV2::new(0),
            max_x: SpatialScalarV2::new(0),
            max_y: SpatialScalarV2::new(0),
        }
    }

    /// Creates nonempty closed bounds from canonical ordered edges.
    #[must_use]
    pub const fn from_edges(
        min_x: SpatialScalarV2,
        min_y: SpatialScalarV2,
        max_x: SpatialScalarV2,
        max_y: SpatialScalarV2,
    ) -> Option<Self> {
        if !min_x.is_in_domain()
            || !min_y.is_in_domain()
            || !max_x.is_in_domain()
            || !max_y.is_in_domain()
            || min_x.raw() > max_x.raw()
            || min_y.raw() > max_y.raw()
        {
            return None;
        }
        Some(Self {
            empty: false,
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }

    /// Reports whether these bounds are the canonical empty value.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.empty
    }

    /// Returns the minimum horizontal edge, or zero for empty bounds.
    #[must_use]
    pub const fn min_x(self) -> SpatialScalarV2 {
        self.min_x
    }

    /// Returns the minimum vertical edge, or zero for empty bounds.
    #[must_use]
    pub const fn min_y(self) -> SpatialScalarV2 {
        self.min_y
    }

    /// Returns the maximum horizontal edge, or zero for empty bounds.
    #[must_use]
    pub const fn max_x(self) -> SpatialScalarV2 {
        self.max_x
    }

    /// Returns the maximum vertical edge, or zero for empty bounds.
    #[must_use]
    pub const fn max_y(self) -> SpatialScalarV2 {
        self.max_y
    }

    /// Intersects two closed bounds and canonicalizes a disjoint result.
    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        if self.empty || other.empty {
            return Self::empty();
        }

        let min_x = maximum(self.min_x, other.min_x);
        let min_y = maximum(self.min_y, other.min_y);
        let max_x = minimum(self.max_x, other.max_x);
        let max_y = minimum(self.max_y, other.max_y);
        if min_x.raw() > max_x.raw() || min_y.raw() > max_y.raw() {
            Self::empty()
        } else {
            Self {
                empty: false,
                min_x,
                min_y,
                max_x,
                max_y,
            }
        }
    }
}

impl Affine2V2 {
    /// Transforms closed local bounds and rounds their exact extrema outward.
    #[must_use = "the transformed bounds or their failing operation must be handled"]
    pub const fn checked_transform_aabb(
        self,
        local: SpatialAabbV2,
    ) -> Result<SpatialAabbV2, SpatialArithmeticOperationV2> {
        if local.is_empty() {
            return Ok(SpatialAabbV2::empty());
        }

        let min_x = match transformed_edge(self.a(), self.c(), self.tx(), local, false) {
            Some(value) => value,
            None => return Err(SpatialArithmeticOperationV2::AabbMinX),
        };
        let min_y = match transformed_edge(self.b(), self.d(), self.ty(), local, false) {
            Some(value) => value,
            None => return Err(SpatialArithmeticOperationV2::AabbMinY),
        };
        let max_x = match transformed_edge(self.a(), self.c(), self.tx(), local, true) {
            Some(value) => value,
            None => return Err(SpatialArithmeticOperationV2::AabbMaxX),
        };
        let max_y = match transformed_edge(self.b(), self.d(), self.ty(), local, true) {
            Some(value) => value,
            None => return Err(SpatialArithmeticOperationV2::AabbMaxY),
        };

        Ok(SpatialAabbV2 {
            empty: false,
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }
}

const fn transformed_edge(
    first: SpatialScalarV2,
    second: SpatialScalarV2,
    translation: SpatialScalarV2,
    local: SpatialAabbV2,
    maximum_edge: bool,
) -> Option<SpatialScalarV2> {
    let first_input = select_edge(first, local.min_x(), local.max_x(), maximum_edge);
    let second_input = select_edge(second, local.min_y(), local.max_y(), maximum_edge);
    let first = (first.raw() as i128) * (first_input.raw() as i128);
    let second = (second.raw() as i128) * (second_input.raw() as i128);
    let translation = (translation.raw() as i128) * (SpatialScalarV2::SCALE as i128);
    let numerator = match first.checked_add(second) {
        Some(value) => value,
        None => return None,
    };
    let numerator = match numerator.checked_add(translation) {
        Some(value) => value,
        None => return None,
    };

    let raw = if maximum_edge {
        ceil_ratio(numerator, SpatialScalarV2::SCALE as i128)
    } else {
        floor_ratio(numerator, SpatialScalarV2::SCALE as i128)
    };
    let raw = match raw {
        Some(value) => value,
        None => return None,
    };
    scalar_from_i128(raw)
}

const fn select_edge(
    coefficient: SpatialScalarV2,
    minimum: SpatialScalarV2,
    maximum: SpatialScalarV2,
    maximum_edge: bool,
) -> SpatialScalarV2 {
    if (coefficient.raw() >= 0) == maximum_edge {
        maximum
    } else {
        minimum
    }
}

const fn minimum(left: SpatialScalarV2, right: SpatialScalarV2) -> SpatialScalarV2 {
    if left.raw() <= right.raw() {
        left
    } else {
        right
    }
}

const fn maximum(left: SpatialScalarV2, right: SpatialScalarV2) -> SpatialScalarV2 {
    if left.raw() >= right.raw() {
        left
    } else {
        right
    }
}
