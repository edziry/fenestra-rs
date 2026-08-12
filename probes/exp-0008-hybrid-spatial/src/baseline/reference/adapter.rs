use std::sync::Arc;

use fenestra_ui_layout::prototype::{LayoutAxisV1, LayoutDimensionV1, LayoutPaddingV1};
use fenestra_ui_spatial::prototype::{
    Affine2V2, SpatialAnchorComponentV2, SpatialAnchorTargetV2, SpatialAnchorV2,
    SpatialBrushContentV2, SpatialBrushKeyV2, SpatialBrushV2, SpatialClipKeyV2, SpatialClipV2,
    SpatialContainerV2, SpatialCoverageV2, SpatialFillRuleV2, SpatialFreePlacementV2,
    SpatialGradientStopV2, SpatialHitV2, SpatialImageDestinationRectV2, SpatialImageKeyV2,
    SpatialImageSourceRectV2, SpatialImageV2, SpatialInputPolicyV2, SpatialLayoutPlacementV2,
    SpatialLocalTransformV2, SpatialNodeKeyV2, SpatialNodeV2, SpatialOffsetV2, SpatialOwnedInputV2,
    SpatialPaintContentV2, SpatialPaintV2, SpatialPathKeyV2, SpatialPathV2, SpatialPathVerbV2,
    SpatialPlacementV2, SpatialPointV2, SpatialRgba8V2, SpatialScalarV2, SpatialSemanticGeometryV2,
    SpatialShapeGeometryV2, SpatialShapeKeyV2, SpatialShapeV2, SpatialViewportV2,
};

use crate::baseline::literal_types::*;

pub(super) fn owned_input(scene: &SceneInputV2) -> Arc<SpatialOwnedInputV2> {
    let (path_verbs, paths) = paths(scene);
    let (polygon_points, shapes) = shapes(scene);
    let (gradient_stops, brushes) = brushes(scene);
    Arc::new(SpatialOwnedInputV2::new(
        viewport(scene.viewport),
        scene
            .nodes
            .iter()
            .map(node)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        polygon_points.into_boxed_slice(),
        path_verbs.into_boxed_slice(),
        paths.into_boxed_slice(),
        shapes.into_boxed_slice(),
        scene
            .clips
            .iter()
            .map(clip)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        gradient_stops.into_boxed_slice(),
        brushes.into_boxed_slice(),
        scene
            .images
            .iter()
            .map(image)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        scene
            .paints
            .iter()
            .map(paint)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        scene
            .hits
            .iter()
            .map(hit)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        scene
            .semantics
            .iter()
            .map(semantic)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    ))
}

pub(super) fn viewport(value: (u32, u32)) -> SpatialViewportV2 {
    SpatialViewportV2::new(
        i32::try_from(value.0).expect("registered viewport width fits i32"),
        i32::try_from(value.1).expect("registered viewport height fits i32"),
    )
}

pub(super) fn point(value: PointV2) -> SpatialPointV2 {
    SpatialPointV2::new(scalar(value.x), scalar(value.y))
}

fn node(value: &NodeInputV2) -> SpatialNodeV2 {
    let placement = match value.placement {
        PlacementInputV2::Root => SpatialPlacementV2::Root,
        PlacementInputV2::Layout {
            width,
            height,
            transform,
        } => SpatialPlacementV2::Layout(SpatialLayoutPlacementV2::new(
            fixed(width),
            fixed(height),
            local_transform(transform),
        )),
        PlacementInputV2::Free {
            width,
            height,
            self_anchor,
            target,
            target_anchor,
            offset,
            transform,
        } => SpatialPlacementV2::Free(SpatialFreePlacementV2::new(
            width,
            height,
            anchor(self_anchor),
            anchor_target(target),
            anchor(target_anchor),
            SpatialOffsetV2::new(scalar(offset.x), scalar(offset.y)),
            local_transform(transform),
        )),
    };
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(value.key),
        value.parent.map(SpatialNodeKeyV2::new),
        placement,
        SpatialContainerV2::new(
            match value.axis {
                AxisV2::Horizontal => LayoutAxisV1::Row,
                AxisV2::Vertical => LayoutAxisV1::Column,
            },
            LayoutPaddingV1::new(
                value.padding[0],
                value.padding[1],
                value.padding[2],
                value.padding[3],
            ),
            value.gap,
        ),
    )
}

