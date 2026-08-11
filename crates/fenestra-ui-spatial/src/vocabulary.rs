/// Closed spatial axis vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialAxisV2 {
    /// Horizontal axis.
    X,
    /// Vertical axis.
    Y,
}

impl SpatialAxisV2 {
    /// Every spatial axis in deterministic vocabulary order.
    pub const ALL: [Self; 2] = [Self::X, Self::Y];
}

/// Closed spatial extent vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialExtentV2 {
    /// Horizontal extent.
    Width,
    /// Vertical extent.
    Height,
}

impl SpatialExtentV2 {
    /// Every spatial extent in deterministic vocabulary order.
    pub const ALL: [Self; 2] = [Self::Width, Self::Height];
}

/// Closed raw-field vocabulary for one spatial node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialNodeFieldV2 {
    /// Dense supplied node key.
    Key,
    /// Supplied spatial parent.
    Parent,
    /// Placement discriminant.
    Placement,
    /// Free-placement width.
    FreeWidth,
    /// Free-placement height.
    FreeHeight,
    /// Free-placement horizontal offset.
    FreeOffsetX,
    /// Free-placement vertical offset.
    FreeOffsetY,
    /// Layout width minimum.
    LayoutWidthMinimum,
    /// Layout width preference.
    LayoutWidthPreferred,
    /// Layout width maximum.
    LayoutWidthMaximum,
    /// Layout height minimum.
    LayoutHeightMinimum,
    /// Layout height preference.
    LayoutHeightPreferred,
    /// Layout height maximum.
    LayoutHeightMaximum,
    /// Child layout axis owned by the node.
    ContainerAxis,
    /// Left container padding.
    PaddingLeft,
    /// Right container padding.
    PaddingRight,
    /// Top container padding.
    PaddingTop,
    /// Bottom container padding.
    PaddingBottom,
    /// Inter-child layout gap.
    Gap,
    /// First affine coefficient.
    AffineA,
    /// Second affine coefficient.
    AffineB,
    /// Third affine coefficient.
    AffineC,
    /// Fourth affine coefficient.
    AffineD,
    /// Horizontal affine translation.
    AffineTx,
    /// Vertical affine translation.
    AffineTy,
    /// Horizontal transform origin.
    TransformOriginX,
    /// Vertical transform origin.
    TransformOriginY,
    /// Horizontal self-anchor component.
    SelfAnchorHorizontal,
    /// Vertical self-anchor component.
    SelfAnchorVertical,
    /// Anchor target discriminant.
    TargetKind,
    /// Supplied anchor target key.
    TargetKey,
    /// Horizontal target-anchor component.
    TargetAnchorHorizontal,
    /// Vertical target-anchor component.
    TargetAnchorVertical,
}

impl SpatialNodeFieldV2 {
    /// Every node field in deterministic validation order.
    pub const ALL: [Self; 33] = [
        Self::Key,
        Self::Parent,
        Self::Placement,
        Self::FreeWidth,
        Self::FreeHeight,
        Self::FreeOffsetX,
        Self::FreeOffsetY,
        Self::LayoutWidthMinimum,
        Self::LayoutWidthPreferred,
        Self::LayoutWidthMaximum,
        Self::LayoutHeightMinimum,
        Self::LayoutHeightPreferred,
        Self::LayoutHeightMaximum,
        Self::ContainerAxis,
        Self::PaddingLeft,
        Self::PaddingRight,
        Self::PaddingTop,
        Self::PaddingBottom,
        Self::Gap,
        Self::AffineA,
        Self::AffineB,
        Self::AffineC,
        Self::AffineD,
        Self::AffineTx,
        Self::AffineTy,
        Self::TransformOriginX,
        Self::TransformOriginY,
        Self::SelfAnchorHorizontal,
        Self::SelfAnchorVertical,
        Self::TargetKind,
        Self::TargetKey,
        Self::TargetAnchorHorizontal,
        Self::TargetAnchorVertical,
    ];
}
