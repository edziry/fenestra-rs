//! Closed leaf vocabularies for raw spatial-content diagnostics.

use crate::vocabulary::SpatialExtentV2;

/// Closed keyed-content table vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialKeyedContentTableV2 {
    /// Path table.
    Path,
    /// Shape table.
    Shape,
    /// Brush table.
    Brush,
    /// Image table.
    Image,
    /// Clip table.
    Clip,
}

impl SpatialKeyedContentTableV2 {
    /// Every keyed-content table in validation order.
    pub const ALL: [Self; 5] = [
        Self::Path,
        Self::Shape,
        Self::Brush,
        Self::Image,
        Self::Clip,
    ];
}

/// Closed unkeyed payload-table vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialPayloadTableV2 {
    /// Path-verb table.
    PathVerb,
    /// Polygon-point table.
    PolygonPoint,
    /// Gradient-stop table.
    GradientStop,
}

impl SpatialPayloadTableV2 {
    /// Every unkeyed payload table in validation order.
    pub const ALL: [Self; 3] = [Self::PathVerb, Self::PolygonPoint, Self::GradientStop];
}

/// Closed authored-reference target vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialContentReferenceV2 {
    /// Path reference.
    Path,
    /// Shape reference.
    Shape,
    /// Brush reference.
    Brush,
    /// Image reference.
    Image,
    /// Clip reference.
    Clip,
    /// Owning spatial-node reference.
    Owner,
}

impl SpatialContentReferenceV2 {
    /// Every authored-reference target in validation order.
    pub const ALL: [Self; 6] = [
        Self::Path,
        Self::Shape,
        Self::Brush,
        Self::Image,
        Self::Clip,
        Self::Owner,
    ];
}

/// Closed ordered-item table vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialOrderedItemTableV2 {
    /// Paint-item table.
    Paint,
    /// Hit-item table.
    Hit,
    /// Semantic-item table.
    Semantic,
}

impl SpatialOrderedItemTableV2 {
    /// Every ordered-item table in validation order.
    pub const ALL: [Self; 3] = [Self::Paint, Self::Hit, Self::Semantic];
}

/// Closed path-grammar failure vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialPathGrammarErrorV2 {
    /// The path has no verbs.
    Empty,
    /// The first verb is not a move.
    FirstNotMove,
    /// A subpath has no drawn segment.
    EmptySubpath,
    /// A drawing verb has no open subpath.
    DrawingWithoutSubpath,
    /// A close verb follows no segment.
    CloseWithoutSegment,
    /// The path ends with a move.
    TrailingMove,
}

impl SpatialPathGrammarErrorV2 {
    /// Every path-grammar failure in validation order.
    pub const ALL: [Self; 6] = [
        Self::Empty,
        Self::FirstNotMove,
        Self::EmptySubpath,
        Self::DrawingWithoutSubpath,
        Self::CloseWithoutSegment,
        Self::TrailingMove,
    ];
}

/// Closed shape failure vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialShapeErrorV2 {
    /// A rectangular extent is negative.
    NegativeExtent,
    /// A circle radius is negative.
    NegativeRadius,
    /// A polygon contains too few points.
    PolygonTooShort,
    /// A polygon repeats its first point at the end.
    PolygonRepeatedFirst,
    /// A polygon has equal adjacent points.
    PolygonAdjacentEqual,
}

impl SpatialShapeErrorV2 {
    /// Every shape failure in validation order.
    pub const ALL: [Self; 5] = [
        Self::NegativeExtent,
        Self::NegativeRadius,
        Self::PolygonTooShort,
        Self::PolygonRepeatedFirst,
        Self::PolygonAdjacentEqual,
    ];
}

/// Closed stroke failure vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialStrokeErrorV2 {
    /// Stroke width is negative.
    NegativeWidth,
    /// Stroke width is zero.
    ZeroWidth,
}

impl SpatialStrokeErrorV2 {
    /// Every stroke failure in validation order.
    pub const ALL: [Self; 2] = [Self::NegativeWidth, Self::ZeroWidth];
}

/// Closed gradient failure vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialGradientErrorV2 {
    /// Gradient endpoints coincide.
    CoincidentEndpoints,
    /// The gradient has too few stops.
    TooFewStops,
    /// The first stop has an invalid offset.
    FirstOffset,
    /// The last stop has an invalid offset.
    LastOffset,
    /// Stop offsets decrease.
    DecreasingOffset,
}

impl SpatialGradientErrorV2 {
    /// Every gradient failure in validation order.
    pub const ALL: [Self; 5] = [
        Self::CoincidentEndpoints,
        Self::TooFewStops,
        Self::FirstOffset,
        Self::LastOffset,
        Self::DecreasingOffset,
    ];
}

/// Closed decoded-image and image-paint failure vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialImageErrorV2 {
    /// One decoded extent is zero.
    ZeroExtent,
    /// Row stride differs from packed RGBA8 storage.
    StrideMismatch,
    /// Byte length differs from the decoded layout.
    LengthMismatch,
    /// One RGBA8 pixel is not premultiplied.
    InvalidPremultipliedPixel,
    /// The source rectangle is empty.
    EmptySource,
    /// The source rectangle exceeds the image.
    SourceOutOfBounds,
    /// One destination extent is negative.
    NegativeDestinationExtent(SpatialExtentV2),
    /// The destination rectangle is empty.
    EmptyDestination,
}

impl SpatialImageErrorV2 {
    /// Every image failure in validation order.
    pub const ALL: [Self; 9] = [
        Self::ZeroExtent,
        Self::StrideMismatch,
        Self::LengthMismatch,
        Self::InvalidPremultipliedPixel,
        Self::EmptySource,
        Self::SourceOutOfBounds,
        Self::NegativeDestinationExtent(SpatialExtentV2::Width),
        Self::NegativeDestinationExtent(SpatialExtentV2::Height),
        Self::EmptyDestination,
    ];
}

/// Closed clip failure vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialClipErrorV2 {
    /// A clip refers to a later parent clip.
    ForwardParent,
    /// A clip and its shape have different owners.
    ShapeOwnerMismatch,
    /// A parent clip owner is not an ancestor.
    OwnerNotAncestor,
    /// An item owner is not a clip-owner descendant.
    ItemOwnerNotDescendant,
}

impl SpatialClipErrorV2 {
    /// Every clip failure in validation order.
    pub const ALL: [Self; 4] = [
        Self::ForwardParent,
        Self::ShapeOwnerMismatch,
        Self::OwnerNotAncestor,
        Self::ItemOwnerNotDescendant,
    ];
}