fn paths(scene: &SceneInputV2) -> (Vec<SpatialPathVerbV2>, Vec<SpatialPathV2>) {
    let mut verbs = Vec::new();
    let mut paths = Vec::new();
    for value in &scene.paths {
        let start = u32::try_from(verbs.len()).expect("registered path start fits u32");
        verbs.extend(value.verbs.iter().copied().map(path_verb));
        paths.push(SpatialPathV2::new(
            SpatialPathKeyV2::new(value.key),
            start,
            u32::try_from(value.verbs.len()).expect("registered path length fits u32"),
        ));
    }
    (verbs, paths)
}

fn path_verb(value: PathVerbInputV2) -> SpatialPathVerbV2 {
    match value {
        PathVerbInputV2::Move(to) => SpatialPathVerbV2::MoveTo { to: point(to) },
        PathVerbInputV2::Line(to) => SpatialPathVerbV2::LineTo { to: point(to) },
        PathVerbInputV2::Quadratic(control, to) => SpatialPathVerbV2::QuadraticTo {
            control: point(control),
            to: point(to),
        },
        PathVerbInputV2::Cubic(control1, control2, to) => SpatialPathVerbV2::CubicTo {
            control1: point(control1),
            control2: point(control2),
            to: point(to),
        },
        PathVerbInputV2::Close => SpatialPathVerbV2::Close,
    }
}

fn shapes(scene: &SceneInputV2) -> (Vec<SpatialPointV2>, Vec<SpatialShapeV2>) {
    let mut points = Vec::new();
    let shapes = scene
        .shapes
        .iter()
        .map(|value| {
            let geometry = match &value.geometry {
                ShapeGeometryInputV2::Rect(value) => SpatialShapeGeometryV2::Rect {
                    origin: point(PointV2 {
                        x: value.x,
                        y: value.y,
                    }),
                    width: scalar(value.width),
                    height: scalar(value.height),
                },
                ShapeGeometryInputV2::Circle { center, radius } => SpatialShapeGeometryV2::Circle {
                    center: point(*center),
                    radius: scalar(*radius),
                },
                ShapeGeometryInputV2::Polygon { points: source } => {
                    let start = u32::try_from(points.len()).expect("polygon start fits u32");
                    points.extend(source.iter().copied().map(point));
                    SpatialShapeGeometryV2::Polygon {
                        point_start: start,
                        point_length: u32::try_from(source.len()).expect("polygon length fits u32"),
                    }
                }
                ShapeGeometryInputV2::Path { path } => SpatialShapeGeometryV2::Path {
                    path: SpatialPathKeyV2::new(*path),
                },
            };
            SpatialShapeV2::new(
                SpatialShapeKeyV2::new(value.key),
                SpatialNodeKeyV2::new(value.owner),
                geometry,
            )
        })
        .collect();
    (points, shapes)
}

fn brushes(scene: &SceneInputV2) -> (Vec<SpatialGradientStopV2>, Vec<SpatialBrushV2>) {
    let mut stops = Vec::new();
    let brushes = scene
        .brushes
        .iter()
        .map(|value| {
            let (key, content) = match value {
                BrushInputV2::Solid { key, color } => (
                    *key,
                    SpatialBrushContentV2::Solid {
                        color: rgba(*color),
                    },
                ),
                BrushInputV2::Linear {
                    key,
                    stops: source,
                    start,
                    end,
                } => {
                    let stop_start = u32::try_from(stops.len()).expect("stop start fits u32");
                    stops.extend(
                        source
                            .iter()
                            .map(|stop| SpatialGradientStopV2::new(stop.offset, rgba(stop.color))),
                    );
                    (
                        *key,
                        SpatialBrushContentV2::LinearGradient {
                            stop_start,
                            stop_length: u32::try_from(source.len()).expect("stop length fits u32"),
                            start: point(*start),
                            end: point(*end),
                        },
                    )
                }
            };
            SpatialBrushV2::new(SpatialBrushKeyV2::new(key), content)
        })
        .collect();
    (stops, brushes)
}

fn clip(value: &ClipInputV2) -> SpatialClipV2 {
    SpatialClipV2::new(
        SpatialClipKeyV2::new(value.key),
        SpatialNodeKeyV2::new(value.owner),
        value.parent.map(SpatialClipKeyV2::new),
        SpatialShapeKeyV2::new(value.shape),
        fill_rule(value.rule),
    )
}

fn image(value: &ImageInputV2) -> SpatialImageV2 {
    SpatialImageV2::new(
        SpatialImageKeyV2::new(value.key),
        value.width,
        value.height,
        value.stride,
        value.bytes.clone().into_boxed_slice(),
    )
}

