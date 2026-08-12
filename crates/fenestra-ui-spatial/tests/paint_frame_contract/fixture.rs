use std::sync::Arc;

use fenestra_ui_layout::prototype::{
    LayoutAxisV1, LayoutDimensionV1, LayoutPaddingV1, ReferenceStackEngineV1,
};
use fenestra_ui_spatial::prototype::{
    Affine2V2, REGISTERED_SPATIAL_LIMITS_V2, SpatialBrushContentV2, SpatialBrushKeyV2,
    SpatialBrushV2, SpatialClipKeyV2, SpatialClipV2, SpatialContainerV2, SpatialCoverageV2,
    SpatialFillRuleV2, SpatialGradientStopV2, SpatialImageKeyV2, SpatialImageV2,
    SpatialLayoutPlacementV2, SpatialLocalTransformV2, SpatialNodeKeyV2, SpatialNodeV2,
    SpatialOwnedInputV2, SpatialPaintContentV2, SpatialPaintV2, SpatialPathKeyV2, SpatialPathV2,
    SpatialPathVerbV2, SpatialPlacementV2, SpatialPointV2, SpatialRgba8V2, SpatialScalarV2,
    SpatialShapeGeometryV2, SpatialShapeKeyV2, SpatialShapeV2, SpatialViewportV2,
    resolve_spatial_v2,
};

use crate::SpatialResolvedSnapshotV2;

pub(super) struct ExpectedTables {
    pub polygon_points: SliceIdentity<SpatialPointV2>,
    pub path_verbs: SliceIdentity<SpatialPathVerbV2>,
    pub paths: SliceIdentity<SpatialPathV2>,
    pub shapes: SliceIdentity<SpatialShapeV2>,
    pub clips: SliceIdentity<SpatialClipV2>,
    pub gradient_stops: SliceIdentity<SpatialGradientStopV2>,
    pub brushes: SliceIdentity<SpatialBrushV2>,
    pub images: SliceIdentity<SpatialImageV2>,
    pub image_bytes: SliceIdentity<u8>,
    pub paints: SliceIdentity<SpatialPaintV2>,
}

pub(super) struct SliceIdentity<T> {
    pub(super) pointer: *const T,
    pub(super) length: usize,
}

impl<T> SliceIdentity<T> {
    pub(super) fn assert(&self, actual: &[T]) {
        assert_eq!(actual.as_ptr(), self.pointer);
        assert_eq!(actual.len(), self.length);
    }
}

pub(super) fn resolved() -> (SpatialResolvedSnapshotV2, ExpectedTables) {
    let viewport = SpatialViewportV2::new(2, 1);
    let container =
        SpatialContainerV2::new(LayoutAxisV1::Column, LayoutPaddingV1::new(0, 0, 0, 0), 0);
    let nodes = vec![
        SpatialNodeV2::new(
            SpatialNodeKeyV2::new(0),
            None,
            SpatialPlacementV2::Root,
            container,
        ),
        SpatialNodeV2::new(
            SpatialNodeKeyV2::new(1),
            Some(SpatialNodeKeyV2::new(0)),
            SpatialPlacementV2::Layout(SpatialLayoutPlacementV2::new(
                LayoutDimensionV1::new(2, 2, 2),
                LayoutDimensionV1::new(1, 1, 1),
                SpatialLocalTransformV2::new(Affine2V2::identity(), point(0, 0)),
            )),
            container,
        ),
    ]
    .into_boxed_slice();
    let polygon_points = vec![point(0, 0), point(1, 0), point(0, 1)].into_boxed_slice();
    let path_verbs = vec![
        SpatialPathVerbV2::MoveTo { to: point(0, 0) },
        SpatialPathVerbV2::LineTo { to: point(1, 0) },
        SpatialPathVerbV2::Close,
    ]
    .into_boxed_slice();
    let paths = vec![SpatialPathV2::new(SpatialPathKeyV2::new(0), 0, 3)].into_boxed_slice();
    let shapes = vec![
        SpatialShapeV2::new(
            SpatialShapeKeyV2::new(0),
            SpatialNodeKeyV2::new(1),
            SpatialShapeGeometryV2::Rect {
                origin: point(0, 0),
                width: scalar(2),
                height: scalar(1),
            },
        ),
        SpatialShapeV2::new(
            SpatialShapeKeyV2::new(1),
            SpatialNodeKeyV2::new(1),
            SpatialShapeGeometryV2::Polygon {
                point_start: 0,
                point_length: 3,
            },
        ),
    ]
    .into_boxed_slice();
    let clips = vec![SpatialClipV2::new(
        SpatialClipKeyV2::new(0),
        SpatialNodeKeyV2::new(1),
        None,
        SpatialShapeKeyV2::new(0),
        SpatialFillRuleV2::NonZero,
    )]
    .into_boxed_slice();
    let gradient_stops = Vec::new().into_boxed_slice();
    let brushes = vec![SpatialBrushV2::new(
        SpatialBrushKeyV2::new(0),
        SpatialBrushContentV2::Solid {
            color: SpatialRgba8V2::new(128, 64, 0, 255),
        },
    )]
    .into_boxed_slice();
    let images = vec![SpatialImageV2::new(
        SpatialImageKeyV2::new(0),
        1,
        1,
        4,
        vec![1, 2, 3, 255].into_boxed_slice(),
    )]
    .into_boxed_slice();
    let paints = vec![SpatialPaintV2::new(
        SpatialNodeKeyV2::new(1),
        0,
        SpatialPaintContentV2::CoveragePaint {
            coverage: SpatialCoverageV2::Fill {
                shape: SpatialShapeKeyV2::new(0),
                rule: SpatialFillRuleV2::NonZero,
            },
            brush: SpatialBrushKeyV2::new(0),
            opacity: 255,
            clip: Some(SpatialClipKeyV2::new(0)),
        },
    )]
    .into_boxed_slice();

    let expected = ExpectedTables {
        polygon_points: identity(&polygon_points),
        path_verbs: identity(&path_verbs),
        paths: identity(&paths),
        shapes: identity(&shapes),
        clips: identity(&clips),
        gradient_stops: identity(&gradient_stops),
        brushes: identity(&brushes),
        images: identity(&images),
        image_bytes: identity(images[0].bytes()),
        paints: identity(&paints),
    };
    let source = Arc::new(SpatialOwnedInputV2::new(
        viewport,
        nodes,
        polygon_points,
        path_verbs,
        paths,
        shapes,
        clips,
        gradient_stops,
        brushes,
        images,
        paints,
        Vec::new().into_boxed_slice(),
        Vec::new().into_boxed_slice(),
    ));
    let snapshot = resolve_spatial_v2(
        &ReferenceStackEngineV1::new(),
        source,
        REGISTERED_SPATIAL_LIMITS_V2,
    )
    .expect("paint frame fixture resolves");
    (snapshot, expected)
}

fn identity<T>(slice: &[T]) -> SliceIdentity<T> {
    SliceIdentity {
        pointer: slice.as_ptr(),
        length: slice.len(),
    }
}

fn point(x: i64, y: i64) -> SpatialPointV2 {
    SpatialPointV2::new(scalar(x), scalar(y))
}

fn scalar(units: i64) -> SpatialScalarV2 {
    SpatialScalarV2::new(units * 65_536)
}
