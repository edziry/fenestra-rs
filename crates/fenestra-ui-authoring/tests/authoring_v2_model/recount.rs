use std::collections::HashSet;

use fenestra_ui_authoring::prototype::AnchorKindV2;
use fenestra_ui_ir::prototype::{
    SourceId, SourceSpan, SpatialBrushContentV2, SpatialClipAddressV2, SpatialCoverageRecipeV2,
    SpatialFieldV2, SpatialNodeDeclarationV2, SpatialNodeParentV2, SpatialPaintRecipeV2,
    SpatialPathVerbRecipeV2, SpatialPlacementRecipeV2, SpatialPointRecipeV2, SpatialProgramV2,
    SpatialShapeGeometryV2, SpatialTransformRecipeV2,
};

use crate::support;

macro_rules! fields {
    ($spans:expr; $($field:expr),+ $(,)?) => {{
        $(field($spans, $field);)+
    }};
}

#[test]
fn independent_raw_walk_recounts_exactly_264_spatial_fields() {
    let compiled = support::compile_fen(support::FIXTURE);
    let spans = field_spans(compiled.spatial());
    assert_eq!(spans.len(), 264);
    assert_eq!(spans.iter().copied().collect::<HashSet<_>>().len(), 264);
    for span in &spans {
        let SourceSpan::Bytes { source, start, end } = *span else {
            panic!("every spatial field must have a logical byte anchor");
        };
        assert_eq!(source, SourceId::new(0));
        assert_eq!(end, start + 1);
    }

    let map_fields = compiled
        .source_map()
        .entries()
        .iter()
        .filter(|entry| entry.anchor_kind() == AnchorKindV2::SpatialField)
        .map(|entry| entry.logical_span())
        .collect::<HashSet<_>>();
    assert_eq!(map_fields.len(), 264);
    assert_eq!(spans.into_iter().collect::<HashSet<_>>(), map_fields);
    assert_eq!(compiled.logical_source_catalog(), &[b'@'; 380]);
    assert_eq!(compiled.source_map().entries().len(), 380);
}

fn field_spans(program: &SpatialProgramV2) -> Vec<SourceSpan> {
    let mut spans = Vec::new();
    let viewport = program.viewport_container();
    fields!(&mut spans; viewport.left(), viewport.right(), viewport.top(), viewport.bottom(), viewport.gap());
    for image in program.images() {
        fields!(&mut spans; image.symbol(), image.width(), image.height(), image.stride());
    }
    for node in program.nodes() {
        collect_node(node, &mut spans);
    }
    spans
}

fn collect_node(node: &SpatialNodeDeclarationV2, spans: &mut Vec<SourceSpan>) {
    fields!(spans; node.symbol(), node.template());
    if let SpatialNodeParentV2::Node(parent) = node.parent() {
        field(spans, parent);
    }
    let container = node.container();
    let padding = container.padding();
    fields!(spans; padding.left(), padding.right(), padding.top(), padding.bottom(), container.gap());
    match node.placement() {
        SpatialPlacementRecipeV2::Layout(layout) => {
            let width = layout.width();
            let height = layout.height();
            fields!(spans; width.minimum(), width.preferred(), width.maximum());
            fields!(spans; height.minimum(), height.preferred(), height.maximum());
            collect_transform(layout.transform(), spans);
        }
        SpatialPlacementRecipeV2::Free(free) => {
            fields!(spans; free.width(), free.height());
            if let fenestra_ui_ir::prototype::SpatialAnchorTargetRecipeV2::Node(target) =
                free.target()
            {
                field(spans, target);
            }
            collect_point(free.offset(), spans);
            collect_transform(free.transform(), spans);
        }
    }
    for shape in node.shapes() {
        field(spans, shape.symbol());
        match shape.geometry() {
            SpatialShapeGeometryV2::Rect {
                origin,
                width,
                height,
            } => {
                collect_point(*origin, spans);
                fields!(spans; *width, *height);
            }
            SpatialShapeGeometryV2::Circle { center, radius } => {
                collect_point(*center, spans);
                field(spans, *radius);
            }
            SpatialShapeGeometryV2::Polygon { points } => {
                for point in points {
                    collect_point(point.point(), spans);
                }
            }
            SpatialShapeGeometryV2::Path { verbs } => {
                for verb in verbs {
                    collect_verb(*verb, spans);
                }
            }
        }
    }
    for brush in node.brushes() {
        field(spans, brush.symbol());
        match brush.content() {
            SpatialBrushContentV2::Solid { color } => field(spans, *color),
            SpatialBrushContentV2::LinearGradient { start, end, stops } => {
                collect_point(*start, spans);
                collect_point(*end, spans);
                for stop in stops {
                    fields!(spans; stop.offset(), stop.color());
                }
            }
        }
    }
    for clip in node.clips() {
        field(spans, clip.symbol());
        if let Some(address) = clip.parent() {
            collect_clip(address, spans);
        }
        field(spans, clip.shape());
    }
    for paint in node.paint_items() {
        collect_paint(*paint, spans);
    }
    for hit in node.hit_items() {
        collect_coverage(hit.coverage(), spans);
        if let Some(clip) = hit.clip() {
            collect_clip(clip, spans);
        }
        field(spans, hit.input_policy());
    }
    for semantic in node.semantic_items() {
        field(spans, semantic.shape());
        if let Some(clip) = semantic.clip() {
            collect_clip(clip, spans);
        }
    }
}