fn paint(value: &PaintInputV2) -> SpatialPaintV2 {
    let content = match &value.content {
        PaintContentInputV2::Coverage {
            coverage,
            brush,
            opacity,
            clip,
        } => SpatialPaintContentV2::CoveragePaint {
            coverage: coverage_value(*coverage),
            brush: SpatialBrushKeyV2::new(*brush),
            opacity: *opacity,
            clip: clip.map(SpatialClipKeyV2::new),
        },
        PaintContentInputV2::Image {
            image,
            source,
            destination,
            opacity,
            clip,
        } => SpatialPaintContentV2::ImagePaint {
            image: SpatialImageKeyV2::new(*image),
            source: SpatialImageSourceRectV2::new(
                image_scalar(source.x),
                image_scalar(source.y),
                image_scalar(source.width),
                image_scalar(source.height),
            ),
            destination: SpatialImageDestinationRectV2::new(
                scalar(destination.x),
                scalar(destination.y),
                scalar(destination.width),
                scalar(destination.height),
            ),
            opacity: *opacity,
            clip: clip.map(SpatialClipKeyV2::new),
        },
    };
    SpatialPaintV2::new(SpatialNodeKeyV2::new(value.owner), value.item, content)
}

fn hit(value: &HitInputV2) -> SpatialHitV2 {
    SpatialHitV2::new(
        SpatialNodeKeyV2::new(value.owner),
        value.item,
        coverage_value(value.coverage),
        value.clip.map(SpatialClipKeyV2::new),
        if value.accepts {
            SpatialInputPolicyV2::Accept
        } else {
            SpatialInputPolicyV2::Ignore
        },
    )
}

fn semantic(value: &SemanticInputV2) -> SpatialSemanticGeometryV2 {
    SpatialSemanticGeometryV2::new(
        SpatialNodeKeyV2::new(value.owner),
        value.item,
        SpatialShapeKeyV2::new(value.shape),
        fill_rule(value.rule),
        value.clip.map(SpatialClipKeyV2::new),
    )
}

fn coverage_value(value: CoverageInputV2) -> SpatialCoverageV2 {
    match value {
        CoverageInputV2::Fill { shape, rule } => SpatialCoverageV2::Fill {
            shape: SpatialShapeKeyV2::new(shape),
            rule: fill_rule(rule),
        },
        CoverageInputV2::RoundStroke { shape, width } => SpatialCoverageV2::RoundStroke {
            shape: SpatialShapeKeyV2::new(shape),
            width: scalar(width),
        },
    }
}

fn local_transform(value: AffineV2) -> SpatialLocalTransformV2 {
    SpatialLocalTransformV2::new(
        Affine2V2::new(
            scalar(value.values[0]),
            scalar(value.values[1]),
            scalar(value.values[2]),
            scalar(value.values[3]),
            scalar(value.values[4]),
            scalar(value.values[5]),
        ),
        point(value.origin),
    )
}

fn anchor(value: [AnchorComponentV2; 2]) -> SpatialAnchorV2 {
    SpatialAnchorV2::new(anchor_component(value[0]), anchor_component(value[1]))
}

fn anchor_component(value: AnchorComponentV2) -> SpatialAnchorComponentV2 {
    match value {
        AnchorComponentV2::Start => SpatialAnchorComponentV2::Start,
        AnchorComponentV2::Center => SpatialAnchorComponentV2::Center,
        AnchorComponentV2::End => SpatialAnchorComponentV2::End,
    }
}

fn anchor_target(value: AnchorTargetV2) -> SpatialAnchorTargetV2 {
    match value {
        AnchorTargetV2::Viewport => SpatialAnchorTargetV2::Viewport,
        AnchorTargetV2::Parent => SpatialAnchorTargetV2::Parent,
        AnchorTargetV2::Node(key) => SpatialAnchorTargetV2::Node(SpatialNodeKeyV2::new(key)),
    }
}

fn fill_rule(value: FillRuleV2) -> SpatialFillRuleV2 {
    match value {
        FillRuleV2::NonZero => SpatialFillRuleV2::NonZero,
        FillRuleV2::EvenOdd => SpatialFillRuleV2::EvenOdd,
    }
}

fn fixed(value: i32) -> LayoutDimensionV1 {
    LayoutDimensionV1::new(value, value, value)
}

fn rgba(value: [u8; 4]) -> SpatialRgba8V2 {
    SpatialRgba8V2::new(value[0], value[1], value[2], value[3])
}

fn image_scalar(value: i64) -> u32 {
    u32::try_from(value / FIXED_ONE_V2).expect("registered image scalar is nonnegative integral")
}

fn scalar(value: i64) -> SpatialScalarV2 {
    SpatialScalarV2::new(value)
}
