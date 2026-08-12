//! Closed field vocabularies for clips and ordered items.

/// Closed raw clip-record field vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialClipFieldV2 {
    /// Dense clip key.
    Key,
    /// Owning spatial-node key.
    Owner,
    /// Optional parent clip key.
    Parent,
    /// Coverage shape key.
    Shape,
    /// Coverage fill rule.
    FillRule,
}

impl SpatialClipFieldV2 {
    /// Every clip field in validation order.
    pub const ALL: [Self; 5] = [
        Self::Key,
        Self::Owner,
        Self::Parent,
        Self::Shape,
        Self::FillRule,
    ];
}

/// Closed raw paint-item field vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialPaintFieldV2 {
    /// Owning spatial-node key.
    Owner,
    /// Stable item ordinal.
    ItemOrdinal,
    /// Paint kind.
    Kind,
    /// Referenced image key.
    Image,
    /// Source horizontal origin.
    SourceX,
    /// Source vertical origin.
    SourceY,
    /// Source width.
    SourceWidth,
    /// Source height.
    SourceHeight,
    /// Destination horizontal origin.
    DestinationX,
    /// Destination vertical origin.
    DestinationY,
    /// Destination width.
    DestinationWidth,
    /// Destination height.
    DestinationHeight,
    /// Coverage kind.
    CoverageKind,
    /// Coverage shape key.
    Shape,
    /// Coverage fill rule.
    FillRule,
    /// Coverage stroke width.
    StrokeWidth,
    /// Referenced brush key.
    Brush,
    /// Paint opacity.
    Opacity,
    /// Optional clip key.
    Clip,
}

impl SpatialPaintFieldV2 {
    /// Every paint-item field in validation order.
    pub const ALL: [Self; 19] = [
        Self::Owner,
        Self::ItemOrdinal,
        Self::Kind,
        Self::Image,
        Self::SourceX,
        Self::SourceY,
        Self::SourceWidth,
        Self::SourceHeight,
        Self::DestinationX,
        Self::DestinationY,
        Self::DestinationWidth,
        Self::DestinationHeight,
        Self::CoverageKind,
        Self::Shape,
        Self::FillRule,
        Self::StrokeWidth,
        Self::Brush,
        Self::Opacity,
        Self::Clip,
    ];
}

/// Closed raw hit-item field vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialHitFieldV2 {
    /// Owning spatial-node key.
    Owner,
    /// Stable item ordinal.
    ItemOrdinal,
    /// Coverage kind.
    CoverageKind,
    /// Coverage shape key.
    Shape,
    /// Coverage fill rule.
    FillRule,
    /// Coverage stroke width.
    StrokeWidth,
    /// Optional clip key.
    Clip,
    /// Input policy.
    InputPolicy,
}

impl SpatialHitFieldV2 {
    /// Every hit-item field in validation order.
    pub const ALL: [Self; 8] = [
        Self::Owner,
        Self::ItemOrdinal,
        Self::CoverageKind,
        Self::Shape,
        Self::FillRule,
        Self::StrokeWidth,
        Self::Clip,
        Self::InputPolicy,
    ];
}

/// Closed raw semantic-item field vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialSemanticFieldV2 {
    /// Owning spatial-node key.
    Owner,
    /// Stable item ordinal.
    ItemOrdinal,
    /// Semantic shape key.
    Shape,
    /// Semantic fill rule.
    FillRule,
    /// Optional clip key.
    Clip,
}

impl SpatialSemanticFieldV2 {
    /// Every semantic-item field in validation order.
    pub const ALL: [Self; 5] = [
        Self::Owner,
        Self::ItemOrdinal,
        Self::Shape,
        Self::FillRule,
        Self::Clip,
    ];
}
