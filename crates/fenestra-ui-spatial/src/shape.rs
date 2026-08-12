//! Raw local shape records.

use crate::geometry_key::{SpatialPathKeyV2, SpatialShapeKeyV2};
use crate::model::{SpatialNodeKeyV2, SpatialPointV2, SpatialScalarV2};

/// Closed vocabulary for one shape discriminant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialShapeKindV2 {
    /// Axis-aligned rectangle.
    Rect,
    /// Circle.
    Circle,
    /// Polygon point range.
    Polygon,
    /// Path reference.
    Path,
}

impl SpatialShapeKindV2 {
    /// Every shape kind in deterministic format order.
    pub const ALL: [Self; 4] = [Self::Rect, Self::Circle, Self::Polygon, Self::Path];
}

/// Raw local geometry payload for one shape.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialShapeGeometryV2 {
    /// Axis-aligned rectangle geometry.
    Rect {
        /// Local near corner.
        origin: SpatialPointV2,
        /// Raw horizontal extent.
        width: SpatialScalarV2,
        /// Raw vertical extent.
        height: SpatialScalarV2,
    },
    /// Circle geometry.
    Circle {
        /// Local center point.
        center: SpatialPointV2,
        /// Raw circle radius.
        radius: SpatialScalarV2,
    },
    /// Polygon point range.
    Polygon {
        /// First supplied point ordinal.
        point_start: u32,
        /// Supplied point count.
        point_length: u32,
    },
    /// Reference to one supplied path.
    Path {
        /// Referenced path key.
        path: SpatialPathKeyV2,
    },
}

/// Raw shape record owned by one spatial node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialShapeV2 {
    key: SpatialShapeKeyV2,
    owner: SpatialNodeKeyV2,
    geometry: SpatialShapeGeometryV2,
}

impl SpatialShapeV2 {
    /// Creates an unvalidated shape record.
    #[must_use]
    pub const fn new(
        key: SpatialShapeKeyV2,
        owner: SpatialNodeKeyV2,
        geometry: SpatialShapeGeometryV2,
    ) -> Self {
        Self {
            key,
            owner,
            geometry,
        }
    }

    /// Returns the dense shape key.
    #[must_use]
    pub const fn key(self) -> SpatialShapeKeyV2 {
        self.key
    }

    /// Returns the owning spatial node key.
    #[must_use]
    pub const fn owner(self) -> SpatialNodeKeyV2 {
        self.owner
    }

    /// Returns the raw local geometry payload.
    #[must_use]
    pub const fn geometry(self) -> SpatialShapeGeometryV2 {
        self.geometry
    }
}
