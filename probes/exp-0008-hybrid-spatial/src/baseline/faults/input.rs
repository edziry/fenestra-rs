use std::sync::Arc;

use fenestra_ui_layout::prototype::{LayoutAxisV1, LayoutPaddingV1, ReferenceStackEngineV1};
use fenestra_ui_spatial::prototype::{
    Affine2V2, REGISTERED_SPATIAL_LIMITS_V2, SpatialAnchorComponentV2, SpatialAnchorTargetV2,
    SpatialAnchorV2, SpatialBrushContentV2, SpatialBrushKeyV2, SpatialBrushV2, SpatialClipKeyV2,
    SpatialClipV2, SpatialContainerV2, SpatialCoverageV2, SpatialFillRuleV2,
    SpatialFreePlacementV2, SpatialGradientStopV2, SpatialHitV2, SpatialImageKeyV2, SpatialImageV2,
    SpatialInputPolicyV2, SpatialLocalTransformV2, SpatialNodeKeyV2, SpatialNodeV2,
    SpatialOffsetV2, SpatialOwnedInputV2, SpatialPaintContentV2, SpatialPaintV2, SpatialPathKeyV2,
    SpatialPathV2, SpatialPathVerbV2, SpatialPlacementV2, SpatialPointV2, SpatialRgba8V2,
    SpatialScalarV2, SpatialSemanticGeometryV2, SpatialShapeGeometryV2, SpatialShapeKeyV2,
    SpatialShapeV2, SpatialViewportV2, resolve_spatial_v2,
};

use super::types::RawSpatialFaultV2;

const S: i64 = SpatialScalarV2::SCALE;

#[derive(Default)]
pub(super) struct Parts {
    pub(super) nodes: Vec<SpatialNodeV2>,
    pub(super) points: Vec<SpatialPointV2>,
    pub(super) verbs: Vec<SpatialPathVerbV2>,
    pub(super) paths: Vec<SpatialPathV2>,
    pub(super) shapes: Vec<SpatialShapeV2>,
    pub(super) clips: Vec<SpatialClipV2>,
    pub(super) stops: Vec<SpatialGradientStopV2>,
    pub(super) brushes: Vec<SpatialBrushV2>,
    pub(super) images: Vec<SpatialImageV2>,
    pub(super) paints: Vec<SpatialPaintV2>,
    pub(super) hits: Vec<SpatialHitV2>,
    pub(super) semantics: Vec<SpatialSemanticGeometryV2>,
}

impl Parts {
    fn valid_topology() -> Self {
        Self {
            nodes: vec![
                root(),
                free(1, 0, SpatialAnchorTargetV2::Viewport, identity()),
            ],
            ..Self::default()
        }
    }

    pub(super) fn owned(self, viewport: SpatialViewportV2) -> Arc<SpatialOwnedInputV2> {
        Arc::new(SpatialOwnedInputV2::new(
            viewport,
            self.nodes.into_boxed_slice(),
            self.points.into_boxed_slice(),
            self.verbs.into_boxed_slice(),
            self.paths.into_boxed_slice(),
            self.shapes.into_boxed_slice(),
            self.clips.into_boxed_slice(),
            self.stops.into_boxed_slice(),
            self.brushes.into_boxed_slice(),
            self.images.into_boxed_slice(),
            self.paints.into_boxed_slice(),
            self.hits.into_boxed_slice(),
            self.semantics.into_boxed_slice(),
        ))
    }
}

pub(super) fn content_faults() -> Vec<RawSpatialFaultV2> {
    vec![
        capture("shape-negative-extent", shape_negative_extent()),
        capture("path-first-not-move", path_first_not_move()),
        capture("brush-too-few-stops", brush_too_few_stops()),
        capture("image-stride-mismatch", image_stride_mismatch()),
        capture("clip-forward-parent", clip_forward_parent()),
        capture("paint-missing-brush", paint_missing_brush()),
        capture("hit-missing-shape", hit_missing_shape()),
    ]
}

