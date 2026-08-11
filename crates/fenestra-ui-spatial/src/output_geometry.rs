//! Raw candidate geometry and clip output records.

use crate::geometry_key::{SpatialClipKeyV2, SpatialShapeKeyV2};
use crate::model::{Affine2V2, SpatialNodeKeyV2, SpatialScalarV2};
use crate::output_aabb::SpatialOutputAabbV2;

/// Unvalidated candidate output for one geometry node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialGeometryOutputRecordV2 {
    key: SpatialNodeKeyV2,
    base_x: SpatialScalarV2,
    base_y: SpatialScalarV2,
    base_width: SpatialScalarV2,
    base_height: SpatialScalarV2,
    world_from_local: Affine2V2,
    world_determinant: i128,
    world_aabb: SpatialOutputAabbV2,
}

impl SpatialGeometryOutputRecordV2 {
    /// Creates an unvalidated candidate geometry record.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        key: SpatialNodeKeyV2,
        base_x: SpatialScalarV2,
        base_y: SpatialScalarV2,
        base_width: SpatialScalarV2,
        base_height: SpatialScalarV2,
        world_from_local: Affine2V2,
        world_determinant: i128,
        world_aabb: SpatialOutputAabbV2,
    ) -> Self {
        Self {
            key,
            base_x,
            base_y,
            base_width,
            base_height,
            world_from_local,
            world_determinant,
            world_aabb,
        }
    }

    /// Returns the raw dense geometry key.
    #[must_use]
    pub const fn key(self) -> SpatialNodeKeyV2 {
        self.key
    }

    /// Returns the raw base horizontal origin.
    #[must_use]
    pub const fn base_x(self) -> SpatialScalarV2 {
        self.base_x
    }

    /// Returns the raw base vertical origin.
    #[must_use]
    pub const fn base_y(self) -> SpatialScalarV2 {
        self.base_y
    }

    /// Returns the raw base horizontal extent.
    #[must_use]
    pub const fn base_width(self) -> SpatialScalarV2 {
        self.base_width
    }

    /// Returns the raw base vertical extent.
    #[must_use]
    pub const fn base_height(self) -> SpatialScalarV2 {
        self.base_height
    }

    /// Returns the raw world-from-local affine.
    #[must_use]
    pub const fn world_from_local(self) -> Affine2V2 {
        self.world_from_local
    }

    /// Returns the supplied widened world determinant.
    #[must_use]
    pub const fn world_determinant(self) -> i128 {
        self.world_determinant
    }

    /// Returns the raw projected world AABB.
    #[must_use]
    pub const fn world_aabb(self) -> SpatialOutputAabbV2 {
        self.world_aabb
    }
}

/// Unvalidated candidate output for one clip primitive.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialClipOutputRecordV2 {
    key: SpatialClipKeyV2,
    world_from_local: Affine2V2,
    world_determinant: i128,
    primitive_world_aabb: SpatialOutputAabbV2,
    owner: SpatialNodeKeyV2,
    parent: Option<SpatialClipKeyV2>,
    shape: SpatialShapeKeyV2,
}

impl SpatialClipOutputRecordV2 {
    /// Creates an unvalidated candidate clip record.
    #[must_use]
    pub const fn new(
        key: SpatialClipKeyV2,
        world_from_local: Affine2V2,
        world_determinant: i128,
        primitive_world_aabb: SpatialOutputAabbV2,
        owner: SpatialNodeKeyV2,
        parent: Option<SpatialClipKeyV2>,
        shape: SpatialShapeKeyV2,
    ) -> Self {
        Self {
            key,
            world_from_local,
            world_determinant,
            primitive_world_aabb,
            owner,
            parent,
            shape,
        }
    }

    /// Returns the raw dense clip key.
    #[must_use]
    pub const fn key(self) -> SpatialClipKeyV2 {
        self.key
    }

    /// Returns the raw owner world-from-local affine.
    #[must_use]
    pub const fn world_from_local(self) -> Affine2V2 {
        self.world_from_local
    }

    /// Returns the supplied widened world determinant.
    #[must_use]
    pub const fn world_determinant(self) -> i128 {
        self.world_determinant
    }

    /// Returns the raw primitive projected AABB before parent intersection.
    #[must_use]
    pub const fn primitive_world_aabb(self) -> SpatialOutputAabbV2 {
        self.primitive_world_aabb
    }

    /// Returns the raw owner node key.
    #[must_use]
    pub const fn owner(self) -> SpatialNodeKeyV2 {
        self.owner
    }

    /// Returns the raw optional parent clip key.
    #[must_use]
    pub const fn parent(self) -> Option<SpatialClipKeyV2> {
        self.parent
    }

    /// Returns the raw shape reference.
    #[must_use]
    pub const fn shape(self) -> SpatialShapeKeyV2 {
        self.shape
    }
}
