//! Closed field vocabularies for spatial geometry and resources.

/// Closed byte-channel vocabulary for decoded image evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialColorChannelV2 {
    /// Red channel.
    R,
    /// Green channel.
    G,
    /// Blue channel.
    B,
    /// Alpha channel.
    A,
}

impl SpatialColorChannelV2 {
    /// Every image channel in storage order.
    pub const ALL: [Self; 4] = [Self::R, Self::G, Self::B, Self::A];
}

/// Closed raw path-record field vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialPathFieldV2 {
    /// Dense path key.
    Key,
    /// First path-verb ordinal.
    VerbStart,
    /// Path-verb count.
    VerbLength,
}

impl SpatialPathFieldV2 {
    /// Every path field in validation order.
    pub const ALL: [Self; 3] = [Self::Key, Self::VerbStart, Self::VerbLength];
}

/// Closed raw path-verb field vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialPathVerbFieldV2 {
    /// Verb kind.
    Kind,
    /// Quadratic control horizontal coordinate.
    ControlX,
    /// Quadratic control vertical coordinate.
    ControlY,
    /// Cubic first-control horizontal coordinate.
    Control1X,
    /// Cubic first-control vertical coordinate.
    Control1Y,
    /// Cubic second-control horizontal coordinate.
    Control2X,
    /// Cubic second-control vertical coordinate.
    Control2Y,
    /// Destination horizontal coordinate.
    ToX,
    /// Destination vertical coordinate.
    ToY,
}

impl SpatialPathVerbFieldV2 {
    /// Every path-verb field in validation order.
    pub const ALL: [Self; 9] = [
        Self::Kind,
        Self::ControlX,
        Self::ControlY,
        Self::Control1X,
        Self::Control1Y,
        Self::Control2X,
        Self::Control2Y,
        Self::ToX,
        Self::ToY,
    ];
}

/// Closed raw shape-record field vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialShapeFieldV2 {
    /// Dense shape key.
    Key,
    /// Owning spatial-node key.
    Owner,
    /// Shape kind.
    Kind,
    /// Rectangle horizontal origin.
    RectX,
    /// Rectangle vertical origin.
    RectY,
    /// Rectangle width.
    RectWidth,
    /// Rectangle height.
    RectHeight,
    /// Circle-center horizontal coordinate.
    CircleCenterX,
    /// Circle-center vertical coordinate.
    CircleCenterY,
    /// Circle radius.
    CircleRadius,
    /// First polygon-point ordinal.
    PolygonPointStart,
    /// Polygon-point count.
    PolygonPointLength,
    /// Referenced path key.
    Path,
}

impl SpatialShapeFieldV2 {
    /// Every shape field in validation order.
    pub const ALL: [Self; 13] = [
        Self::Key,
        Self::Owner,
        Self::Kind,
        Self::RectX,
        Self::RectY,
        Self::RectWidth,
        Self::RectHeight,
        Self::CircleCenterX,
        Self::CircleCenterY,
        Self::CircleRadius,
        Self::PolygonPointStart,
        Self::PolygonPointLength,
        Self::Path,
    ];
}

/// Closed raw polygon-point field vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialPolygonPointFieldV2 {
    /// Horizontal coordinate.
    X,
    /// Vertical coordinate.
    Y,
}

impl SpatialPolygonPointFieldV2 {
    /// Every polygon-point field in validation order.
    pub const ALL: [Self; 2] = [Self::X, Self::Y];
}

/// Closed raw brush-record field vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialBrushFieldV2 {
    /// Dense brush key.
    Key,
    /// Brush kind.
    Kind,
    /// First gradient-stop ordinal.
    GradientStopStart,
    /// Gradient-stop count.
    GradientStopLength,
    /// Solid-color red channel.
    ColorR,
    /// Solid-color green channel.
    ColorG,
    /// Solid-color blue channel.
    ColorB,
    /// Solid-color alpha channel.
    ColorA,
    /// Gradient-start horizontal coordinate.
    GradientStartX,
    /// Gradient-start vertical coordinate.
    GradientStartY,
    /// Gradient-end horizontal coordinate.
    GradientEndX,
    /// Gradient-end vertical coordinate.
    GradientEndY,
}

impl SpatialBrushFieldV2 {
    /// Every brush field in validation order.
    pub const ALL: [Self; 12] = [
        Self::Key,
        Self::Kind,
        Self::GradientStopStart,
        Self::GradientStopLength,
        Self::ColorR,
        Self::ColorG,
        Self::ColorB,
        Self::ColorA,
        Self::GradientStartX,
        Self::GradientStartY,
        Self::GradientEndX,
        Self::GradientEndY,
    ];
}

/// Closed raw gradient-stop field vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialGradientStopFieldV2 {
    /// Stop offset.
    Offset,
    /// Red channel.
    R,
    /// Green channel.
    G,
    /// Blue channel.
    B,
    /// Alpha channel.
    A,
}

impl SpatialGradientStopFieldV2 {
    /// Every gradient-stop field in validation order.
    pub const ALL: [Self; 5] = [Self::Offset, Self::R, Self::G, Self::B, Self::A];
}

/// Closed raw decoded-image field vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialImageFieldV2 {
    /// Dense image key.
    Key,
    /// Decoded width.
    Width,
    /// Decoded height.
    Height,
    /// Row stride in bytes.
    Stride,
    /// Decoded byte length.
    ByteLength,
    /// One decoded pixel.
    Pixel,
}

impl SpatialImageFieldV2 {
    /// Every decoded-image field in validation order.
    pub const ALL: [Self; 6] = [
        Self::Key,
        Self::Width,
        Self::Height,
        Self::Stride,
        Self::ByteLength,
        Self::Pixel,
    ];
}
