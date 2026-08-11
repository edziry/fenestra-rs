use super::super::*;

#[test]
fn finite_topology_vocabularies_are_exact() {
    assert_eq!(
        SpatialAnchorComponentV2::ALL,
        [
            SpatialAnchorComponentV2::Start,
            SpatialAnchorComponentV2::Center,
            SpatialAnchorComponentV2::End,
        ]
    );
    assert_eq!(
        SpatialAnchorTargetKindV2::ALL,
        [
            SpatialAnchorTargetKindV2::Viewport,
            SpatialAnchorTargetKindV2::Parent,
            SpatialAnchorTargetKindV2::Node,
        ]
    );
    assert_eq!(
        SpatialPlacementKindV2::ALL,
        [
            SpatialPlacementKindV2::Root,
            SpatialPlacementKindV2::Layout,
            SpatialPlacementKindV2::Free,
        ]
    );
    assert_eq!(SpatialAxisV2::ALL, [SpatialAxisV2::X, SpatialAxisV2::Y]);
    assert_eq!(
        SpatialExtentV2::ALL,
        [SpatialExtentV2::Width, SpatialExtentV2::Height]
    );
}

#[test]
fn node_field_vocabulary_is_exact() {
    use SpatialNodeFieldV2::{
        AffineA, AffineB, AffineC, AffineD, AffineTx, AffineTy, ContainerAxis, FreeHeight,
        FreeOffsetX, FreeOffsetY, FreeWidth, Gap, Key, LayoutHeightMaximum, LayoutHeightMinimum,
        LayoutHeightPreferred, LayoutWidthMaximum, LayoutWidthMinimum, LayoutWidthPreferred,
        PaddingBottom, PaddingLeft, PaddingRight, PaddingTop, Parent, Placement,
        SelfAnchorHorizontal, SelfAnchorVertical, TargetAnchorHorizontal, TargetAnchorVertical,
        TargetKey, TargetKind, TransformOriginX, TransformOriginY,
    };

    assert_eq!(
        SpatialNodeFieldV2::ALL,
        [
            Key,
            Parent,
            Placement,
            FreeWidth,
            FreeHeight,
            FreeOffsetX,
            FreeOffsetY,
            LayoutWidthMinimum,
            LayoutWidthPreferred,
            LayoutWidthMaximum,
            LayoutHeightMinimum,
            LayoutHeightPreferred,
            LayoutHeightMaximum,
            ContainerAxis,
            PaddingLeft,
            PaddingRight,
            PaddingTop,
            PaddingBottom,
            Gap,
            AffineA,
            AffineB,
            AffineC,
            AffineD,
            AffineTx,
            AffineTy,
            TransformOriginX,
            TransformOriginY,
            SelfAnchorHorizontal,
            SelfAnchorVertical,
            TargetKind,
            TargetKey,
            TargetAnchorHorizontal,
            TargetAnchorVertical,
        ]
    );
}

#[test]
fn input_failure_vocabularies_are_exact() {
    use LayoutConstraintFieldV1::{Maximum, Minimum, Preferred};
    use LayoutExtentV1::{Height, Width};
    use LayoutPaddingSideV1::{Bottom, Left, Right, Top};
    use SpatialAxisV2::{X, Y};
    use SpatialContainerErrorKindV2::{NegativeGap, NegativePadding, PaddingExceedsExtent};
    use SpatialInputErrorKindV2::{
        EmptyInput, ForwardSpatialParent, FreeOffsetOutOfDomain, InvalidContainer,
        InvalidLayoutDimensions, InvalidPreorder, InvalidRootKey, InvalidRootPlacement,
        MissingSpatialParent, NegativeFreeExtent, NegativeViewport, NonDenseNodeKey, RootHasParent,
        RootPlacementOnNonRoot,
    };
    use SpatialLayoutDimensionErrorKindV2::{InvertedConstraint, NegativeConstraint};

    let container = [
        NegativePadding(Left),
        NegativePadding(Right),
        NegativePadding(Top),
        NegativePadding(Bottom),
        PaddingExceedsExtent(Width),
        PaddingExceedsExtent(Height),
        NegativeGap,
    ];
    assert_eq!(SpatialContainerErrorKindV2::ALL, container);

    let dimensions = [
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
    ];
    assert_eq!(SpatialLayoutDimensionErrorKindV2::ALL, dimensions);

    let expected = [
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
        FreeOffsetOutOfDomain(X),
        FreeOffsetOutOfDomain(Y),
        InvalidContainer(container[0]),
        InvalidContainer(container[1]),
        InvalidContainer(container[2]),
        InvalidContainer(container[3]),
        InvalidContainer(container[4]),
        InvalidContainer(container[5]),
        InvalidContainer(container[6]),
        InvalidLayoutDimensions(dimensions[0]),
        InvalidLayoutDimensions(dimensions[1]),
        InvalidLayoutDimensions(dimensions[2]),
        InvalidLayoutDimensions(dimensions[3]),
        InvalidLayoutDimensions(dimensions[4]),
        InvalidLayoutDimensions(dimensions[5]),
        InvalidLayoutDimensions(dimensions[6]),
        InvalidLayoutDimensions(dimensions[7]),
    ];
    assert_eq!(SpatialInputErrorKindV2::ALL, expected);

    assert_eq!(
        SpatialDependencyErrorKindV2::ALL,
        [
            SpatialDependencyErrorKindV2::MissingTarget,
            SpatialDependencyErrorKindV2::SentinelNodeTarget,
            SpatialDependencyErrorKindV2::SelfTarget,
            SpatialDependencyErrorKindV2::Cycle,
        ]
    );
}

#[test]
fn initial_locations_are_typed_and_payload_free() {
    let locations = [
        SpatialErrorLocationV2::Input,
        SpatialErrorLocationV2::Viewport {
            extent: SpatialExtentV2::Width,
        },
        SpatialErrorLocationV2::Node { index: 3 },
        SpatialErrorLocationV2::NodeField {
            index: 4,
            field: SpatialNodeFieldV2::TargetKey,
        },
        SpatialErrorLocationV2::Island { index: 5 },
        SpatialErrorLocationV2::Dependency { ordinal: 6 },
    ];

    assert_eq!(locations.len(), 6);
    assert_eq!(locations.map(location_ordinal), [0, 1, 2, 3, 4, 5]);
    assert_eq!(
        locations[3],
        SpatialErrorLocationV2::NodeField {
            index: 4,
            field: SpatialNodeFieldV2::TargetKey,
        }
    );
}

fn location_ordinal(location: SpatialErrorLocationV2) -> u8 {
    match location {
        SpatialErrorLocationV2::Input => 0,
        SpatialErrorLocationV2::Viewport { .. } => 1,
        SpatialErrorLocationV2::Node { .. } => 2,
        SpatialErrorLocationV2::NodeField { .. } => 3,
        SpatialErrorLocationV2::Island { .. } => 4,
        SpatialErrorLocationV2::Dependency { .. } => 5,
    }
}
