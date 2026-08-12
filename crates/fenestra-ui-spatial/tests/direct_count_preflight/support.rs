use fenestra_ui_layout::prototype::{LayoutAxisV1, LayoutPaddingV1};
use fenestra_ui_spatial::prototype::{
    SpatialBrushContentV2, SpatialBrushKeyV2, SpatialBrushV2, SpatialClipKeyV2, SpatialClipV2,
    SpatialContainerV2, SpatialCoverageV2, SpatialFillRuleV2, SpatialGradientStopV2, SpatialHitV2,
    SpatialImageKeyV2, SpatialImageV2, SpatialInputPolicyV2, SpatialLimitKindV2, SpatialLimitsV2,
    SpatialNodeKeyV2, SpatialNodeV2, SpatialOwnedInputV2, SpatialPaintContentV2, SpatialPaintV2,
    SpatialPathKeyV2, SpatialPathV2, SpatialPathVerbV2, SpatialPlacementV2, SpatialPointV2,
    SpatialRgba8V2, SpatialScalarV2, SpatialSemanticGeometryV2, SpatialShapeGeometryV2,
    SpatialShapeKeyV2, SpatialShapeV2, SpatialViewportV2,
};

pub(super) const DIRECT_COUNT: usize = 12;

pub(super) fn limits_with_direct(maxima: [usize; DIRECT_COUNT]) -> SpatialLimitsV2 {
    assert_eq!(SpatialLimitKindV2::DIRECT_ALL.len(), DIRECT_COUNT);
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    values[..DIRECT_COUNT].copy_from_slice(&maxima);
    SpatialLimitsV2::new(values)
}

pub(super) fn owned_input(counts: [usize; DIRECT_COUNT]) -> SpatialOwnedInputV2 {
    SpatialOwnedInputV2::new(
        SpatialViewportV2::new(0, 0),
        vec![node(); counts[0]].into_boxed_slice(),
        vec![point(); counts[9]].into_boxed_slice(),
        vec![SpatialPathVerbV2::Close; counts[8]].into_boxed_slice(),
        vec![path(); counts[7]].into_boxed_slice(),
        vec![shape(); counts[1]].into_boxed_slice(),
        vec![clip(); counts[3]].into_boxed_slice(),
        vec![gradient_stop(); counts[10]].into_boxed_slice(),
        vec![brush(); counts[2]].into_boxed_slice(),
        (0..counts[11])
            .map(|_| image())
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        vec![paint(); counts[4]].into_boxed_slice(),
        vec![hit(); counts[5]].into_boxed_slice(),
        vec![semantic(); counts[6]].into_boxed_slice(),
    )
}

fn point() -> SpatialPointV2 {
    SpatialPointV2::new(SpatialScalarV2::new(0), SpatialScalarV2::new(0))
}

fn node() -> SpatialNodeV2 {
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(0),
        None,
        SpatialPlacementV2::Root,
        SpatialContainerV2::new(LayoutAxisV1::Column, LayoutPaddingV1::new(0, 0, 0, 0), 0),
    )
}

fn shape() -> SpatialShapeV2 {
    SpatialShapeV2::new(
        SpatialShapeKeyV2::new(0),
        SpatialNodeKeyV2::new(0),
        SpatialShapeGeometryV2::Rect {
            origin: point(),
            width: SpatialScalarV2::new(0),
            height: SpatialScalarV2::new(0),
        },
    )
}

fn brush() -> SpatialBrushV2 {
    SpatialBrushV2::new(
        SpatialBrushKeyV2::new(0),
        SpatialBrushContentV2::Solid {
            color: SpatialRgba8V2::new(0, 0, 0, 0),
        },
    )
}

fn clip() -> SpatialClipV2 {
    SpatialClipV2::new(
        SpatialClipKeyV2::new(0),
        SpatialNodeKeyV2::new(0),
        None,
        SpatialShapeKeyV2::new(0),
        SpatialFillRuleV2::NonZero,
    )
}

fn coverage() -> SpatialCoverageV2 {
    SpatialCoverageV2::Fill {
        shape: SpatialShapeKeyV2::new(0),
        rule: SpatialFillRuleV2::NonZero,
    }
}

fn paint() -> SpatialPaintV2 {
    SpatialPaintV2::new(
        SpatialNodeKeyV2::new(0),
        0,
        SpatialPaintContentV2::CoveragePaint {
            coverage: coverage(),
            brush: SpatialBrushKeyV2::new(0),
            opacity: 0,
            clip: None,
        },
    )
}

fn hit() -> SpatialHitV2 {
    SpatialHitV2::new(
        SpatialNodeKeyV2::new(0),
        0,
        coverage(),
        None,
        SpatialInputPolicyV2::Ignore,
    )
}

fn semantic() -> SpatialSemanticGeometryV2 {
    SpatialSemanticGeometryV2::new(
        SpatialNodeKeyV2::new(0),
        0,
        SpatialShapeKeyV2::new(0),
        SpatialFillRuleV2::NonZero,
        None,
    )
}

fn path() -> SpatialPathV2 {
    SpatialPathV2::new(SpatialPathKeyV2::new(0), 0, 0)
}

fn gradient_stop() -> SpatialGradientStopV2 {
    SpatialGradientStopV2::new(0, SpatialRgba8V2::new(0, 0, 0, 0))
}

fn image() -> SpatialImageV2 {
    SpatialImageV2::new(
        SpatialImageKeyV2::new(0),
        0,
        0,
        0,
        Vec::new().into_boxed_slice(),
    )
}
