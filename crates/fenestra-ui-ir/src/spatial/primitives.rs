use crate::ids::{SpatialClipSymbolV2, SpatialNodeSymbolV2};

use super::{SpatialBindingV2, SpatialFieldV2};

/// Address of a clip owned by a symbolic spatial node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialClipAddressV2 {
    owner: SpatialFieldV2<SpatialNodeSymbolV2>,
    clip: SpatialFieldV2<SpatialClipSymbolV2>,
}

impl SpatialClipAddressV2 {
    /// Creates a node-local clip address.
    #[must_use]
    pub const fn new(
        owner: SpatialFieldV2<SpatialNodeSymbolV2>,
        clip: SpatialFieldV2<SpatialClipSymbolV2>,
    ) -> Self {
        Self { owner, clip }
    }

    /// Returns the owning node symbol.
    #[must_use]
    pub const fn owner(self) -> SpatialFieldV2<SpatialNodeSymbolV2> {
        self.owner
    }

    /// Returns the node-local clip symbol.
    #[must_use]
    pub const fn clip(self) -> SpatialFieldV2<SpatialClipSymbolV2> {
        self.clip
    }
}

/// Symbolic two-dimensional point recipe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialPointRecipeV2 {
    x: SpatialFieldV2<SpatialBindingV2<i64>>,
    y: SpatialFieldV2<SpatialBindingV2<i64>>,
}

impl SpatialPointRecipeV2 {
    /// Creates a symbolic point recipe.
    #[must_use]
    pub const fn new(
        x: SpatialFieldV2<SpatialBindingV2<i64>>,
        y: SpatialFieldV2<SpatialBindingV2<i64>>,
    ) -> Self {
        Self { x, y }
    }

    /// Returns the horizontal coordinate recipe.
    #[must_use]
    pub const fn x(self) -> SpatialFieldV2<SpatialBindingV2<i64>> {
        self.x
    }

    /// Returns the vertical coordinate recipe.
    #[must_use]
    pub const fn y(self) -> SpatialFieldV2<SpatialBindingV2<i64>> {
        self.y
    }
}

/// Symbolic padding recipe in physical edge order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialPaddingRecipeV2 {
    left: SpatialFieldV2<SpatialBindingV2<i32>>,
    right: SpatialFieldV2<SpatialBindingV2<i32>>,
    top: SpatialFieldV2<SpatialBindingV2<i32>>,
    bottom: SpatialFieldV2<SpatialBindingV2<i32>>,
}

impl SpatialPaddingRecipeV2 {
    /// Creates a symbolic padding recipe.
    #[must_use]
    pub const fn new(
        left: SpatialFieldV2<SpatialBindingV2<i32>>,
        right: SpatialFieldV2<SpatialBindingV2<i32>>,
        top: SpatialFieldV2<SpatialBindingV2<i32>>,
        bottom: SpatialFieldV2<SpatialBindingV2<i32>>,
    ) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    /// Returns the left padding recipe.
    #[must_use]
    pub const fn left(self) -> SpatialFieldV2<SpatialBindingV2<i32>> {
        self.left
    }

    /// Returns the right padding recipe.
    #[must_use]
    pub const fn right(self) -> SpatialFieldV2<SpatialBindingV2<i32>> {
        self.right
    }

    /// Returns the top padding recipe.
    #[must_use]
    pub const fn top(self) -> SpatialFieldV2<SpatialBindingV2<i32>> {
        self.top
    }

    /// Returns the bottom padding recipe.
    #[must_use]
    pub const fn bottom(self) -> SpatialFieldV2<SpatialBindingV2<i32>> {
        self.bottom
    }
}

/// Symbolic minimum, preferred, and maximum dimension recipe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialDimensionRecipeV2 {
    minimum: SpatialFieldV2<SpatialBindingV2<i32>>,
    preferred: SpatialFieldV2<SpatialBindingV2<i32>>,
    maximum: SpatialFieldV2<SpatialBindingV2<i32>>,
}

impl SpatialDimensionRecipeV2 {
    /// Creates a symbolic dimension recipe.
    #[must_use]
    pub const fn new(
        minimum: SpatialFieldV2<SpatialBindingV2<i32>>,
        preferred: SpatialFieldV2<SpatialBindingV2<i32>>,
        maximum: SpatialFieldV2<SpatialBindingV2<i32>>,
    ) -> Self {
        Self {
            minimum,
            preferred,
            maximum,
        }
    }

    /// Returns the minimum dimension recipe.
    #[must_use]
    pub const fn minimum(self) -> SpatialFieldV2<SpatialBindingV2<i32>> {
        self.minimum
    }

    /// Returns the preferred dimension recipe.
    #[must_use]
    pub const fn preferred(self) -> SpatialFieldV2<SpatialBindingV2<i32>> {
        self.preferred
    }

    /// Returns the maximum dimension recipe.
    #[must_use]
    pub const fn maximum(self) -> SpatialFieldV2<SpatialBindingV2<i32>> {
        self.maximum
    }
}
