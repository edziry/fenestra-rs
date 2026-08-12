use fenestra_ui_ir::prototype::{
    InputPolicy, SourceSpan, SpatialBrushContentV2, SpatialBrushDeclarationV2,
    SpatialBrushSymbolV2, SpatialClipAddressV2, SpatialClipDeclarationV2, SpatialClipSymbolV2,
    SpatialCoverageRecipeV2, SpatialFieldV2, SpatialFillRuleV2, SpatialGradientStopV2,
    SpatialHitRecipeV2, SpatialImageDeclarationV2, SpatialImageSymbolV2, SpatialPaintRecipeV2,
    SpatialPathVerbRecipeV2, SpatialPolygonPointV2, SpatialSemanticRecipeV2,
    SpatialShapeDeclarationV2, SpatialShapeGeometryV2, SpatialShapeSymbolV2,
};

use super::mapper_program::{MapperFault, OUTER_SYMBOL, SCALE, SpanCursor, point};
use super::{COLOR, IMAGE_COLOR, POLICY, WIDTH};

const RECT: SpatialShapeSymbolV2 = SpatialShapeSymbolV2::new(11);
const CIRCLE: SpatialShapeSymbolV2 = SpatialShapeSymbolV2::new(12);
const POLYGON: SpatialShapeSymbolV2 = SpatialShapeSymbolV2::new(13);
const PATH: SpatialShapeSymbolV2 = SpatialShapeSymbolV2::new(14);
const SOLID: SpatialBrushSymbolV2 = SpatialBrushSymbolV2::new(21);
const GRADIENT: SpatialBrushSymbolV2 = SpatialBrushSymbolV2::new(22);
const OUTER_CLIP: SpatialClipSymbolV2 = SpatialClipSymbolV2::new(31);
const INNER_CLIP: SpatialClipSymbolV2 = SpatialClipSymbolV2::new(32);
const IMAGE: SpatialImageSymbolV2 = SpatialImageSymbolV2::new(41);

pub(super) struct Content {
    pub shapes: Vec<SpatialShapeDeclarationV2>,
    pub brushes: Vec<SpatialBrushDeclarationV2>,
    pub clips: Vec<SpatialClipDeclarationV2>,
    pub paints: Vec<SpatialPaintRecipeV2>,
    pub hits: Vec<SpatialHitRecipeV2>,
    pub semantics: Vec<SpatialSemanticRecipeV2>,
    pub spans: ContentSpans,
}

pub(super) struct ContentSpans {
    pub image: SpatialImageDeclarationV2,
    pub path_first_verb: SourceSpan,
    pub gradient_last_offset: SourceSpan,
}

pub(super) fn content(spans: &mut SpanCursor, fault: MapperFault) -> Content {
    let rect = rect(spans);
    let circle = circle(spans);
    let polygon = polygon(spans);
    let (path, path_first_verb) = path(spans, fault == MapperFault::PathVerb);
    let solid = solid(spans);
    let (gradient, gradient_last_offset) = gradient(spans, fault == MapperFault::GradientStop);
    let clips = clips(spans);
    let paints = paints(spans);
    let hits = vec![SpatialHitRecipeV2::new(
        SpatialCoverageRecipeV2::RoundStroke {
            shape: spans.field(POLYGON),
            width: spans.binding_literal(SCALE),
        },
        Some(clip_address(spans, INNER_CLIP)),
        spans.binding_property::<InputPolicy>(POLICY),
        spans.take(),
    )];
    let semantics = vec![SpatialSemanticRecipeV2::new(
        spans.field(PATH),
        SpatialFillRuleV2::EvenOdd,
        Some(clip_address(spans, OUTER_CLIP)),
        spans.take(),
    )];
    let image = image(spans);
    Content {
        shapes: vec![rect, circle, polygon, path],
        brushes: vec![solid, gradient],
        clips,
        paints,
        hits,
        semantics,
        spans: ContentSpans {
            image,
            path_first_verb,
            gradient_last_offset,
        },
    }
}

fn rect(spans: &mut SpanCursor) -> SpatialShapeDeclarationV2 {
    SpatialShapeDeclarationV2::new(
        spans.field(RECT),
        SpatialShapeGeometryV2::Rect {
            origin: point(spans, 0, 0),
            width: spans.binding_property(WIDTH),
            height: spans.binding_literal(8 * SCALE),
        },
        spans.take(),
    )
}

fn circle(spans: &mut SpanCursor) -> SpatialShapeDeclarationV2 {
    SpatialShapeDeclarationV2::new(
        spans.field(CIRCLE),
        SpatialShapeGeometryV2::Circle {
            center: point(spans, 4 * SCALE, 4 * SCALE),
            radius: spans.binding_literal(3 * SCALE),
        },
        spans.take(),
    )
}

fn polygon(spans: &mut SpanCursor) -> SpatialShapeDeclarationV2 {
    SpatialShapeDeclarationV2::new(
        spans.field(POLYGON),
        SpatialShapeGeometryV2::Polygon {
            points: vec![
                polygon_point(spans, 0, 0),
                polygon_point(spans, 6 * SCALE, 0),
                polygon_point(spans, 0, 6 * SCALE),
            ],
        },
        spans.take(),
    )
}