pub(super) fn dependency_cycle() -> RawSpatialFaultV2 {
    let parts = Parts {
        nodes: vec![
            root(),
            free(1, 0, node_target(2), identity()),
            free(2, 0, node_target(1), identity()),
        ],
        ..Parts::default()
    };
    capture(
        "dependency-cycle",
        reject(parts, SpatialViewportV2::new(8, 8)),
    )
}

pub(super) fn singular_transform() -> RawSpatialFaultV2 {
    let parts = Parts {
        nodes: vec![
            root(),
            free(1, 0, SpatialAnchorTargetV2::Viewport, identity()),
            free(2, 0, SpatialAnchorTargetV2::Viewport, identity()),
            free(3, 0, SpatialAnchorTargetV2::Viewport, singular()),
        ],
        ..Parts::default()
    };
    capture(
        "singular-transform",
        reject(parts, SpatialViewportV2::new(8, 8)),
    )
}

pub(super) fn empty_owned(viewport: SpatialViewportV2) -> Arc<SpatialOwnedInputV2> {
    let mut parts = Parts::default();
    parts.nodes.push(root());
    parts.owned(viewport)
}

fn shape_negative_extent() -> fenestra_ui_spatial::prototype::SpatialResolveErrorV2 {
    let mut parts = Parts::valid_topology();
    parts.shapes.push(SpatialShapeV2::new(
        SpatialShapeKeyV2::new(0),
        SpatialNodeKeyV2::new(1),
        SpatialShapeGeometryV2::Rect {
            origin: point(0, 0),
            width: scalar(-1),
            height: scalar(S),
        },
    ));
    reject(parts, SpatialViewportV2::new(8, 8))
}

fn path_first_not_move() -> fenestra_ui_spatial::prototype::SpatialResolveErrorV2 {
    let mut parts = Parts::valid_topology();
    parts.verbs.push(SpatialPathVerbV2::Close);
    parts
        .paths
        .push(SpatialPathV2::new(SpatialPathKeyV2::new(0), 0, 1));
    reject(parts, SpatialViewportV2::new(8, 8))
}

fn brush_too_few_stops() -> fenestra_ui_spatial::prototype::SpatialResolveErrorV2 {
    let mut parts = Parts::valid_topology();
    parts.stops.push(SpatialGradientStopV2::new(
        0,
        SpatialRgba8V2::new(0, 0, 0, 255),
    ));
    parts.brushes.push(SpatialBrushV2::new(
        SpatialBrushKeyV2::new(0),
        SpatialBrushContentV2::LinearGradient {
            stop_start: 0,
            stop_length: 1,
            start: point(0, 0),
            end: point(S, 0),
        },
    ));
    reject(parts, SpatialViewportV2::new(8, 8))
}

fn image_stride_mismatch() -> fenestra_ui_spatial::prototype::SpatialResolveErrorV2 {
    let mut parts = Parts::valid_topology();
    parts.images.push(SpatialImageV2::new(
        SpatialImageKeyV2::new(0),
        1,
        1,
        3,
        vec![0; 4].into_boxed_slice(),
    ));
    reject(parts, SpatialViewportV2::new(8, 8))
}

fn clip_forward_parent() -> fenestra_ui_spatial::prototype::SpatialResolveErrorV2 {
    let mut parts = Parts::valid_topology();
    parts.shapes.push(valid_shape());
    parts.clips.push(SpatialClipV2::new(
        SpatialClipKeyV2::new(0),
        SpatialNodeKeyV2::new(1),
        Some(SpatialClipKeyV2::new(0)),
        SpatialShapeKeyV2::new(0),
        SpatialFillRuleV2::NonZero,
    ));
    reject(parts, SpatialViewportV2::new(8, 8))
}

