use fenestra_ui_ir::prototype::{
    InputPolicy, SpatialAxisV2, SpatialBrushContentV2, SpatialBrushDeclarationV2,
    SpatialClipDeclarationV2, SpatialCoverageRecipeV2, SpatialFillRuleV2, SpatialGradientStopV2,
    SpatialHitRecipeV2, SpatialNodeDeclarationV2, SpatialNodeParentV2, SpatialPaintRecipeV2,
    SpatialPathVerbRecipeV2, SpatialPlacementRecipeV2, SpatialPolygonPointV2,
    SpatialSemanticRecipeV2, SpatialShapeDeclarationV2, SpatialShapeGeometryV2,
};

use super::super::value::{
    address, brush, field, fixed_lit, fixed_prop, i32_lit, i32_prop, image, input_lit, input_prop,
    node, point, rgba_lit, rgba_prop, shape, span, template,
};
use super::layout::{container, identity, layout_placement};

pub(super) fn scene() -> SpatialNodeDeclarationV2 {
    SpatialNodeDeclarationV2::new(
        node(0, 65),
        template(0, 66),
        SpatialNodeParentV2::Viewport,
        SpatialPlacementRecipeV2::Layout(layout_placement(
            [i32_lit(0, 74), i32_prop(0, 75), i32_lit(240, 76)],
            [i32_lit(0, 77), i32_prop(1, 78), i32_lit(180, 79)],
            identity(81),
        )),
        container(
            SpatialAxisV2::Column,
            [
                i32_prop(2, 68),
                i32_lit(2, 69),
                i32_prop(2, 70),
                i32_lit(2, 71),
            ],
            i32_prop(2, 72),
        ),
        shapes(),
        brushes(),
        clips(),
        paints(),
        hits(),
        semantics(),
        span(64),
    )
}

fn shapes() -> Vec<SpatialShapeDeclarationV2> {
    vec![
        SpatialShapeDeclarationV2::new(
            shape(0, 90),
            SpatialShapeGeometryV2::Rect {
                origin: point(fixed_lit(0, 91), fixed_lit(0, 92)),
                width: fixed_prop(0, 93),
                height: fixed_prop(1, 94),
            },
            span(89),
        ),
        SpatialShapeDeclarationV2::new(
            shape(1, 96),
            SpatialShapeGeometryV2::Circle {
                center: point(fixed_lit(327_680, 97), fixed_lit(327_680, 98)),
                radius: fixed_lit(131_072, 99),
            },
            span(95),
        ),
        SpatialShapeDeclarationV2::new(
            shape(2, 101),
            SpatialShapeGeometryV2::Polygon {
                points: vec![
                    polygon_point(0, 0, 102, 103, 104),
                    polygon_point(655_360, 0, 105, 106, 107),
                    polygon_point(327_680, 524_288, 108, 109, 110),
                ],
            },
            span(100),
        ),
        SpatialShapeDeclarationV2::new(
            shape(3, 112),
            SpatialShapeGeometryV2::Path {
                verbs: path_verbs(),
            },
            span(111),
        ),
    ]
}

fn polygon_point(
    x: i64,
    y: i64,
    record: u32,
    x_anchor: u32,
    y_anchor: u32,
) -> SpatialPolygonPointV2 {
    SpatialPolygonPointV2::new(
        point(fixed_lit(x, x_anchor), fixed_lit(y, y_anchor)),
        span(record),
    )
}

fn path_verbs() -> Vec<SpatialPathVerbRecipeV2> {
    vec![
        SpatialPathVerbRecipeV2::MoveTo {
            to: point(fixed_lit(0, 114), fixed_lit(0, 115)),
            span: span(113),
        },
        SpatialPathVerbRecipeV2::LineTo {
            to: point(fixed_lit(655_360, 117), fixed_lit(0, 118)),
            span: span(116),
        },
        SpatialPathVerbRecipeV2::QuadraticTo {
            control: point(fixed_lit(983_040, 120), fixed_lit(327_680, 121)),
            to: point(fixed_lit(655_360, 122), fixed_lit(655_360, 123)),
            span: span(119),
        },
        SpatialPathVerbRecipeV2::CubicTo {
            control1: point(fixed_lit(327_680, 125), fixed_lit(983_040, 126)),
            control2: point(fixed_lit(0, 127), fixed_lit(983_040, 128)),
            to: point(fixed_lit(0, 129), fixed_lit(655_360, 130)),
            span: span(124),
        },
        SpatialPathVerbRecipeV2::Close { span: span(131) },
    ]
}