fn collect_transform(transform: SpatialTransformRecipeV2, spans: &mut Vec<SourceSpan>) {
    fields!(spans; transform.a(), transform.b(), transform.c(), transform.d(), transform.tx(), transform.ty());
    collect_point(transform.origin(), spans);
}

fn collect_point(point: SpatialPointRecipeV2, spans: &mut Vec<SourceSpan>) {
    fields!(spans; point.x(), point.y());
}

fn collect_clip(clip: SpatialClipAddressV2, spans: &mut Vec<SourceSpan>) {
    fields!(spans; clip.owner(), clip.clip());
}

fn collect_coverage(coverage: SpatialCoverageRecipeV2, spans: &mut Vec<SourceSpan>) {
    match coverage {
        SpatialCoverageRecipeV2::Fill { shape, .. } => field(spans, shape),
        SpatialCoverageRecipeV2::RoundStroke { shape, width } => {
            fields!(spans; shape, width);
        }
    }
}

fn collect_paint(paint: SpatialPaintRecipeV2, spans: &mut Vec<SourceSpan>) {
    match paint {
        SpatialPaintRecipeV2::CoveragePaint {
            coverage,
            brush,
            opacity,
            clip,
            ..
        } => {
            collect_coverage(coverage, spans);
            fields!(spans; brush, opacity);
            if let Some(clip) = clip {
                collect_clip(clip, spans);
            }
        }
        SpatialPaintRecipeV2::ImagePaint {
            image,
            source_x,
            source_y,
            source_width,
            source_height,
            destination_origin,
            destination_width,
            destination_height,
            opacity,
            clip,
            ..
        } => {
            fields!(spans; image, source_x, source_y, source_width, source_height);
            collect_point(destination_origin, spans);
            fields!(spans; destination_width, destination_height, opacity);
            if let Some(clip) = clip {
                collect_clip(clip, spans);
            }
        }
    }
}

fn collect_verb(verb: SpatialPathVerbRecipeV2, spans: &mut Vec<SourceSpan>) {
    match verb {
        SpatialPathVerbRecipeV2::MoveTo { to, .. } | SpatialPathVerbRecipeV2::LineTo { to, .. } => {
            collect_point(to, spans)
        }
        SpatialPathVerbRecipeV2::QuadraticTo { control, to, .. } => {
            collect_point(control, spans);
            collect_point(to, spans);
        }
        SpatialPathVerbRecipeV2::CubicTo {
            control1,
            control2,
            to,
            ..
        } => {
            collect_point(control1, spans);
            collect_point(control2, spans);
            collect_point(to, spans);
        }
        SpatialPathVerbRecipeV2::Close { .. } => {}
    }
}

fn field<T>(spans: &mut Vec<SourceSpan>, value: SpatialFieldV2<T>) {
    spans.push(value.span());
}
