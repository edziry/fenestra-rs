use crate::*;
use fenestra_ui_ir::prototype::{InputPolicy, SchemaNamespace, SchemaRevision, TemplateNodeId};

use super::round_trips::*;

#[test]
fn content_and_program_getters_preserve_every_constructor_slot() {
    let address = SpatialClipAddressV2::new(
        field(SpatialNodeSymbolV2::new(1), 200),
        field(SpatialClipSymbolV2::new(2), 201),
    );
    assert_eq!(address.owner(), field(SpatialNodeSymbolV2::new(1), 200));
    assert_eq!(address.clip(), field(SpatialClipSymbolV2::new(2), 201));

    let polygon = SpatialPolygonPointV2::new(point(3, 4, 202), span(204));
    assert_eq!(polygon.point(), point(3, 4, 202));
    assert_eq!(polygon.span(), span(204));

    let shape = SpatialShapeDeclarationV2::new(
        field(SpatialShapeSymbolV2::new(5), 205),
        SpatialShapeGeometryV2::Polygon {
            points: vec![polygon],
        },
        span(206),
    );
    assert_eq!(shape.symbol(), field(SpatialShapeSymbolV2::new(5), 205));
    assert!(
        matches!(shape.geometry(), SpatialShapeGeometryV2::Polygon { points } if points == &[polygon])
    );
    assert_eq!(shape.span(), span(206));

    let stop = SpatialGradientStopV2::new(
        field(7, 207),
        field(SpatialBindingV2::Literal([8, 9, 10, 11]), 208),
        span(209),
    );
    assert_eq!(stop.offset(), field(7, 207));
    assert_eq!(
        stop.color(),
        field(SpatialBindingV2::Literal([8, 9, 10, 11]), 208)
    );
    assert_eq!(stop.span(), span(209));

    let brush = SpatialBrushDeclarationV2::new(
        field(SpatialBrushSymbolV2::new(12), 210),
        SpatialBrushContentV2::LinearGradient {
            start: point(13, 14, 211),
            end: point(15, 16, 213),
            stops: vec![stop],
        },
        span(215),
    );
    assert_eq!(brush.symbol(), field(SpatialBrushSymbolV2::new(12), 210));
    assert!(
        matches!(brush.content(), SpatialBrushContentV2::LinearGradient { start, end, stops } if *start == point(13, 14, 211) && *end == point(15, 16, 213) && stops == &[stop])
    );
    assert_eq!(brush.span(), span(215));

    let clip = SpatialClipDeclarationV2::new(
        field(SpatialClipSymbolV2::new(17), 216),
        Some(address),
        field(SpatialShapeSymbolV2::new(18), 217),
        SpatialFillRuleV2::EvenOdd,
        span(218),
    );
    assert_eq!(clip.symbol(), field(SpatialClipSymbolV2::new(17), 216));
    assert_eq!(clip.parent(), Some(address));
    assert_eq!(clip.shape(), field(SpatialShapeSymbolV2::new(18), 217));
    assert_eq!(clip.fill_rule(), SpatialFillRuleV2::EvenOdd);
    assert_eq!(clip.span(), span(218));

    let coverage = SpatialCoverageRecipeV2::RoundStroke {
        shape: field(SpatialShapeSymbolV2::new(19), 219),
        width: fixed(20, 220),
    };
    let hit = SpatialHitRecipeV2::new(
        coverage,
        Some(address),
        field(SpatialBindingV2::Literal(InputPolicy::Ignore), 221),
        span(222),
    );
    assert_eq!(hit.coverage(), coverage);
    assert_eq!(hit.clip(), Some(address));
    assert_eq!(
        hit.input_policy(),
        field(SpatialBindingV2::Literal(InputPolicy::Ignore), 221)
    );
    assert_eq!(hit.span(), span(222));

    let semantic = SpatialSemanticRecipeV2::new(
        field(SpatialShapeSymbolV2::new(23), 223),
        SpatialFillRuleV2::NonZero,
        Some(address),
        span(224),
    );
    assert_eq!(semantic.shape(), field(SpatialShapeSymbolV2::new(23), 223));
    assert_eq!(semantic.fill_rule(), SpatialFillRuleV2::NonZero);
    assert_eq!(semantic.clip(), Some(address));
    assert_eq!(semantic.span(), span(224));

    let image = SpatialImageDeclarationV2::new(
        field(SpatialImageSymbolV2::new(25), 225),
        field(26, 226),
        field(27, 227),
        field(28, 228),
        vec![29, 30, 31].into_boxed_slice(),
        span(229),
    );
    assert_eq!(image.symbol(), field(SpatialImageSymbolV2::new(25), 225));
    assert_eq!(
        [image.width(), image.height(), image.stride()],
        [field(26, 226), field(27, 227), field(28, 228)]
    );
    assert_eq!(image.bytes(), &[29, 30, 31]);
    assert_eq!(image.span(), span(229));

    assert_node_and_program_round_trip(shape, brush, clip, hit, semantic, image);
}

fn assert_node_and_program_round_trip(
    shape: SpatialShapeDeclarationV2,
    brush: SpatialBrushDeclarationV2,
    clip: SpatialClipDeclarationV2,
    hit: SpatialHitRecipeV2,
    semantic: SpatialSemanticRecipeV2,
    image: SpatialImageDeclarationV2,
) {
    let paint = coverage_paint(400);
    let node = SpatialNodeDeclarationV2::new(
        field(SpatialNodeSymbolV2::new(41), 410),
        field(TemplateNodeId::new(42), 411),
        SpatialNodeParentV2::Node(field(SpatialNodeSymbolV2::new(43), 412)),
        SpatialPlacementRecipeV2::Layout(layout(420)),
        container(440),
        vec![shape.clone()],
        vec![brush.clone()],
        vec![clip],
        vec![paint],
        vec![hit],
        vec![semantic],
        span(450),
    );
    assert_eq!(node.symbol(), field(SpatialNodeSymbolV2::new(41), 410));
    assert_eq!(node.template(), field(TemplateNodeId::new(42), 411));
    assert_eq!(
        node.parent(),
        SpatialNodeParentV2::Node(field(SpatialNodeSymbolV2::new(43), 412))
    );
    assert_eq!(
        node.placement(),
        SpatialPlacementRecipeV2::Layout(layout(420))
    );
    assert_eq!(node.container(), container(440));
    assert_eq!(node.shapes(), &[shape]);
    assert_eq!(node.brushes(), &[brush]);
    assert_eq!(node.clips(), &[clip]);
    assert_eq!(node.paint_items(), &[paint]);
    assert_eq!(node.hit_items(), &[hit]);
    assert_eq!(node.semantic_items(), &[semantic]);
    assert_eq!(node.span(), span(450));

    let program = SpatialProgramV2::new(
        SUPPORTED_SPATIAL_FORMAT,
        SchemaNamespace::new(51),
        SchemaRevision::new(52),
        viewport(460),
        vec![node.clone()],
        vec![image.clone()],
        span(470),
    );
    assert_eq!(program.format(), SUPPORTED_SPATIAL_FORMAT);
    assert_eq!(program.schema_namespace(), SchemaNamespace::new(51));
    assert_eq!(program.schema_revision(), SchemaRevision::new(52));
    assert_eq!(program.viewport_container(), viewport(460));
    assert_eq!(program.nodes(), &[node]);
    assert_eq!(program.images(), &[image]);
    assert_eq!(program.span(), span(470));
}
