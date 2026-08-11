use fenestra_ui_layout::prototype::{LayoutAxisV1, LayoutDimensionV1, LayoutPaddingV1};

use crate::model::{
    SpatialAnchorTargetV2, SpatialAnchorV2, SpatialLocalTransformV2, SpatialNodeKeyV2,
    SpatialOffsetV2, SpatialViewportV2,
};

/// Child-layout style owned by one spatial node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialContainerV2 {
    axis: LayoutAxisV1,
    padding: LayoutPaddingV1,
    gap: i32,
}

impl SpatialContainerV2 {
    /// Creates one raw child-layout container style.
    #[must_use]
    pub const fn new(axis: LayoutAxisV1, padding: LayoutPaddingV1, gap: i32) -> Self {
        Self { axis, padding, gap }
    }

    /// Returns the child stack axis.
    #[must_use]
    pub const fn axis(self) -> LayoutAxisV1 {
        self.axis
    }

    /// Returns the child container padding.
    #[must_use]
    pub const fn padding(self) -> LayoutPaddingV1 {
        self.padding
    }

    /// Returns the inter-child gap.
    #[must_use]
    pub const fn gap(self) -> i32 {
        self.gap
    }
}

/// Raw dimensions and transform for a layout-participating node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialLayoutPlacementV2 {
    width: LayoutDimensionV1,
    height: LayoutDimensionV1,
    transform: SpatialLocalTransformV2,
}

impl SpatialLayoutPlacementV2 {
    /// Creates one raw layout placement.
    #[must_use]
    pub const fn new(
        width: LayoutDimensionV1,
        height: LayoutDimensionV1,
        transform: SpatialLocalTransformV2,
    ) -> Self {
        Self {
            width,
            height,
            transform,
        }
    }

    /// Returns the layout width constraints.
    #[must_use]
    pub const fn width(self) -> LayoutDimensionV1 {
        self.width
    }

    /// Returns the layout height constraints.
    #[must_use]
    pub const fn height(self) -> LayoutDimensionV1 {
        self.height
    }

    /// Returns the node's local transform.
    #[must_use]
    pub const fn transform(self) -> SpatialLocalTransformV2 {
        self.transform
    }
}

/// Raw size, anchors, offset, and transform for a freely placed node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialFreePlacementV2 {
    width: i32,
    height: i32,
    self_anchor: SpatialAnchorV2,
    target: SpatialAnchorTargetV2,
    target_anchor: SpatialAnchorV2,
    offset: SpatialOffsetV2,
    transform: SpatialLocalTransformV2,
}

impl SpatialFreePlacementV2 {
    /// Creates one raw free placement.
    #[must_use]
    pub const fn new(
        width: i32,
        height: i32,
        self_anchor: SpatialAnchorV2,
        target: SpatialAnchorTargetV2,
        target_anchor: SpatialAnchorV2,
        offset: SpatialOffsetV2,
        transform: SpatialLocalTransformV2,
    ) -> Self {
        Self {
            width,
            height,
            self_anchor,
            target,
            target_anchor,
            offset,
            transform,
        }
    }

    /// Returns the raw free-placement width.
    #[must_use]
    pub const fn width(self) -> i32 {
        self.width
    }

    /// Returns the raw free-placement height.
    #[must_use]
    pub const fn height(self) -> i32 {
        self.height
    }

    /// Returns the anchor on this node's own box.
    #[must_use]
    pub const fn self_anchor(self) -> SpatialAnchorV2 {
        self.self_anchor
    }

    /// Returns the anchor target.
    #[must_use]
    pub const fn target(self) -> SpatialAnchorTargetV2 {
        self.target
    }

    /// Returns the anchor component on the target box.
    #[must_use]
    pub const fn target_anchor(self) -> SpatialAnchorV2 {
        self.target_anchor
    }

    /// Returns the authored placement offset.
    #[must_use]
    pub const fn offset(self) -> SpatialOffsetV2 {
        self.offset
    }

    /// Returns the node's local transform.
    #[must_use]
    pub const fn transform(self) -> SpatialLocalTransformV2 {
        self.transform
    }
}

/// Closed placement kind vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialPlacementKindV2 {
    /// Synthetic viewport sentinel placement.
    Root,
    /// Automatic layout participation.
    Layout,
    /// Free two-dimensional placement.
    Free,
}

impl SpatialPlacementKindV2 {
    /// Every placement kind in deterministic vocabulary order.
    pub const ALL: [Self; 3] = [Self::Root, Self::Layout, Self::Free];
}

/// Authored placement for one spatial node.
///
/// Payload enums intentionally have no fieldless `ALL` array.
///
/// ```compile_fail,E0599
/// use fenestra_ui_spatial::prototype::SpatialPlacementV2;
///
/// let _ = SpatialPlacementV2::ALL;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialPlacementV2 {
    /// Synthetic viewport sentinel placement.
    Root,
    /// Participates in its incoming layout island.
    Layout(SpatialLayoutPlacementV2),
    /// Resolves through anchors and coordinates.
    Free(SpatialFreePlacementV2),
}

/// One raw spatial node in authored preorder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialNodeV2 {
    key: SpatialNodeKeyV2,
    parent: Option<SpatialNodeKeyV2>,
    placement: SpatialPlacementV2,
    container: SpatialContainerV2,
}

impl SpatialNodeV2 {
    /// Creates one raw authored-preorder spatial node.
    #[must_use]
    pub const fn new(
        key: SpatialNodeKeyV2,
        parent: Option<SpatialNodeKeyV2>,
        placement: SpatialPlacementV2,
        container: SpatialContainerV2,
    ) -> Self {
        Self {
            key,
            parent,
            placement,
            container,
        }
    }

    /// Returns the dense pass-local key.
    #[must_use]
    pub const fn key(self) -> SpatialNodeKeyV2 {
        self.key
    }

    /// Returns the spatial parent, or `None` for the sentinel.
    #[must_use]
    pub const fn parent(self) -> Option<SpatialNodeKeyV2> {
        self.parent
    }

    /// Returns the incoming placement.
    #[must_use]
    pub const fn placement(self) -> SpatialPlacementV2 {
        self.placement
    }

    /// Returns the child-layout container style.
    #[must_use]
    pub const fn container(self) -> SpatialContainerV2 {
        self.container
    }
}

/// Borrowed raw topology input for one spatial computation.
#[derive(Clone, Copy)]
pub struct SpatialTopologyInputV2<'a> {
    viewport: SpatialViewportV2,
    nodes: &'a [SpatialNodeV2],
}

impl<'a> SpatialTopologyInputV2<'a> {
    /// Creates one borrowed topology input.
    #[must_use]
    pub const fn new(viewport: SpatialViewportV2, nodes: &'a [SpatialNodeV2]) -> Self {
        Self { viewport, nodes }
    }

    /// Returns the logical viewport.
    #[must_use]
    pub const fn viewport(self) -> SpatialViewportV2 {
        self.viewport
    }

    /// Returns authored nodes in supplied preorder.
    #[must_use]
    pub const fn nodes(self) -> &'a [SpatialNodeV2] {
        self.nodes
    }
}
