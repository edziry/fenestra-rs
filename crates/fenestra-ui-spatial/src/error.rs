use fenestra_ui_layout::prototype::{LayoutConstraintFieldV1, LayoutExtentV1, LayoutPaddingSideV1};

use crate::geometry_field::{
    SpatialBrushFieldV2, SpatialColorChannelV2, SpatialGradientStopFieldV2, SpatialImageFieldV2,
    SpatialPathFieldV2, SpatialPathVerbFieldV2, SpatialPolygonPointFieldV2, SpatialShapeFieldV2,
};
use crate::item_field::{
    SpatialClipFieldV2, SpatialHitFieldV2, SpatialPaintFieldV2, SpatialSemanticFieldV2,
};
use crate::output_field::{SpatialOutputFieldV2, SpatialOutputTableV2};
use crate::vocabulary::{SpatialAxisV2, SpatialExtentV2, SpatialNodeFieldV2};

/// Closed container failure vocabulary for raw spatial input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialContainerErrorKindV2 {
    /// One padding side is negative.
    NegativePadding(LayoutPaddingSideV1),
    /// Resolved padding exceeds its box extent.
    PaddingExceedsExtent(LayoutExtentV1),
    /// The inter-child gap is negative.
    NegativeGap,
}

impl SpatialContainerErrorKindV2 {
    /// Every container failure in deterministic validation order.
    pub const ALL: [Self; 7] = [
        Self::NegativePadding(LayoutPaddingSideV1::Left),
        Self::NegativePadding(LayoutPaddingSideV1::Right),
        Self::NegativePadding(LayoutPaddingSideV1::Top),
        Self::NegativePadding(LayoutPaddingSideV1::Bottom),
        Self::PaddingExceedsExtent(LayoutExtentV1::Width),
        Self::PaddingExceedsExtent(LayoutExtentV1::Height),
        Self::NegativeGap,
    ];
}

/// Closed layout-dimension failure vocabulary for raw spatial input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialLayoutDimensionErrorKindV2 {
    /// One dimension constraint is negative.
    NegativeConstraint {
        /// Logical extent containing the constraint.
        extent: LayoutExtentV1,
        /// Constraint field that failed.
        field: LayoutConstraintFieldV1,
    },
    /// A dimension has an inverted minimum and maximum.
    InvertedConstraint(LayoutExtentV1),
}

impl SpatialLayoutDimensionErrorKindV2 {
    /// Every dimension failure in deterministic validation order.
    pub const ALL: [Self; 8] = dimension_error_kinds();
}

/// Closed raw-input failure vocabulary for spatial topology.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialInputErrorKindV2 {
    /// The input has no sentinel record.
    EmptyInput,
    /// The sentinel key is not zero.
    InvalidRootKey,
    /// The sentinel declares a spatial parent.
    RootHasParent,
    /// The sentinel does not use root placement.
    InvalidRootPlacement,
    /// A supplied spatial key is not its dense ordinal.
    NonDenseNodeKey,
    /// A supplied spatial parent key is absent.
    MissingSpatialParent,
    /// A supplied spatial parent is not earlier in preorder.
    ForwardSpatialParent,
    /// Authored preorder attempts to reopen a closed subtree.
    InvalidPreorder,
    /// A non-sentinel node uses root placement.
    RootPlacementOnNonRoot,
    /// One logical viewport extent is negative.
    NegativeViewport(SpatialExtentV2),
    /// One freely placed extent is negative.
    NegativeFreeExtent(SpatialExtentV2),
    /// One free-placement offset lies outside the scalar domain.
    FreeOffsetOutOfDomain(SpatialAxisV2),
    /// One node has an invalid child-layout container.
    InvalidContainer(SpatialContainerErrorKindV2),
    /// One layout placement has invalid dimension constraints.
    InvalidLayoutDimensions(SpatialLayoutDimensionErrorKindV2),
}

impl SpatialInputErrorKindV2 {
    /// Every raw-input failure in deterministic validation order.
    pub const ALL: [Self; 30] = input_error_kinds();
}

