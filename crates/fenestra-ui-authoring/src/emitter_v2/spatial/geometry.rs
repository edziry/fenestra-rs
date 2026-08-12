use fenestra_ui_ir::prototype::{
    SpatialBrushContentV2, SpatialBrushDeclarationV2, SpatialGradientStopV2,
    SpatialPathVerbRecipeV2, SpatialPolygonPointV2, SpatialShapeDeclarationV2,
    SpatialShapeGeometryV2,
};
use proc_macro2::TokenStream;

use crate::emitter::builder::{ir_call, ir_record};

use super::layout::point;
use super::records;
use super::value::{
    field_brush, field_color_binding, field_i64_binding, field_shape, field_u16, source_span,
};

pub(super) fn shapes(values: &[SpatialShapeDeclarationV2]) -> TokenStream {
    records(values.iter().map(shape).collect())
}

pub(super) fn brushes(values: &[SpatialBrushDeclarationV2]) -> TokenStream {
    records(values.iter().map(brush).collect())
}

fn shape(value: &SpatialShapeDeclarationV2) -> TokenStream {
    ir_call(
        &["SpatialShapeDeclarationV2", "new"],
        vec![
            field_shape(value.symbol()),
            geometry(value.geometry()),
            source_span(value.span()),
        ],
        true,
    )
}

fn geometry(value: &SpatialShapeGeometryV2) -> TokenStream {
    match value {
        SpatialShapeGeometryV2::Rect {
            origin,
            width,
            height,
        } => ir_record(
            &["SpatialShapeGeometryV2", "Rect"],
            vec![
                ("origin", point(*origin)),
                ("width", field_i64_binding(*width)),
                ("height", field_i64_binding(*height)),
            ],
        ),
        SpatialShapeGeometryV2::Circle { center, radius } => ir_record(
            &["SpatialShapeGeometryV2", "Circle"],
            vec![
                ("center", point(*center)),
                ("radius", field_i64_binding(*radius)),
            ],
        ),
        SpatialShapeGeometryV2::Polygon { points } => ir_record(
            &["SpatialShapeGeometryV2", "Polygon"],
            vec![(
                "points",
                records(points.iter().copied().map(polygon_point).collect()),
            )],
        ),
        SpatialShapeGeometryV2::Path { verbs } => ir_record(
            &["SpatialShapeGeometryV2", "Path"],
            vec![(
                "verbs",
                records(verbs.iter().copied().map(path_verb).collect()),
            )],
        ),
    }
}

fn polygon_point(value: SpatialPolygonPointV2) -> TokenStream {
    ir_call(
        &["SpatialPolygonPointV2", "new"],
        vec![point(value.point()), source_span(value.span())],
        true,
    )
}

fn path_verb(value: SpatialPathVerbRecipeV2) -> TokenStream {
    match value {
        SpatialPathVerbRecipeV2::MoveTo { to, span } => ir_record(
            &["SpatialPathVerbRecipeV2", "MoveTo"],
            vec![("to", point(to)), ("span", source_span(span))],
        ),
        SpatialPathVerbRecipeV2::LineTo { to, span } => ir_record(
            &["SpatialPathVerbRecipeV2", "LineTo"],
            vec![("to", point(to)), ("span", source_span(span))],
        ),
        SpatialPathVerbRecipeV2::QuadraticTo { control, to, span } => ir_record(
            &["SpatialPathVerbRecipeV2", "QuadraticTo"],
            vec![
                ("control", point(control)),
                ("to", point(to)),
                ("span", source_span(span)),
            ],
        ),
        SpatialPathVerbRecipeV2::CubicTo {
            control1,
            control2,
            to,
            span,
        } => ir_record(
            &["SpatialPathVerbRecipeV2", "CubicTo"],
            vec![
                ("control1", point(control1)),
                ("control2", point(control2)),
                ("to", point(to)),
                ("span", source_span(span)),
            ],
        ),
        SpatialPathVerbRecipeV2::Close { span } => ir_record(
            &["SpatialPathVerbRecipeV2", "Close"],
            vec![("span", source_span(span))],
        ),
    }
}

fn brush(value: &SpatialBrushDeclarationV2) -> TokenStream {
    ir_call(
        &["SpatialBrushDeclarationV2", "new"],
        vec![
            field_brush(value.symbol()),
            brush_content(value.content()),
            source_span(value.span()),
        ],
        true,
    )
}

fn brush_content(value: &SpatialBrushContentV2) -> TokenStream {
    match value {
        SpatialBrushContentV2::Solid { color } => ir_record(
            &["SpatialBrushContentV2", "Solid"],
            vec![("color", field_color_binding(*color))],
        ),
        SpatialBrushContentV2::LinearGradient { start, end, stops } => ir_record(
            &["SpatialBrushContentV2", "LinearGradient"],
            vec![
                ("start", point(*start)),
                ("end", point(*end)),
                (
                    "stops",
                    records(stops.iter().copied().map(gradient_stop).collect()),
                ),
            ],
        ),
    }
}

fn gradient_stop(value: SpatialGradientStopV2) -> TokenStream {
    ir_call(
        &["SpatialGradientStopV2", "new"],
        vec![
            field_u16(value.offset()),
            field_color_binding(value.color()),
            source_span(value.span()),
        ],
        true,
    )
}
