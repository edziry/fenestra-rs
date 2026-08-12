//! Raw hit and semantic geometry items.

use crate::coverage::{SpatialCoverageV2, SpatialFillRuleV2};
use crate::geometry_key::{SpatialClipKeyV2, SpatialShapeKeyV2};
use crate::model::SpatialNodeKeyV2;

/// Closed input participation policy for one hit item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialInputPolicyV2 {
    /// Participates in hit selection.
    Accept,
    /// Does not participate in hit selection.
    Ignore,
}

impl SpatialInputPolicyV2 {
    /// Every input policy in deterministic format order.
    pub const ALL: [Self; 2] = [Self::Accept, Self::Ignore];
}

/// Raw hit item owned by one spatial node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialHitV2 {
    owner: SpatialNodeKeyV2,
    item_ordinal: u32,
    coverage: SpatialCoverageV2,
    clip: Option<SpatialClipKeyV2>,
    input_policy: SpatialInputPolicyV2,
}

impl SpatialHitV2 {
    /// Creates an unvalidated hit item.
    #[must_use]
    pub const fn new(
        owner: SpatialNodeKeyV2,
        item_ordinal: u32,
        coverage: SpatialCoverageV2,
        clip: Option<SpatialClipKeyV2>,
        input_policy: SpatialInputPolicyV2,
    ) -> Self {
        Self {
            owner,
            item_ordinal,
            coverage,
            clip,
            input_policy,
        }
    }

    /// Returns the owning spatial node key.
    #[must_use]
    pub const fn owner(self) -> SpatialNodeKeyV2 {
        self.owner
    }

    /// Returns the raw owner-local item ordinal.
    #[must_use]
    pub const fn item_ordinal(self) -> u32 {
        self.item_ordinal
    }

    /// Returns the independent local hit coverage.
    #[must_use]
    pub const fn coverage(self) -> SpatialCoverageV2 {
        self.coverage
    }

    /// Returns the optional referenced clip key.
    #[must_use]
    pub const fn clip(self) -> Option<SpatialClipKeyV2> {
        self.clip
    }

    /// Returns the raw input participation policy.
    #[must_use]
    pub const fn input_policy(self) -> SpatialInputPolicyV2 {
        self.input_policy
    }
}

/// Raw semantic geometry item owned by one spatial node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialSemanticGeometryV2 {
    owner: SpatialNodeKeyV2,
    item_ordinal: u32,
    shape: SpatialShapeKeyV2,
    fill_rule: SpatialFillRuleV2,
    clip: Option<SpatialClipKeyV2>,
}

impl SpatialSemanticGeometryV2 {
    /// Creates an unvalidated semantic geometry item.
    #[must_use]
    pub const fn new(
        owner: SpatialNodeKeyV2,
        item_ordinal: u32,
        shape: SpatialShapeKeyV2,
        fill_rule: SpatialFillRuleV2,
        clip: Option<SpatialClipKeyV2>,
    ) -> Self {
        Self {
            owner,
            item_ordinal,
            shape,
            fill_rule,
            clip,
        }
    }

    /// Returns the owning spatial node key.
    #[must_use]
    pub const fn owner(self) -> SpatialNodeKeyV2 {
        self.owner
    }

    /// Returns the raw owner-local item ordinal.
    #[must_use]
    pub const fn item_ordinal(self) -> u32 {
        self.item_ordinal
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

    /// Returns the optional referenced clip key.
    #[must_use]
    pub const fn clip(self) -> Option<SpatialClipKeyV2> {
        self.clip
    }
}
