//! Raw coverage and clip records.

use crate::geometry_key::{SpatialClipKeyV2, SpatialShapeKeyV2};
use crate::model::{SpatialNodeKeyV2, SpatialScalarV2};

/// Closed winding rule used by filled geometry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialFillRuleV2 {
    /// Uses nonzero winding accumulation.
    NonZero,
    /// Uses parity of winding crossings.
    EvenOdd,
}

impl SpatialFillRuleV2 {
    /// Every fill rule in deterministic format order.
    pub const ALL: [Self; 2] = [Self::NonZero, Self::EvenOdd];
}

/// Closed vocabulary for one coverage discriminant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialCoverageKindV2 {
    /// Filled shape coverage.
    Fill,
    /// Solid round-stroke coverage.
    RoundStroke,
}

impl SpatialCoverageKindV2 {
    /// Every coverage kind in deterministic format order.
    pub const ALL: [Self; 2] = [Self::Fill, Self::RoundStroke];
}

/// Raw shape coverage payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialCoverageV2 {
    /// Fills one referenced shape.
    Fill {
        /// Referenced shape key.
        shape: SpatialShapeKeyV2,
        /// Authored winding rule.
        rule: SpatialFillRuleV2,
    },
    /// Applies a solid round stroke to one shape.
    RoundStroke {
        /// Referenced shape key.
        shape: SpatialShapeKeyV2,
        /// Raw stroke width.
        width: SpatialScalarV2,
    },
}

/// Raw clip record owned by one spatial node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialClipV2 {
    key: SpatialClipKeyV2,
    owner: SpatialNodeKeyV2,
    parent: Option<SpatialClipKeyV2>,
    shape: SpatialShapeKeyV2,
    fill_rule: SpatialFillRuleV2,
}

impl SpatialClipV2 {
    /// Creates an unvalidated clip record.
    #[must_use]
    pub const fn new(
        key: SpatialClipKeyV2,
        owner: SpatialNodeKeyV2,
        parent: Option<SpatialClipKeyV2>,
        shape: SpatialShapeKeyV2,
        fill_rule: SpatialFillRuleV2,
    ) -> Self {
        Self {
            key,
            owner,
            parent,
            shape,
            fill_rule,
        }
    }

    /// Returns the dense clip key.
    #[must_use]
    pub const fn key(self) -> SpatialClipKeyV2 {
        self.key
    }

    /// Returns the owning spatial node key.
    #[must_use]
    pub const fn owner(self) -> SpatialNodeKeyV2 {
        self.owner
    }

    /// Returns the optional earlier clip key.
    #[must_use]
    pub const fn parent(self) -> Option<SpatialClipKeyV2> {
        self.parent
    }

    /// Returns the referenced shape key.
    #[must_use]
    pub const fn shape(self) -> SpatialShapeKeyV2 {
        self.shape
    }

    /// Returns the authored fill rule.
    #[must_use]
    pub const fn fill_rule(self) -> SpatialFillRuleV2 {
        self.fill_rule
    }
}