/// Closed dependency-graph failure vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialDependencyErrorKindV2 {
    /// An anchor target key is absent.
    MissingTarget,
    /// A node attempts to target the synthetic sentinel by key.
    SentinelNodeTarget,
    /// A node attempts to target itself.
    SelfTarget,
    /// Placement dependencies contain a cycle.
    Cycle,
}

impl SpatialDependencyErrorKindV2 {
    /// Every dependency failure in deterministic validation order.
    pub const ALL: [Self; 4] = [
        Self::MissingTarget,
        Self::SentinelNodeTarget,
        Self::SelfTarget,
        Self::Cycle,
    ];
}

/// Trusted location for aggregate spatial diagnostics.
///
/// Indexed locations intentionally have no finite `ALL` array.
///
/// ```compile_fail,E0599
/// use fenestra_ui_spatial::prototype::SpatialErrorLocationV2;
///
/// let _ = SpatialErrorLocationV2::ALL;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialErrorLocationV2 {
    /// Whole spatial input.
    Input,
    /// One logical viewport extent.
    Viewport {
        /// Extent selected by validation order.
        extent: SpatialExtentV2,
    },
    /// One supplied node ordinal without a raw-field attribution.
    Node {
        /// Trusted node ordinal.
        index: u32,
    },
    /// One raw field on a supplied node ordinal.
    NodeField {
        /// Trusted node ordinal.
        index: u32,
        /// Closed field selected by validation order.
        field: SpatialNodeFieldV2,
    },
    /// One derived layout island index.
    Island {
        /// Trusted dense island index after stable-key ordering.
        index: u32,
    },
    /// One dependency unit selected by its stable ordinal.
    Dependency {
        /// Lowest spatial key produced by the unit.
        ordinal: u32,
    },
    /// One raw path record and field.
    Path {
        /// Trusted path ordinal.
        index: u32,
        /// Closed path field selected by validation order.
        field: SpatialPathFieldV2,
    },
    /// One raw path-verb record and field.
    PathVerb {
        /// Trusted containing path ordinal.
        path: u32,
        /// Trusted path-verb ordinal.
        verb: u32,
        /// Closed path-verb field selected by validation order.
        field: SpatialPathVerbFieldV2,
    },
    /// One raw shape record and field.
    Shape {
        /// Trusted shape ordinal.
        index: u32,
        /// Closed shape field selected by validation order.
        field: SpatialShapeFieldV2,
    },
    /// One raw polygon point and field.
    PolygonPoint {
        /// Trusted containing shape ordinal.
        shape: u32,
        /// Trusted polygon-point ordinal.
        point: u32,
        /// Closed polygon-point field selected by validation order.
        field: SpatialPolygonPointFieldV2,
    },
    /// One raw brush record and field.
    Brush {
        /// Trusted brush ordinal.
        index: u32,
        /// Closed brush field selected by validation order.
        field: SpatialBrushFieldV2,
    },
    /// One raw gradient stop and field.
    GradientStop {
        /// Trusted containing brush ordinal.
        brush: u32,
        /// Trusted gradient-stop ordinal.
        stop: u32,
        /// Closed gradient-stop field selected by validation order.
        field: SpatialGradientStopFieldV2,
    },
    /// One raw decoded-image record and field.
    Image {
        /// Trusted image ordinal.
        index: u32,
        /// Closed image field selected by validation order.
        field: SpatialImageFieldV2,
    },
    /// One decoded image pixel and channel.
    ImagePixel {
        /// Trusted image ordinal.
        image: u32,
        /// Widened decoded-pixel ordinal.
        pixel: u128,
        /// Closed channel selected by validation order.
        channel: SpatialColorChannelV2,
    },
    /// One raw clip record and field.
    Clip {
        /// Trusted clip ordinal.
        index: u32,
        /// Closed clip field selected by validation order.
        field: SpatialClipFieldV2,
    },
    /// One raw paint-item record and field.
    Paint {
        /// Trusted paint-item ordinal.
        index: u32,
        /// Closed paint-item field selected by validation order.
        field: SpatialPaintFieldV2,
    },
    /// One raw hit-item record and field.
    Hit {
        /// Trusted hit-item ordinal.
        index: u32,
        /// Closed hit-item field selected by validation order.
        field: SpatialHitFieldV2,
    },
    /// One raw semantic-item record and field.
    Semantic {
        /// Trusted semantic-item ordinal.
        index: u32,
        /// Closed semantic-item field selected by validation order.
        field: SpatialSemanticFieldV2,
    },
    /// One resolved-output table.
    Output {
        /// Closed output table selected by validation order.
        table: SpatialOutputTableV2,
    },
    /// One resolved-output record and field.
    OutputRecord {
        /// Closed output table selected by validation order.
        table: SpatialOutputTableV2,
        /// Trusted output-record ordinal.
        index: u32,
        /// Closed output field selected by validation order.
        field: SpatialOutputFieldV2,
    },
}

