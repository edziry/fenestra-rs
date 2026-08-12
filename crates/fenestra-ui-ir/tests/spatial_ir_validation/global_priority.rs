use super::*;
use support::*;

fn phase_program(first: usize) -> (SpatialProgramV2, SpatialValidationLimitsV2) {
    let placement = if first <= 10 {
        free_placement(
            SpatialAnchorTargetRecipeV2::Node(field(SpatialNodeSymbolV2::new(999), span(3760))),
            3060,
        )
    } else {
        placement(3060)
    };
    let shapes = if first <= 5 {
        vec![shape(0, 3300), shape(0, 3310)]
    } else {
        vec![shape(0, 3300)]
    };
    let brushes = if first <= 6 {
        vec![brush(0, 3410), brush(0, 3420)]
    } else {
        vec![brush(0, 3410)]
    };
    let clips = if first <= 8 {
        vec![
            SpatialClipDeclarationV2::new(
                field(SpatialClipSymbolV2::new(0), span(3630)),
                None,
                field(SpatialShapeSymbolV2::new(0), span(3631)),
                SpatialFillRuleV2::NonZero,
                span(3632),
            ),
            SpatialClipDeclarationV2::new(
                field(SpatialClipSymbolV2::new(0), span(3640)),
                None,
                field(SpatialShapeSymbolV2::new(0), span(3641)),
                SpatialFillRuleV2::NonZero,
                span(3642),
            ),
        ]
    } else {
        Vec::new()
    };
    let paint = SpatialPaintRecipeV2::CoveragePaint {
        coverage: coverage(0, 3740),
        brush: field(
            SpatialBrushSymbolV2::new(if first <= 9 { 999 } else { 0 }),
            span(3750),
        ),
        opacity: field(255, span(3751)),
        clip: None,
        span: span(3752),
    };
    let root = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement,
        shapes,
        brushes,
        clips,
        vec![paint],
        Vec::new(),
        Vec::new(),
        3100,
    );
    let mut nodes = vec![root];
    if first <= 3 {
        nodes.push(node(0, STATIC_A, SpatialNodeParentV2::Viewport, 3120));
    }
    let image = |symbol, index| {
        SpatialImageDeclarationV2::new(
            field(SpatialImageSymbolV2::new(symbol), span(index)),
            field(1, span(index + 1)),
            field(1, span(index + 2)),
            field(1, span(index + 3)),
            vec![0].into_boxed_slice(),
            span(index + 4),
        )
    };
    let images = if first <= 7 {
        vec![image(0, 3520), image(0, 3530)]
    } else {
        Vec::new()
    };
    let viewport = if first <= 4 {
        SpatialViewportContainerV2::new(
            SpatialAxisV2::Row,
            field(0, invalid_span(3200)),
            field(0, span(3201)),
            field(0, span(3202)),
            field(0, span(3203)),
            field(0, span(3204)),
            span(3205),
        )
    } else {
        viewport(3200)
    };
    let input = program_with(
        if first <= 1 {
            SpatialFormatVersion::new(999)
        } else {
            SUPPORTED_SPATIAL_FORMAT
        },
        NS,
        REV,
        viewport,
        nodes,
        images,
        span(3000),
    );
    let mut limits = [64; 13];
    if first <= 2 {
        limits[0] = 0;
    }
    (input, SpatialValidationLimitsV2::new(limits))
}

#[test]
fn the_complete_ten_phase_priority_ladder_is_observable() {
    let style = style();
    let expected = [
        (IrValidationErrorKind::UnsupportedSpatialFormat, span(3000)),
        (
            IrValidationErrorKind::LimitExceeded(ValidationLimitKind::SpatialNodes),
            span(3108),
        ),
        (IrValidationErrorKind::DuplicateSpatialNode, span(3120)),
        (IrValidationErrorKind::InvalidSourceSpan, invalid_span(3200)),
        (IrValidationErrorKind::DuplicateSpatialShape, span(3310)),
        (IrValidationErrorKind::DuplicateSpatialBrush, span(3420)),
        (IrValidationErrorKind::DuplicateSpatialImage, span(3530)),
        (IrValidationErrorKind::DuplicateSpatialClip, span(3640)),
        (IrValidationErrorKind::MissingSpatialBrush, span(3750)),
        (
            IrValidationErrorKind::MissingSpatialAnchorTarget,
            span(3760),
        ),
    ];
    for (offset, (kind, source)) in expected.into_iter().enumerate() {
        let (program, limits) = phase_program(offset + 1);
        let error = validate_spatial(&style, program, limits).expect_err("phase should fail");
        assert_eq!(error.kind(), kind, "phase {}", offset + 1);
        assert_eq!(error.span(), source, "phase {}", offset + 1);
    }
}
