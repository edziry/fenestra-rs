//! Closed field and table vocabularies for resolved spatial output.

/// Closed resolved-output table vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialOutputTableV2 {
    /// Resolved geometry records.
    Geometry,
    /// Resolved clip records.
    Clip,
    /// Resolved paint records.
    Paint,
    /// Resolved hit records.
    Hit,
    /// Resolved semantic records.
    Semantic,
}

impl SpatialOutputTableV2 {
    /// Every resolved-output table in validation order.
    pub const ALL: [Self; 5] = [
        Self::Geometry,
        Self::Clip,
        Self::Paint,
        Self::Hit,
        Self::Semantic,
    ];
}

/// Closed resolved-output field vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialOutputFieldV2 {
    /// Stable record key.
    Key,
    /// Base-box horizontal origin.
    BaseX,
    /// Base-box vertical origin.
    BaseY,
    /// Base-box width.
    BaseWidth,
    /// Base-box height.
    BaseHeight,
    /// World-affine first matrix component.
    AffineA,
    /// World-affine second matrix component.
    AffineB,
    /// World-affine third matrix component.
    AffineC,
    /// World-affine fourth matrix component.
    AffineD,
    /// World-affine horizontal translation.
    AffineTx,
    /// World-affine vertical translation.
    AffineTy,
    /// Exact world-affine determinant.
    Determinant,
    /// Conservative-bound empty marker.
    AabbEmpty,
    /// Conservative-bound minimum horizontal coordinate.
    AabbMinX,
    /// Conservative-bound minimum vertical coordinate.
    AabbMinY,
    /// Conservative-bound maximum horizontal coordinate.
    AabbMaxX,
    /// Conservative-bound maximum vertical coordinate.
    AabbMaxY,
    /// Owning spatial-node key.
    Owner,
    /// Parent record key.
    Parent,
    /// Referenced shape key.
    Shape,
    /// Referenced brush key.
    Brush,
    /// Referenced image key.
    Image,
    /// Referenced clip key.
    Clip,
    /// Stable stack ordinal.
    StackOrdinal,
    /// Stable item ordinal.
    ItemOrdinal,
}

impl SpatialOutputFieldV2 {
    /// Every resolved-output field in validation order.
    pub const ALL: [Self; 25] = [
        Self::Key,
        Self::BaseX,
        Self::BaseY,
        Self::BaseWidth,
        Self::BaseHeight,
        Self::AffineA,
        Self::AffineB,
        Self::AffineC,
        Self::AffineD,
        Self::AffineTx,
        Self::AffineTy,
        Self::Determinant,
        Self::AabbEmpty,
        Self::AabbMinX,
        Self::AabbMinY,
        Self::AabbMaxX,
        Self::AabbMaxY,
        Self::Owner,
        Self::Parent,
        Self::Shape,
        Self::Brush,
        Self::Image,
        Self::Clip,
        Self::StackOrdinal,
        Self::ItemOrdinal,
    ];
}