fn brushes() -> Vec<SpatialBrushDeclarationV2> {
    vec![
        SpatialBrushDeclarationV2::new(
            brush(0, 133),
            SpatialBrushContentV2::Solid {
                color: rgba_prop(4, 134),
            },
            span(132),
        ),
        SpatialBrushDeclarationV2::new(
            brush(1, 136),
            SpatialBrushContentV2::LinearGradient {
                start: point(fixed_lit(0, 137), fixed_lit(0, 138)),
                end: point(fixed_lit(655_360, 139), fixed_lit(0, 140)),
                stops: vec![
                    SpatialGradientStopV2::new(field(0, 142), rgba_prop(5, 143), span(141)),
                    SpatialGradientStopV2::new(
                        field(32_768, 145),
                        rgba_lit([128, 64, 32, 255], 146),
                        span(144),
                    ),
                    SpatialGradientStopV2::new(field(65_535, 148), rgba_prop(4, 149), span(147)),
                ],
            },
            span(135),
        ),
    ]
}

fn clips() -> Vec<SpatialClipDeclarationV2> {
    vec![
        SpatialClipDeclarationV2::new(
            super::super::value::clip(0, 151),
            None,
            shape(0, 152),
            SpatialFillRuleV2::NonZero,
            span(150),
        ),
        SpatialClipDeclarationV2::new(
            super::super::value::clip(1, 154),
            Some(address(0, 155, 0, 156)),
            shape(2, 157),
            SpatialFillRuleV2::EvenOdd,
            span(153),
        ),
    ]
}

fn paints() -> Vec<SpatialPaintRecipeV2> {
    vec![
        SpatialPaintRecipeV2::CoveragePaint {
            coverage: fill(0, 159, SpatialFillRuleV2::NonZero),
            brush: brush(0, 160),
            opacity: field(255, 161),
            clip: Some(address(0, 162, 1, 163)),
            span: span(158),
        },
        SpatialPaintRecipeV2::CoveragePaint {
            coverage: stroke(3, 165, fixed_prop(2, 166)),
            brush: brush(1, 167),
            opacity: field(200, 168),
            clip: None,
            span: span(164),
        },
        SpatialPaintRecipeV2::ImagePaint {
            image: image(0, 170),
            source_x: field(0, 171),
            source_y: field(0, 172),
            source_width: field(2, 173),
            source_height: field(2, 174),
            destination_origin: point(fixed_lit(131_072, 175), fixed_lit(131_072, 176)),
            destination_width: fixed_lit(524_288, 177),
            destination_height: fixed_lit(524_288, 178),
            opacity: field(192, 179),
            clip: Some(address(0, 180, 0, 181)),
            span: span(169),
        },
    ]
}

fn hits() -> Vec<SpatialHitRecipeV2> {
    vec![
        SpatialHitRecipeV2::new(
            fill(2, 183, SpatialFillRuleV2::EvenOdd),
            Some(address(0, 184, 1, 185)),
            input_prop(7, 186),
            span(182),
        ),
        SpatialHitRecipeV2::new(
            stroke(3, 188, fixed_lit(65_536, 189)),
            None,
            input_lit(InputPolicy::Accept, 190),
            span(187),
        ),
        SpatialHitRecipeV2::new(
            fill(1, 192, SpatialFillRuleV2::NonZero),
            None,
            input_lit(InputPolicy::Ignore, 193),
            span(191),
        ),
    ]
}

fn semantics() -> Vec<SpatialSemanticRecipeV2> {
    vec![
        SpatialSemanticRecipeV2::new(
            shape(1, 195),
            SpatialFillRuleV2::NonZero,
            Some(address(0, 196, 0, 197)),
            span(194),
        ),
        SpatialSemanticRecipeV2::new(shape(2, 199), SpatialFillRuleV2::EvenOdd, None, span(198)),
    ]
}

fn fill(shape_value: u32, anchor: u32, rule: SpatialFillRuleV2) -> SpatialCoverageRecipeV2 {
    SpatialCoverageRecipeV2::Fill {
        shape: shape(shape_value, anchor),
        rule,
    }
}

fn stroke(
    shape_value: u32,
    shape_anchor: u32,
    width: fenestra_ui_ir::prototype::SpatialFieldV2<
        fenestra_ui_ir::prototype::SpatialBindingV2<i64>,
    >,
) -> SpatialCoverageRecipeV2 {
    SpatialCoverageRecipeV2::RoundStroke {
        shape: shape(shape_value, shape_anchor),
        width,
    }
}
