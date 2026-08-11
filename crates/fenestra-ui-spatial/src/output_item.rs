//! Raw candidate paint, hit, and semantic output records.

use crate::content_key::{SpatialBrushKeyV2, SpatialImageKeyV2};
use crate::geometry_key::{SpatialClipKeyV2, SpatialShapeKeyV2};
use crate::model::{Affine2V2, SpatialNodeKeyV2};
use crate::output_aabb::SpatialOutputAabbV2;

/// Raw resource reference for one candidate paint output record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialPaintOutputReferenceV2 {
    /// Shape coverage painted with one brush.
    Coverage {
        /// Referenced shape key.
        shape: SpatialShapeKeyV2,
        /// Referenced brush key.
        brush: SpatialBrushKeyV2,
    },
    /// Image paint using one image resource.
    Image {
        /// Referenced image key.
        image: SpatialImageKeyV2,
    },
}

/// Unvalidated candidate output for one paint item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialPaintOutputRecordV2 {
    key: u32,
    world_from_local: Affine2V2,
    world_determinant: i128,
    world_aabb: SpatialOutputAabbV2,
    owner: SpatialNodeKeyV2,
    reference: SpatialPaintOutputReferenceV2,
    clip: Option<SpatialClipKeyV2>,
    stack_ordinal: u32,
    item_ordinal: u32,
}

impl SpatialPaintOutputRecordV2 {
    /// Creates an unvalidated candidate paint record.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        key: u32,
        world_from_local: Affine2V2,
        world_determinant: i128,
        world_aabb: SpatialOutputAabbV2,
        owner: SpatialNodeKeyV2,
        reference: SpatialPaintOutputReferenceV2,
        clip: Option<SpatialClipKeyV2>,
        stack_ordinal: u32,
        item_ordinal: u32,
    ) -> Self {
        Self {
            key,
            world_from_local,
            world_determinant,
            world_aabb,
            owner,
            reference,
            clip,
            stack_ordinal,
            item_ordinal,
        }
    }

    /// Returns the raw dense paint-row key.
    #[must_use]
    pub const fn key(self) -> u32 {
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

    /// Returns the raw unclipped world AABB.
    #[must_use]
    pub const fn world_aabb(self) -> SpatialOutputAabbV2 {
        self.world_aabb
    }

    /// Returns the raw owner node key.
    #[must_use]
    pub const fn owner(self) -> SpatialNodeKeyV2 {
        self.owner
    }

    /// Returns the raw paint resource reference.
    #[must_use]
    pub const fn reference(self) -> SpatialPaintOutputReferenceV2 {
        self.reference
    }

    /// Returns the raw optional terminal clip key.
    #[must_use]
    pub const fn clip(self) -> Option<SpatialClipKeyV2> {
        self.clip
    }

    /// Returns the raw stack ordinal.
    #[must_use]
    pub const fn stack_ordinal(self) -> u32 {
        self.stack_ordinal
    }

    /// Returns the raw owner-local item ordinal.
    #[must_use]
    pub const fn item_ordinal(self) -> u32 {
        self.item_ordinal
    }
}

/// Unvalidated candidate output for one hit item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialHitOutputRecordV2 {
    key: u32,
    world_from_local: Affine2V2,
    world_determinant: i128,
    world_aabb: SpatialOutputAabbV2,
    owner: SpatialNodeKeyV2,
    shape: SpatialShapeKeyV2,
    clip: Option<SpatialClipKeyV2>,
    stack_ordinal: u32,
    item_ordinal: u32,
}

impl SpatialHitOutputRecordV2 {
    /// Creates an unvalidated candidate hit record.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        key: u32,
        world_from_local: Affine2V2,
        world_determinant: i128,
        world_aabb: SpatialOutputAabbV2,
        owner: SpatialNodeKeyV2,
        shape: SpatialShapeKeyV2,
        clip: Option<SpatialClipKeyV2>,
        stack_ordinal: u32,
        item_ordinal: u32,
    ) -> Self {
        Self {
            key,
            world_from_local,
            world_determinant,
            world_aabb,
            owner,
            shape,
            clip,
            stack_ordinal,
            item_ordinal,
        }
    }

    /// Returns the raw dense hit-row key.
    #[must_use]
    pub const fn key(self) -> u32 {
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

    /// Returns the raw unclipped world AABB.
    #[must_use]
    pub const fn world_aabb(self) -> SpatialOutputAabbV2 {
        self.world_aabb
    }

    /// Returns the raw owner node key.
    #[must_use]
    pub const fn owner(self) -> SpatialNodeKeyV2 {
        self.owner
    }

    /// Returns the raw shape reference.
    #[must_use]
    pub const fn shape(self) -> SpatialShapeKeyV2 {
        self.shape
    }

    /// Returns the raw optional terminal clip key.
    #[must_use]
    pub const fn clip(self) -> Option<SpatialClipKeyV2> {
        self.clip
    }

    /// Returns the raw stack ordinal.
    #[must_use]
    pub const fn stack_ordinal(self) -> u32 {
        self.stack_ordinal
    }

    /// Returns the raw owner-local item ordinal.
    #[must_use]
    pub const fn item_ordinal(self) -> u32 {
        self.item_ordinal
    }
}

/// Unvalidated candidate output for one semantic item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialSemanticOutputRecordV2 {
    key: u32,
    world_from_local: Affine2V2,
    world_determinant: i128,
    world_aabb: SpatialOutputAabbV2,
    owner: SpatialNodeKeyV2,
    shape: SpatialShapeKeyV2,
    clip: Option<SpatialClipKeyV2>,
    stack_ordinal: u32,
    item_ordinal: u32,
}

impl SpatialSemanticOutputRecordV2 {
    /// Creates an unvalidated candidate semantic record.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        key: u32,
        world_from_local: Affine2V2,
        world_determinant: i128,
        world_aabb: SpatialOutputAabbV2,
        owner: SpatialNodeKeyV2,
        shape: SpatialShapeKeyV2,
        clip: Option<SpatialClipKeyV2>,
        stack_ordinal: u32,
        item_ordinal: u32,
    ) -> Self {
        Self {
            key,
            world_from_local,
            world_determinant,
            world_aabb,
            owner,
            shape,
            clip,
            stack_ordinal,
            item_ordinal,
        }
    }

    /// Returns the raw dense semantic-row key.
    #[must_use]
    pub const fn key(self) -> u32 {
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

    /// Returns the raw unclipped world AABB.
    #[must_use]
    pub const fn world_aabb(self) -> SpatialOutputAabbV2 {
        self.world_aabb
    }

    /// Returns the raw owner node key.
    #[must_use]
    pub const fn owner(self) -> SpatialNodeKeyV2 {
        self.owner
    }

    /// Returns the raw shape reference.
    #[must_use]
    pub const fn shape(self) -> SpatialShapeKeyV2 {
        self.shape
    }

    /// Returns the raw optional terminal clip key.
    #[must_use]
    pub const fn clip(self) -> Option<SpatialClipKeyV2> {
        self.clip
    }

    /// Returns the raw stack ordinal.
    #[must_use]
    pub const fn stack_ordinal(self) -> u32 {
        self.stack_ordinal
    }

    /// Returns the raw owner-local item ordinal.
    #[must_use]
    pub const fn item_ordinal(self) -> u32 {
        self.item_ordinal
    }
}
