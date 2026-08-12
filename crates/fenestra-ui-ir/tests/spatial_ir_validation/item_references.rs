use super::*;
use support::*;

fn clip(symbol: u32, shape: u32, index: u32) -> SpatialClipDeclarationV2 {
    SpatialClipDeclarationV2::new(
        field(SpatialClipSymbolV2::new(symbol), span(index)),
        None,
        field(SpatialShapeSymbolV2::new(shape), span(index + 1)),
        SpatialFillRuleV2::NonZero,
        span(index + 2),
    )
}

fn address(
    owner: u32,
    owner_span: SourceSpan,
    clip: u32,
    clip_span: SourceSpan,
) -> SpatialClipAddressV2 {
    SpatialClipAddressV2::new(
        field(SpatialNodeSymbolV2::new(owner), owner_span),
        field(SpatialClipSymbolV2::new(clip), clip_span),
    )
}

#[test]
fn phase_nine_checks_all_paints_before_all_hits_then_all_semantics() {
    let style = style();
    let missing_brush_span = span(960);
    let first = node_with(
        0,
        STATIC_A,
        SpatialNodeParentV2::Viewport,
        placement(940),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![SpatialHitRecipeV2::new(
            coverage(999, 950),
            None,
            field(SpatialBindingV2::Literal(InputPolicy::Accept), span(951)),
            span(952),
        )],
        Vec::new(),
        939,
    );
    let second = node_with(
        1,
        STATIC_B,
        SpatialNodeParentV2::Viewport,
        placement(970),
        vec![shape(0, 980)],
        Vec::new(),
        Vec::new(),
        vec![SpatialPaintRecipeV2::CoveragePaint {
            coverage: coverage(0, 959),
            brush: field(SpatialBrushSymbolV2::new(999), missing_brush_span),
            opacity: field(255, span(961)),
            clip: None,
            span: span(962),
        }],
        Vec::new(),
        Vec::new(),
        969,
    );
    assert_error(
        &style,
        program(vec![first, second]),
        IrValidationErrorKind::MissingSpatialBrush,
        missing_brush_span,
    );
}

#[test]
fn phase_nine_checks_all_hits_before_all_semantics() {
    let style = style();
    let hit_span = span(6490);
    let first = node_with(
        0,
        STATIC_A,
        SpatialNodeParentV2::Viewport,
        placement(6491),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![SpatialSemanticRecipeV2::new(
            field(SpatialShapeSymbolV2::new(999), span(6492)),
            SpatialFillRuleV2::NonZero,
            None,
            span(6493),
        )],
        6489,
    );
    let second = node_with(
        1,
        STATIC_B,
        SpatialNodeParentV2::Viewport,
        placement(6500),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![SpatialHitRecipeV2::new(
            coverage(999, 6490),
            None,
            field(SpatialBindingV2::Literal(InputPolicy::Accept), span(6501)),
            span(6502),
        )],
        Vec::new(),
        6499,
    );
    assert_error(
        &style,
        program(vec![first, second]),
        IrValidationErrorKind::MissingSpatialShape,
        hit_span,
    );
}

#[test]
fn paint_hit_and_semantic_references_each_fail_independently() {
    let style = style();
    let image_span = span(2980);
    let image_paint = SpatialPaintRecipeV2::ImagePaint {
        image: field(SpatialImageSymbolV2::new(999), image_span),
        source_x: field(0, span(991)),
        source_y: field(0, span(992)),
        source_width: field(1, span(993)),
        source_height: field(1, span(994)),
        destination_origin: point(0, 0, 995),
        destination_width: lit_f(1, 997),
        destination_height: lit_f(1, 998),
        opacity: field(255, span(999)),
        clip: None,
        span: span(1000),
    };
    let paint_node = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(1001),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![image_paint],
        Vec::new(),
        Vec::new(),
        989,
    );
    assert_error(
        &style,
        program(vec![paint_node]),
        IrValidationErrorKind::MissingSpatialImage,
        image_span,
    );

    let hit_span = span(2981);
    let hit_node = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(1011),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![SpatialHitRecipeV2::new(
            coverage(999, 2981),
            None,
            field(SpatialBindingV2::Literal(InputPolicy::Accept), span(1011)),
            span(1012),
        )],
        Vec::new(),
        1009,
    );
    assert_error(
        &style,
        program(vec![hit_node]),
        IrValidationErrorKind::MissingSpatialShape,
        hit_span,
    );

    let semantic_span = span(2982);
    let semantic_node = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(1021),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![SpatialSemanticRecipeV2::new(
            field(SpatialShapeSymbolV2::new(999), semantic_span),
            SpatialFillRuleV2::NonZero,
            None,
            span(1022),
        )],
        1019,
    );
    assert_error(
        &style,
        program(vec![semantic_node]),
        IrValidationErrorKind::MissingSpatialShape,
        semantic_span,
    );
}

