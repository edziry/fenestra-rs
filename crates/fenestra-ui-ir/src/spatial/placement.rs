use super::{
    SpatialAnchorComponentV2, SpatialAnchorTargetRecipeV2, SpatialAxisV2, SpatialBindingV2,
    SpatialDimensionRecipeV2, SpatialFieldV2, SpatialPaddingRecipeV2, SpatialPointRecipeV2,
    SpatialTransformRecipeV2,
};

/// Symbolic container recipe for one spatial node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialContainerRecipeV2 {
    axis: SpatialAxisV2,
    padding: SpatialPaddingRecipeV2,
    gap: SpatialFieldV2<SpatialBindingV2<i32>>,
}

impl SpatialContainerRecipeV2 {
    /// Creates a symbolic container recipe.
    #[must_use]
    pub const fn new(
        axis: SpatialAxisV2,
        padding: SpatialPaddingRecipeV2,
        gap: SpatialFieldV2<SpatialBindingV2<i32>>,
    ) -> Self {
        Self { axis, padding, gap }
    }

    /// Returns the child layout axis.
    #[must_use]
    pub const fn axis(self) -> SpatialAxisV2 {
        self.axis
    }

    /// Returns the container padding recipe.
    #[must_use]
    pub const fn padding(self) -> SpatialPaddingRecipeV2 {
        self.padding
    }

    /// Returns the child gap recipe.
    #[must_use]
    pub const fn gap(self) -> SpatialFieldV2<SpatialBindingV2<i32>> {
        self.gap
    }
}

/// Placement recipe for a node participating in parent layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialLayoutPlacementRecipeV2 {
    width: SpatialDimensionRecipeV2,
    height: SpatialDimensionRecipeV2,
    transform: SpatialTransformRecipeV2,
}

impl SpatialLayoutPlacementRecipeV2 {
    /// Creates a layout placement recipe.
    #[must_use]
    pub const fn new(
        width: SpatialDimensionRecipeV2,
        height: SpatialDimensionRecipeV2,
        transform: SpatialTransformRecipeV2,
    ) -> Self {
        Self {
            width,
            height,
            transform,
        }
    }

    /// Returns the width constraints.
    #[must_use]
    pub const fn width(self) -> SpatialDimensionRecipeV2 {
        self.width
    }

    /// Returns the height constraints.
    #[must_use]
    pub const fn height(self) -> SpatialDimensionRecipeV2 {
        self.height
    }

    /// Returns the post-layout transform.
    #[must_use]
    pub const fn transform(self) -> SpatialTransformRecipeV2 {
        self.transform
    }
}

/// Explicitly anchored placement recipe outside parent layout flow.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialFreePlacementRecipeV2 {
    width: SpatialFieldV2<SpatialBindingV2<i32>>,
    height: SpatialFieldV2<SpatialBindingV2<i32>>,
    self_anchor: [SpatialAnchorComponentV2; 2],
    target: SpatialAnchorTargetRecipeV2,
    target_anchor: [SpatialAnchorComponentV2; 2],
    offset: SpatialPointRecipeV2,
    transform: SpatialTransformRecipeV2,
}

impl SpatialFreePlacementRecipeV2 {
    /// Creates a free placement recipe.
    #[must_use]
    pub const fn new(
        width: SpatialFieldV2<SpatialBindingV2<i32>>,
        height: SpatialFieldV2<SpatialBindingV2<i32>>,
        self_anchor: [SpatialAnchorComponentV2; 2],
        target: SpatialAnchorTargetRecipeV2,
        target_anchor: [SpatialAnchorComponentV2; 2],
        offset: SpatialPointRecipeV2,
        transform: SpatialTransformRecipeV2,
    ) -> Self {
        Self {
            width,
            height,
            self_anchor,
            target,
            target_anchor,
            offset,
            transform,
        }
    }

    /// Returns the explicit width recipe.
    #[must_use]
    pub const fn width(self) -> SpatialFieldV2<SpatialBindingV2<i32>> {
        self.width
    }

    /// Returns the explicit height recipe.
    #[must_use]
    pub const fn height(self) -> SpatialFieldV2<SpatialBindingV2<i32>> {
        self.height
    }

    /// Returns the horizontal and vertical anchor on this node.
    #[must_use]
    pub const fn self_anchor(self) -> [SpatialAnchorComponentV2; 2] {
        self.self_anchor
    }

    /// Returns the anchor target recipe.
    #[must_use]
    pub const fn target(self) -> SpatialAnchorTargetRecipeV2 {
        self.target
    }

    /// Returns the horizontal and vertical anchor on the target.
    #[must_use]
    pub const fn target_anchor(self) -> [SpatialAnchorComponentV2; 2] {
        self.target_anchor
    }

    /// Returns the offset from the aligned anchors.
    #[must_use]
    pub const fn offset(self) -> SpatialPointRecipeV2 {
        self.offset
    }

    /// Returns the post-placement transform.
    #[must_use]
    pub const fn transform(self) -> SpatialTransformRecipeV2 {
        self.transform
    }
}

/// Placement mode for a symbolic spatial node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialPlacementRecipeV2 {
    /// Participates in parent layout.
    Layout(SpatialLayoutPlacementRecipeV2),
    /// Uses an explicit anchor relationship.
    Free(SpatialFreePlacementRecipeV2),
}
