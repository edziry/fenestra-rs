use crate::*;
use fenestra_ui_ir::prototype::PropertyId;

use super::round_trips::*;

#[test]
fn payload_enums_are_exhaustive_and_every_variant_is_constructible() {
    assert_eq!(binding_kind(&SpatialBindingV2::Literal(1_i32)), 0);
    assert_eq!(
        binding_kind(&SpatialBindingV2::<i32>::Property(PropertyId::new(2))),
        1
    );
    assert_eq!(node_parent_kind(&SpatialNodeParentV2::Viewport), 0);
    assert_eq!(
        node_parent_kind(&SpatialNodeParentV2::Node(field(
            SpatialNodeSymbolV2::new(3),
            300,
        ))),
        1
    );
    assert_eq!(
        [
            anchor_kind(&SpatialAnchorTargetRecipeV2::Viewport),
            anchor_kind(&SpatialAnchorTargetRecipeV2::Parent),
            anchor_kind(&SpatialAnchorTargetRecipeV2::Node(field(
                SpatialNodeSymbolV2::new(4),
                301,
            ))),
        ],
        [0, 1, 2]
    );
    assert_eq!(
        [
            placement_kind(&SpatialPlacementRecipeV2::Layout(layout(310))),
            placement_kind(&SpatialPlacementRecipeV2::Free(free(330))),
        ],
        [0, 1]
    );

    let verbs = [
        SpatialPathVerbRecipeV2::MoveTo {
            to: point(1, 2, 350),
            span: span(352),
        },
        SpatialPathVerbRecipeV2::LineTo {
            to: point(3, 4, 353),
            span: span(355),
        },
        SpatialPathVerbRecipeV2::QuadraticTo {
            control: point(5, 6, 356),
            to: point(7, 8, 358),
            span: span(360),
        },
        SpatialPathVerbRecipeV2::CubicTo {
            control1: point(9, 10, 361),
            control2: point(11, 12, 363),
            to: point(13, 14, 365),
            span: span(367),
        },
        SpatialPathVerbRecipeV2::Close { span: span(368) },
    ];
    assert_eq!(verbs.each_ref().map(path_kind), [0, 1, 2, 3, 4]);
    assert_eq!(
        verbs.map(|verb| verb.span()),
        [span(352), span(355), span(360), span(367), span(368)]
    );

    assert_eq!(
        [
            shape_kind(&rect()),
            shape_kind(&circle()),
            shape_kind(&SpatialShapeGeometryV2::Polygon { points: Vec::new() }),
            shape_kind(&SpatialShapeGeometryV2::Path { verbs: Vec::new() }),
        ],
        [0, 1, 2, 3]
    );
    assert_eq!([brush_kind(&solid()), brush_kind(&gradient())], [0, 1]);
    assert_eq!([coverage_kind(&fill()), coverage_kind(&stroke())], [0, 1]);
    let paints = [coverage_paint(370), image_paint(380)];
    assert_eq!(paints.each_ref().map(paint_kind), [0, 1]);
    assert_eq!(paints.map(|paint| paint.span()), [span(375), span(389)]);
}

fn binding_kind<T>(value: &SpatialBindingV2<T>) -> u8 {
    match value {
        SpatialBindingV2::Literal(_) => 0,
        SpatialBindingV2::Property(_) => 1,
    }
}

fn node_parent_kind(value: &SpatialNodeParentV2) -> u8 {
    match value {
        SpatialNodeParentV2::Viewport => 0,
        SpatialNodeParentV2::Node(_) => 1,
    }
}

fn anchor_kind(value: &SpatialAnchorTargetRecipeV2) -> u8 {
    match value {
        SpatialAnchorTargetRecipeV2::Viewport => 0,
        SpatialAnchorTargetRecipeV2::Parent => 1,
        SpatialAnchorTargetRecipeV2::Node(_) => 2,
    }
}

fn placement_kind(value: &SpatialPlacementRecipeV2) -> u8 {
    match value {
        SpatialPlacementRecipeV2::Layout(_) => 0,
        SpatialPlacementRecipeV2::Free(_) => 1,
    }
}

fn path_kind(value: &SpatialPathVerbRecipeV2) -> u8 {
    match value {
        SpatialPathVerbRecipeV2::MoveTo { .. } => 0,
        SpatialPathVerbRecipeV2::LineTo { .. } => 1,
        SpatialPathVerbRecipeV2::QuadraticTo { .. } => 2,
        SpatialPathVerbRecipeV2::CubicTo { .. } => 3,
        SpatialPathVerbRecipeV2::Close { .. } => 4,
    }
}

fn shape_kind(value: &SpatialShapeGeometryV2) -> u8 {
    match value {
        SpatialShapeGeometryV2::Rect { .. } => 0,
        SpatialShapeGeometryV2::Circle { .. } => 1,
        SpatialShapeGeometryV2::Polygon { .. } => 2,
        SpatialShapeGeometryV2::Path { .. } => 3,
    }
}

fn brush_kind(value: &SpatialBrushContentV2) -> u8 {
    match value {
        SpatialBrushContentV2::Solid { .. } => 0,
        SpatialBrushContentV2::LinearGradient { .. } => 1,
    }
}

fn coverage_kind(value: &SpatialCoverageRecipeV2) -> u8 {
    match value {
        SpatialCoverageRecipeV2::Fill { .. } => 0,
        SpatialCoverageRecipeV2::RoundStroke { .. } => 1,
    }
}

fn paint_kind(value: &SpatialPaintRecipeV2) -> u8 {
    match value {
        SpatialPaintRecipeV2::CoveragePaint { .. } => 0,
        SpatialPaintRecipeV2::ImagePaint { .. } => 1,
    }
}
