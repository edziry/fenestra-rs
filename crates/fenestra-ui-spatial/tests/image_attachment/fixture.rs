use std::sync::Arc;

use fenestra_ui_layout::prototype::{
    LayoutAxisV1, LayoutDimensionV1, LayoutPaddingV1, ReferenceStackEngineV1,
};
use fenestra_ui_spatial::prototype::{
    Affine2V2, REGISTERED_SPATIAL_LIMITS_V2, SpatialBrushContentV2, SpatialBrushKeyV2,
    SpatialBrushV2, SpatialClipKeyV2, SpatialClipV2, SpatialContainerV2, SpatialCoverageV2,
    SpatialFillRuleV2, SpatialHitV2, SpatialInputPolicyV2, SpatialLayoutPlacementV2,
    SpatialLocalTransformV2, SpatialNodeKeyV2, SpatialNodeV2, SpatialOwnedInputV2,
    SpatialPaintContentV2, SpatialPaintV2, SpatialPlacementV2, SpatialPointV2,
    SpatialResolvedSnapshotV2, SpatialRgba8V2, SpatialSemanticGeometryV2, SpatialShapeGeometryV2,
    SpatialShapeKeyV2, SpatialShapeV2, SpatialViewportV2, resolve_spatial_v2,
};

use super::scalar;

pub(super) fn resolved() -> SpatialResolvedSnapshotV2 {
    resolve_spatial_v2(
        &ReferenceStackEngineV1::new(),
        source(),
        REGISTERED_SPATIAL_LIMITS_V2,
    )
    .unwrap()
}

pub(super) fn source() -> Arc<SpatialOwnedInputV2> {
    let nodes = vec![
        SpatialNodeV2::new(
            SpatialNodeKeyV2::new(0),
            None,
            SpatialPlacementV2::Root,
            container(),
        ),
        node(1, 0, 4, 2, 0, 0),
        node(2, 1, 2, 1, 0, 0),
        node(3, 1, 2, 1, 1, -1),
    ];
    let shapes = vec![
        shape(0, 1, 4, 2),
        shape(1, 2, 2, 1),
        shape(2, 3, 2, 1),
        shape(3, 1, 1, 2),
    ];
    let brushes = [[200, 0, 0, 255], [0, 0, 200, 255], [0, 128, 0, 128]]
        .into_iter()
        .enumerate()
        .map(|(key, rgba)| {
            SpatialBrushV2::new(
                SpatialBrushKeyV2::new(key as u32),
                SpatialBrushContentV2::Solid {
                    color: SpatialRgba8V2::new(rgba[0], rgba[1], rgba[2], rgba[3]),
                },
            )
        })
        .collect();
    let paints = (0..3)
        .map(|key| {
            SpatialPaintV2::new(
                SpatialNodeKeyV2::new(key + 1),
                0,
                SpatialPaintContentV2::CoveragePaint {
                    coverage: coverage(key),
                    brush: SpatialBrushKeyV2::new(key),
                    opacity: 255,
                    clip: None,
                },
            )
        })
        .collect();
    Arc::new(SpatialOwnedInputV2::new(
        SpatialViewportV2::new(4, 2),
        nodes.into_boxed_slice(),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        shapes.into_boxed_slice(),
        Box::new([SpatialClipV2::new(
            SpatialClipKeyV2::new(0),
            SpatialNodeKeyV2::new(1),
            None,
            SpatialShapeKeyV2::new(3),
            SpatialFillRuleV2::NonZero,
        )]),
        Box::new([]),
        brushes,
        Box::new([]),
        paints,
        Box::new([SpatialHitV2::new(
            SpatialNodeKeyV2::new(1),
            0,
            coverage(0),
            None,
            SpatialInputPolicyV2::Accept,
        )]),
        Box::new([SpatialSemanticGeometryV2::new(
            SpatialNodeKeyV2::new(1),
            0,
            SpatialShapeKeyV2::new(0),
            SpatialFillRuleV2::NonZero,
            None,
        )]),
    ))
}

fn container() -> SpatialContainerV2 {
    SpatialContainerV2::new(LayoutAxisV1::Column, LayoutPaddingV1::new(0, 0, 0, 0), 0)
}

fn node(key: u32, parent: u32, width: i32, height: i32, x: i32, y: i32) -> SpatialNodeV2 {
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(key),
        Some(SpatialNodeKeyV2::new(parent)),
        SpatialPlacementV2::Layout(SpatialLayoutPlacementV2::new(
            LayoutDimensionV1::new(width, width, width),
            LayoutDimensionV1::new(height, height, height),
            SpatialLocalTransformV2::new(Affine2V2::translation(scalar(x), scalar(y)), point(0, 0)),
        )),
        container(),
    )
}

fn shape(key: u32, owner: u32, width: i32, height: i32) -> SpatialShapeV2 {
    SpatialShapeV2::new(
        SpatialShapeKeyV2::new(key),
        SpatialNodeKeyV2::new(owner),
        SpatialShapeGeometryV2::Rect {
            origin: point(0, 0),
            width: scalar(width),
            height: scalar(height),
        },
    )
}

fn coverage(shape: u32) -> SpatialCoverageV2 {
    SpatialCoverageV2::Fill {
        shape: SpatialShapeKeyV2::new(shape),
        rule: SpatialFillRuleV2::NonZero,
    }
}

fn point(x: i32, y: i32) -> SpatialPointV2 {
    SpatialPointV2::new(scalar(x), scalar(y))
}
