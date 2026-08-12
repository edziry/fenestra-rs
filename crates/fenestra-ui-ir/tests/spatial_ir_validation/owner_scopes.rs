use super::*;
use support::*;

#[test]
fn property_bindings_resolve_through_the_containing_nodes_component() {
    let style = style();
    let source = span(2400);
    let dimensions = SpatialDimensionRecipeV2::new(
        field(SpatialBindingV2::Property(SCALAR), source),
        lit_i(1, 2401),
        lit_i(2, 2402),
    );
    let valid_dimensions =
        SpatialDimensionRecipeV2::new(lit_i(0, 2410), lit_i(1, 2411), lit_i(2, 2412));
    let declaration = node_with(
        0,
        STATIC_C,
        SpatialNodeParentV2::Viewport,
        SpatialPlacementRecipeV2::Layout(SpatialLayoutPlacementRecipeV2::new(
            dimensions,
            valid_dimensions,
            transform(2403),
        )),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        2399,
    );

    assert_error(
        &style,
        program(vec![declaration]),
        IrValidationErrorKind::SpatialPropertyTypeMismatch,
        source,
    );
}

#[test]
fn absent_properties_are_unknown_in_the_owner_component() {
    let style = style();
    let source = span(6050);
    let declaration = node_with(
        0,
        STATIC_C,
        SpatialNodeParentV2::Viewport,
        placement(6051),
        Vec::new(),
        vec![SpatialBrushDeclarationV2::new(
            field(SpatialBrushSymbolV2::new(0), span(6060)),
            SpatialBrushContentV2::Solid {
                color: field(SpatialBindingV2::Property(COLOR), source),
            },
            span(6061),
        )],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        6049,
    );
    assert_error(
        &style,
        program(vec![declaration]),
        IrValidationErrorKind::UnknownSpatialProperty,
        source,
    );
}

#[test]
fn shape_and_brush_symbols_are_strictly_owner_local() {
    let style = style();
    let shape_span = span(2410);
    let first = node_with(
        0,
        STATIC_A,
        SpatialNodeParentV2::Viewport,
        placement(2411),
        vec![shape(99, 2420)],
        vec![brush(99, 2430)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        2409,
    );
    let second = node_with(
        1,
        STATIC_B,
        SpatialNodeParentV2::Viewport,
        placement(2440),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![SpatialPaintRecipeV2::CoveragePaint {
            coverage: SpatialCoverageRecipeV2::Fill {
                shape: field(SpatialShapeSymbolV2::new(99), shape_span),
                rule: SpatialFillRuleV2::NonZero,
            },
            brush: field(SpatialBrushSymbolV2::new(99), span(2411)),
            opacity: field(255, span(2412)),
            clip: None,
            span: span(2413),
        }],
        Vec::new(),
        Vec::new(),
        2439,
    );
    assert_error(
        &style,
        program(vec![first.clone(), second]),
        IrValidationErrorKind::MissingSpatialShape,
        shape_span,
    );

    let brush_span = span(2450);
    let second = node_with(
        1,
        STATIC_B,
        SpatialNodeParentV2::Viewport,
        placement(2451),
        vec![shape(0, 2460)],
        Vec::new(),
        Vec::new(),
        vec![SpatialPaintRecipeV2::CoveragePaint {
            coverage: coverage(0, 2470),
            brush: field(SpatialBrushSymbolV2::new(99), brush_span),
            opacity: field(255, span(2471)),
            clip: None,
            span: span(2472),
        }],
        Vec::new(),
        Vec::new(),
        2449,
    );
    assert_error(
        &style,
        program(vec![first, second]),
        IrValidationErrorKind::MissingSpatialBrush,
        brush_span,
    );
}

#[test]
fn a_clips_shape_is_local_to_the_clip_owner() {
    let style = style();
    let source = span(2480);
    let nodes = vec![
        node_with(
            0,
            STATIC_A,
            SpatialNodeParentV2::Viewport,
            placement(2481),
            vec![shape(77, 2490)],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            2479,
        ),
        node_with(
            1,
            STATIC_B,
            SpatialNodeParentV2::Viewport,
            placement(2500),
            Vec::new(),
            Vec::new(),
            vec![SpatialClipDeclarationV2::new(
                field(SpatialClipSymbolV2::new(0), span(2501)),
                None,
                field(SpatialShapeSymbolV2::new(77), source),
                SpatialFillRuleV2::NonZero,
                span(2502),
            )],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            2499,
        ),
    ];
    assert_error(
        &style,
        program(nodes),
        IrValidationErrorKind::MissingSpatialShape,
        source,
    );
}

#[test]
fn local_brush_symbols_may_repeat_across_owners() {
    let style = style();
    let make_node = |symbol, target, index| {
        node_with(
            symbol,
            target,
            SpatialNodeParentV2::Viewport,
            placement(index),
            vec![shape(7, index + 20)],
            vec![brush(7, index + 30)],
            Vec::new(),
            vec![paint(7, 7, index + 40)],
            Vec::new(),
            Vec::new(),
            index - 1,
        )
    };
    validate(
        &style,
        program(vec![
            make_node(0, STATIC_A, 6080),
            make_node(1, STATIC_B, 6140),
        ]),
    )
    .expect("brush symbols are local to their owning node");
}
