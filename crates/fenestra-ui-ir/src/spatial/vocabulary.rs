use crate::ids::SpatialNodeSymbolV2;

use super::SpatialFieldV2;

/// Primary axis of a spatial container.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialAxisV2 {
    /// Places children from left to right.
    Row,
    /// Places children from top to bottom.
    Column,
}

impl SpatialAxisV2 {
    /// All spatial axes in stable declaration order.
    pub const ALL: [Self; 2] = [Self::Row, Self::Column];
}

/// One component of a two-dimensional anchor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialAnchorComponentV2 {
    /// Anchors at the start edge.
    Start,
    /// Anchors at the center.
    Center,
    /// Anchors at the end edge.
    End,
}

impl SpatialAnchorComponentV2 {
    /// All anchor components in stable declaration order.
    pub const ALL: [Self; 3] = [Self::Start, Self::Center, Self::End];
}

/// Fill rule used to determine shape coverage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialFillRuleV2 {
    /// Uses non-zero winding.
    NonZero,
    /// Uses even-odd winding.
    EvenOdd,
}

impl SpatialFillRuleV2 {
    /// All fill rules in stable declaration order.
    pub const ALL: [Self; 2] = [Self::NonZero, Self::EvenOdd];
}

/// Parent declaration for a symbolic spatial node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialNodeParentV2 {
    /// Places the node directly below the viewport.
    Viewport,
    /// Places the node below the identified spatial node.
    Node(SpatialFieldV2<SpatialNodeSymbolV2>),
}

/// Target used by a free-placement anchor recipe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialAnchorTargetRecipeV2 {
    /// Anchors to the viewport.
    Viewport,
    /// Anchors to the declared parent.
    Parent,
    /// Anchors to the identified spatial node.
    Node(SpatialFieldV2<SpatialNodeSymbolV2>),
}
