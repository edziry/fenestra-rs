use super::*;
use support::*;

fn assert_invalid(style: &ValidatedStyleProgram, program: SpatialProgramV2, source: SourceSpan) {
    assert_error(
        style,
        program,
        IrValidationErrorKind::InvalidSourceSpan,
        source,
    );
}

#[test]
fn node_record_and_symbol_spans_are_checked_before_linkage() {
    let style = style();
    let record_span = invalid_span(1600);
    let record = SpatialNodeDeclarationV2::new(
        field(SpatialNodeSymbolV2::new(0), invalid_span(1601)),
        field(TemplateNodeId::new(999), invalid_span(1602)),
        parent(999, 1603),
        placement(1604),
        container(1610),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        record_span,
    );
    assert_invalid(&style, program(vec![record]), record_span);

    let symbol_span = invalid_span(1620);
    let symbol = SpatialNodeDeclarationV2::new(
        field(SpatialNodeSymbolV2::new(0), symbol_span),
        field(ROOT, span(1621)),
        SpatialNodeParentV2::Viewport,
        placement(1622),
        container(1630),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        span(1635),
    );
    assert_invalid(&style, program(vec![symbol]), symbol_span);
}

#[test]
fn nested_shape_path_and_polygon_spans_are_checked() {
    let style = style();
    let verb_span = invalid_span(1640);
    let path = SpatialShapeDeclarationV2::new(
        field(SpatialShapeSymbolV2::new(0), span(1641)),
        SpatialShapeGeometryV2::Path {
            verbs: vec![SpatialPathVerbRecipeV2::MoveTo {
                to: point(0, 0, 1642),
                span: verb_span,
            }],
        },
        span(1644),
    );
    let path_node = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(1650),
        vec![path],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        1649,
    );
    assert_invalid(&style, program(vec![path_node]), verb_span);

    let point_span = invalid_span(1660);
    let polygon = SpatialShapeDeclarationV2::new(
        field(SpatialShapeSymbolV2::new(0), span(1661)),
        SpatialShapeGeometryV2::Polygon {
            points: vec![SpatialPolygonPointV2::new(point(0, 0, 1662), point_span)],
        },
        span(1664),
    );
    let polygon_node = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(1670),
        vec![polygon],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        1669,
    );
    assert_invalid(&style, program(vec![polygon_node]), point_span);
}

#[test]
fn gradient_stop_and_image_field_spans_are_checked() {
    let style = style();
    let stop_span = invalid_span(1680);
    let gradient = SpatialBrushDeclarationV2::new(
        field(SpatialBrushSymbolV2::new(0), span(1681)),
        SpatialBrushContentV2::LinearGradient {
            start: point(0, 0, 1682),
            end: point(1, 1, 1684),
            stops: vec![SpatialGradientStopV2::new(
                field(0, span(1686)),
                field(SpatialBindingV2::Literal([0, 0, 0, 255]), span(1687)),
                stop_span,
            )],
        },
        span(1688),
    );
    let gradient_node = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(1690),
        Vec::new(),
        vec![gradient],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        1689,
    );
    assert_invalid(&style, program(vec![gradient_node]), stop_span);

    let width_span = invalid_span(1700);
    let image = SpatialImageDeclarationV2::new(
        field(SpatialImageSymbolV2::new(0), span(1701)),
        field(1, width_span),
        field(1, span(1702)),
        field(4, span(1703)),
        vec![0, 0, 0, 0].into_boxed_slice(),
        span(1704),
    );
    let image_program = program_with(
        SUPPORTED_SPATIAL_FORMAT,
        NS,
        REV,
        viewport(1705),
        Vec::new(),
        vec![image],
        span(1706),
    );
    assert_invalid(&style, image_program, width_span);
}

fn item_node(
    paints: Vec<SpatialPaintRecipeV2>,
    hits: Vec<SpatialHitRecipeV2>,
    semantics: Vec<SpatialSemanticRecipeV2>,
    index: u32,
) -> SpatialNodeDeclarationV2 {
    node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(index),
        vec![shape(0, index + 10)],
        vec![brush(0, index + 20)],
        Vec::new(),
        paints,
        hits,
        semantics,
        index - 1,
    )
}

#[test]
fn paint_hit_and_semantic_record_spans_are_checked() {
    let style = style();
    let paint_span = invalid_span(1720);
    let paint = SpatialPaintRecipeV2::CoveragePaint {
        coverage: coverage(0, 1721),
        brush: field(SpatialBrushSymbolV2::new(0), span(1722)),
        opacity: field(255, span(1723)),
        clip: None,
        span: paint_span,
    };
    assert_invalid(
        &style,
        program(vec![item_node(vec![paint], Vec::new(), Vec::new(), 1730)]),
        paint_span,
    );

    let hit_span = invalid_span(1740);
    let hit = SpatialHitRecipeV2::new(
        coverage(0, 1741),
        None,
        field(SpatialBindingV2::Literal(InputPolicy::Accept), span(1742)),
        hit_span,
    );
    assert_invalid(
        &style,
        program(vec![item_node(Vec::new(), vec![hit], Vec::new(), 1750)]),
        hit_span,
    );

    let semantic_span = invalid_span(1760);
    let semantic = SpatialSemanticRecipeV2::new(
        field(SpatialShapeSymbolV2::new(0), span(1761)),
        SpatialFillRuleV2::NonZero,
        None,
        semantic_span,
    );
    assert_invalid(
        &style,
        program(vec![item_node(
            Vec::new(),
            Vec::new(),
            vec![semantic],
            1770,
        )]),
        semantic_span,
    );
}

#[test]
fn paint_hit_and_semantic_leaf_spans_precede_reference_checks() {
    let style = style();
    let paint_span = invalid_span(1780);
    let paint = SpatialPaintRecipeV2::CoveragePaint {
        coverage: coverage(0, 1781),
        brush: field(SpatialBrushSymbolV2::new(999), paint_span),
        opacity: field(255, span(1782)),
        clip: None,
        span: span(1783),
    };
    assert_invalid(
        &style,
        program(vec![item_node(vec![paint], Vec::new(), Vec::new(), 1790)]),
        paint_span,
    );

    let hit_span = invalid_span(1800);
    let hit = SpatialHitRecipeV2::new(
        coverage(0, 1801),
        None,
        field(SpatialBindingV2::Property(PropertyId::new(999)), hit_span),
        span(1802),
    );
    assert_invalid(
        &style,
        program(vec![item_node(Vec::new(), vec![hit], Vec::new(), 1810)]),
        hit_span,
    );

    let semantic_span = invalid_span(1820);
    let semantic = SpatialSemanticRecipeV2::new(
        field(SpatialShapeSymbolV2::new(999), semantic_span),
        SpatialFillRuleV2::NonZero,
        None,
        span(1821),
    );
    assert_invalid(
        &style,
        program(vec![item_node(
            Vec::new(),
            Vec::new(),
            vec![semantic],
            1830,
        )]),
        semantic_span,
    );
}