#[test]
fn terminal_item_clip_addresses_validate_existence_and_ancestry() {
    let style = style();
    let clip_span = span(1041);
    let missing = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(1030),
        vec![shape(0, 1031)],
        vec![brush(0, 1034)],
        Vec::new(),
        vec![SpatialPaintRecipeV2::CoveragePaint {
            coverage: coverage(0, 1037),
            brush: field(SpatialBrushSymbolV2::new(0), span(1038)),
            opacity: field(255, span(1039)),
            clip: Some(address(0, span(1040), 999, clip_span)),
            span: span(1042),
        }],
        Vec::new(),
        Vec::new(),
        1029,
    );
    assert_error(
        &style,
        program(vec![missing]),
        IrValidationErrorKind::MissingSpatialClip,
        clip_span,
    );

    let owner_span = span(1060);
    let siblings = vec![
        node_with(
            0,
            STATIC_A,
            SpatialNodeParentV2::Viewport,
            placement(1050),
            vec![shape(0, 1051)],
            Vec::new(),
            vec![clip(0, 0, 1054)],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            1049,
        ),
        node_with(
            1,
            STATIC_B,
            SpatialNodeParentV2::Viewport,
            placement(1070),
            vec![shape(0, 1071)],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![SpatialSemanticRecipeV2::new(
                field(SpatialShapeSymbolV2::new(0), span(1074)),
                SpatialFillRuleV2::EvenOdd,
                Some(address(0, owner_span, 0, span(1061))),
                span(1075),
            )],
            1069,
        ),
    ];
    assert_error(
        &style,
        program(siblings),
        IrValidationErrorKind::SpatialClipOwnerNotAncestor,
        owner_span,
    );
}

#[test]
fn ancestor_clip_addresses_are_valid_for_paint_hit_and_semantic_items() {
    let style = style();
    let ancestor = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(1080),
        vec![shape(0, 1081)],
        Vec::new(),
        vec![clip(0, 0, 1084)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        1079,
    );
    let terminal = address(0, span(1090), 0, span(1091));
    let child = node_with(
        1,
        STATIC_A,
        parent(0, 1092),
        placement(1093),
        vec![shape(0, 1094)],
        vec![brush(0, 1097)],
        Vec::new(),
        vec![SpatialPaintRecipeV2::CoveragePaint {
            coverage: coverage(0, 1100),
            brush: field(SpatialBrushSymbolV2::new(0), span(1101)),
            opacity: field(255, span(1102)),
            clip: Some(terminal),
            span: span(1103),
        }],
        vec![SpatialHitRecipeV2::new(
            coverage(0, 1104),
            Some(terminal),
            field(SpatialBindingV2::Literal(InputPolicy::Accept), span(1105)),
            span(1106),
        )],
        vec![SpatialSemanticRecipeV2::new(
            field(SpatialShapeSymbolV2::new(0), span(1107)),
            SpatialFillRuleV2::EvenOdd,
            Some(terminal),
            span(1108),
        )],
        1091,
    );
    validate(&style, program(vec![ancestor, child])).expect("ancestor clips should validate");
}