fn paint_missing_brush() -> fenestra_ui_spatial::prototype::SpatialResolveErrorV2 {
    let mut parts = Parts::valid_topology();
    parts.shapes.push(valid_shape());
    parts.paints.push(SpatialPaintV2::new(
        SpatialNodeKeyV2::new(1),
        0,
        SpatialPaintContentV2::CoveragePaint {
            coverage: fill(0),
            brush: SpatialBrushKeyV2::new(0),
            opacity: 255,
            clip: None,
        },
    ));
    reject(parts, SpatialViewportV2::new(8, 8))
}

fn hit_missing_shape() -> fenestra_ui_spatial::prototype::SpatialResolveErrorV2 {
    let mut parts = Parts::valid_topology();
    parts.hits.push(SpatialHitV2::new(
        SpatialNodeKeyV2::new(1),
        0,
        fill(0),
        None,
        SpatialInputPolicyV2::Accept,
    ));
    reject(parts, SpatialViewportV2::new(8, 8))
}

fn reject(
    parts: Parts,
    viewport: SpatialViewportV2,
) -> fenestra_ui_spatial::prototype::SpatialResolveErrorV2 {
    resolve_spatial_v2(
        &ReferenceStackEngineV1::new(),
        parts.owned(viewport),
        REGISTERED_SPATIAL_LIMITS_V2,
    )
    .err()
    .expect("registered invalid raw input must be rejected")
}

fn capture(
    label: &'static str,
    error: fenestra_ui_spatial::prototype::SpatialResolveErrorV2,
) -> RawSpatialFaultV2 {
    RawSpatialFaultV2 {
        label,
        kind: error.kind(),
        location: error.location(),
        observed: error.observed(),
        maximum: error.maximum(),
    }
}

pub(super) fn valid_shape() -> SpatialShapeV2 {
    SpatialShapeV2::new(
        SpatialShapeKeyV2::new(0),
        SpatialNodeKeyV2::new(1),
        SpatialShapeGeometryV2::Rect {
            origin: point(0, 0),
            width: scalar(S),
            height: scalar(S),
        },
    )
}

pub(super) fn fill(shape: u32) -> SpatialCoverageV2 {
    SpatialCoverageV2::Fill {
        shape: SpatialShapeKeyV2::new(shape),
        rule: SpatialFillRuleV2::NonZero,
    }
}

pub(super) fn root() -> SpatialNodeV2 {
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(0),
        None,
        SpatialPlacementV2::Root,
        container(),
    )
}

pub(super) fn free(
    key: u32,
    parent: u32,
    target: SpatialAnchorTargetV2,
    transform: SpatialLocalTransformV2,
) -> SpatialNodeV2 {
    let anchor = SpatialAnchorV2::new(
        SpatialAnchorComponentV2::Start,
        SpatialAnchorComponentV2::Start,
    );
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(key),
        Some(SpatialNodeKeyV2::new(parent)),
        SpatialPlacementV2::Free(SpatialFreePlacementV2::new(
            4,
            4,
            anchor,
            target,
            anchor,
            SpatialOffsetV2::new(scalar(0), scalar(0)),
            transform,
        )),
        container(),
    )
}

fn node_target(key: u32) -> SpatialAnchorTargetV2 {
    SpatialAnchorTargetV2::Node(SpatialNodeKeyV2::new(key))
}

pub(super) fn container() -> SpatialContainerV2 {
    SpatialContainerV2::new(LayoutAxisV1::Column, LayoutPaddingV1::new(0, 0, 0, 0), 0)
}

pub(super) fn identity() -> SpatialLocalTransformV2 {
    SpatialLocalTransformV2::new(Affine2V2::identity(), point(0, 0))
}

fn singular() -> SpatialLocalTransformV2 {
    SpatialLocalTransformV2::new(
        Affine2V2::new(
            scalar(0),
            scalar(0),
            scalar(0),
            scalar(0),
            scalar(0),
            scalar(0),
        ),
        point(0, 0),
    )
}

pub(super) fn point(x: i64, y: i64) -> SpatialPointV2 {
    SpatialPointV2::new(scalar(x), scalar(y))
}

pub(super) fn scalar(raw: i64) -> SpatialScalarV2 {
    SpatialScalarV2::new(raw)
}