fn polygon_point(spans: &mut SpanCursor, x: i64, y: i64) -> SpatialPolygonPointV2 {
    SpatialPolygonPointV2::new(point(spans, x, y), spans.take())
}

fn path(spans: &mut SpanCursor, malformed: bool) -> (SpatialShapeDeclarationV2, SourceSpan) {
    let first_span = spans.take();
    let first_point = point(spans, 0, 0);
    let first = if malformed {
        SpatialPathVerbRecipeV2::LineTo {
            to: first_point,
            span: first_span,
        }
    } else {
        SpatialPathVerbRecipeV2::MoveTo {
            to: first_point,
            span: first_span,
        }
    };
    let verbs = vec![
        first,
        SpatialPathVerbRecipeV2::LineTo {
            to: point(spans, 6 * SCALE, 0),
            span: spans.take(),
        },
        SpatialPathVerbRecipeV2::QuadraticTo {
            control: point(spans, 7 * SCALE, SCALE),
            to: point(spans, 6 * SCALE, 3 * SCALE),
            span: spans.take(),
        },
        SpatialPathVerbRecipeV2::CubicTo {
            control1: point(spans, 5 * SCALE, 5 * SCALE),
            control2: point(spans, SCALE, 5 * SCALE),
            to: point(spans, 0, 3 * SCALE),
            span: spans.take(),
        },
        SpatialPathVerbRecipeV2::Close { span: spans.take() },
    ];
    (
        SpatialShapeDeclarationV2::new(
            spans.field(PATH),
            SpatialShapeGeometryV2::Path { verbs },
            spans.take(),
        ),
        first_span,
    )
}

fn solid(spans: &mut SpanCursor) -> SpatialBrushDeclarationV2 {
    SpatialBrushDeclarationV2::new(
        spans.field(SOLID),
        SpatialBrushContentV2::Solid {
            color: spans.binding_property::<[u8; 4]>(COLOR),
        },
        spans.take(),
    )
}

fn gradient(spans: &mut SpanCursor, malformed: bool) -> (SpatialBrushDeclarationV2, SourceSpan) {
    let first = SpatialGradientStopV2::new(
        spans.field(0),
        spans.binding_literal([0, 0, 0, 255]),
        spans.take(),
    );
    let last_offset = spans.take();
    let last = SpatialGradientStopV2::new(
        SpatialFieldV2::new(if malformed { u16::MAX - 1 } else { u16::MAX }, last_offset),
        spans.binding_property::<[u8; 4]>(COLOR),
        spans.take(),
    );
    (
        SpatialBrushDeclarationV2::new(
            spans.field(GRADIENT),
            SpatialBrushContentV2::LinearGradient {
                start: point(spans, 0, 0),
                end: point(spans, 8 * SCALE, 0),
                stops: vec![first, last],
            },
            spans.take(),
        ),
        last_offset,
    )
}

fn clips(spans: &mut SpanCursor) -> Vec<SpatialClipDeclarationV2> {
    vec![
        SpatialClipDeclarationV2::new(
            spans.field(OUTER_CLIP),
            None,
            spans.field(RECT),
            SpatialFillRuleV2::NonZero,
            spans.take(),
        ),
        SpatialClipDeclarationV2::new(
            spans.field(INNER_CLIP),
            Some(clip_address(spans, OUTER_CLIP)),
            spans.field(CIRCLE),
            SpatialFillRuleV2::EvenOdd,
            spans.take(),
        ),
    ]
}

fn paints(spans: &mut SpanCursor) -> Vec<SpatialPaintRecipeV2> {
    vec![
        SpatialPaintRecipeV2::CoveragePaint {
            coverage: SpatialCoverageRecipeV2::Fill {
                shape: spans.field(RECT),
                rule: SpatialFillRuleV2::NonZero,
            },
            brush: spans.field(SOLID),
            opacity: spans.field(255),
            clip: Some(clip_address(spans, OUTER_CLIP)),
            span: spans.take(),
        },
        SpatialPaintRecipeV2::CoveragePaint {
            coverage: SpatialCoverageRecipeV2::RoundStroke {
                shape: spans.field(CIRCLE),
                width: spans.binding_literal(SCALE),
            },
            brush: spans.field(GRADIENT),
            opacity: spans.field(200),
            clip: Some(clip_address(spans, INNER_CLIP)),
            span: spans.take(),
        },
        SpatialPaintRecipeV2::ImagePaint {
            image: spans.field(IMAGE),
            source_x: spans.field(0),
            source_y: spans.field(0),
            source_width: spans.field(1),
            source_height: spans.field(1),
            destination_origin: point(spans, 0, 0),
            destination_width: spans.binding_literal(SCALE),
            destination_height: spans.binding_literal(SCALE),
            opacity: spans.field(255),
            clip: None,
            span: spans.take(),
        },
    ]
}

fn clip_address(spans: &mut SpanCursor, clip: SpatialClipSymbolV2) -> SpatialClipAddressV2 {
    SpatialClipAddressV2::new(spans.field(OUTER_SYMBOL), spans.field(clip))
}

fn image(spans: &mut SpanCursor) -> SpatialImageDeclarationV2 {
    SpatialImageDeclarationV2::new(
        spans.field(IMAGE),
        spans.field(1),
        spans.field(1),
        spans.field(4),
        IMAGE_COLOR.into(),
        spans.take(),
    )
}