const fn dimension_error_kinds() -> [SpatialLayoutDimensionErrorKindV2; 8] {
    use LayoutConstraintFieldV1::{Maximum, Minimum, Preferred};
    use LayoutExtentV1::{Height, Width};
    use SpatialLayoutDimensionErrorKindV2::{InvertedConstraint, NegativeConstraint};

    [
        NegativeConstraint {
            extent: Width,
            field: Minimum,
        },
        NegativeConstraint {
            extent: Width,
            field: Preferred,
        },
        NegativeConstraint {
            extent: Width,
            field: Maximum,
        },
        InvertedConstraint(Width),
        NegativeConstraint {
            extent: Height,
            field: Minimum,
        },
        NegativeConstraint {
            extent: Height,
            field: Preferred,
        },
        NegativeConstraint {
            extent: Height,
            field: Maximum,
        },
        InvertedConstraint(Height),
    ]
}

const fn input_error_kinds() -> [SpatialInputErrorKindV2; 30] {
    use SpatialInputErrorKindV2::{
        EmptyInput, ForwardSpatialParent, FreeOffsetOutOfDomain, InvalidContainer,
        InvalidLayoutDimensions, InvalidPreorder, InvalidRootKey, InvalidRootPlacement,
        MissingSpatialParent, NegativeFreeExtent, NegativeViewport, NonDenseNodeKey, RootHasParent,
        RootPlacementOnNonRoot,
    };

    [
        EmptyInput,
        InvalidRootKey,
        RootHasParent,
        InvalidRootPlacement,
        NonDenseNodeKey,
        MissingSpatialParent,
        ForwardSpatialParent,
        InvalidPreorder,
        RootPlacementOnNonRoot,
        NegativeViewport(SpatialExtentV2::Width),
        NegativeViewport(SpatialExtentV2::Height),
        NegativeFreeExtent(SpatialExtentV2::Width),
        NegativeFreeExtent(SpatialExtentV2::Height),
        FreeOffsetOutOfDomain(SpatialAxisV2::X),
        FreeOffsetOutOfDomain(SpatialAxisV2::Y),
        InvalidContainer(SpatialContainerErrorKindV2::ALL[0]),
        InvalidContainer(SpatialContainerErrorKindV2::ALL[1]),
        InvalidContainer(SpatialContainerErrorKindV2::ALL[2]),
        InvalidContainer(SpatialContainerErrorKindV2::ALL[3]),
        InvalidContainer(SpatialContainerErrorKindV2::ALL[4]),
        InvalidContainer(SpatialContainerErrorKindV2::ALL[5]),
        InvalidContainer(SpatialContainerErrorKindV2::ALL[6]),
        InvalidLayoutDimensions(SpatialLayoutDimensionErrorKindV2::ALL[0]),
        InvalidLayoutDimensions(SpatialLayoutDimensionErrorKindV2::ALL[1]),
        InvalidLayoutDimensions(SpatialLayoutDimensionErrorKindV2::ALL[2]),
        InvalidLayoutDimensions(SpatialLayoutDimensionErrorKindV2::ALL[3]),
        InvalidLayoutDimensions(SpatialLayoutDimensionErrorKindV2::ALL[4]),
        InvalidLayoutDimensions(SpatialLayoutDimensionErrorKindV2::ALL[5]),
        InvalidLayoutDimensions(SpatialLayoutDimensionErrorKindV2::ALL[6]),
        InvalidLayoutDimensions(SpatialLayoutDimensionErrorKindV2::ALL[7]),
    ]
}
