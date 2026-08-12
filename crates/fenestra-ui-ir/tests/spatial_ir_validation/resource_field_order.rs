use super::*;
use support::*;

fn ordered_span(base: u32, position: usize, first_invalid: usize) -> SourceSpan {
    let index = base + u32::try_from(position).expect("small test position");
    if position >= first_invalid {
        invalid_span(index)
    } else {
        span(index)
    }
}

fn fixed(base: u32, position: usize, winner: usize) -> SpatialFieldV2<SpatialBindingV2<i64>> {
    field(
        SpatialBindingV2::Literal(1),
        ordered_span(base, position, winner),
    )
}

fn color(base: u32, position: usize, winner: usize) -> SpatialFieldV2<SpatialBindingV2<[u8; 4]>> {
    field(
        SpatialBindingV2::Literal([0, 0, 0, 255]),
        ordered_span(base, position, winner),
    )
}

fn brush_program(brush: SpatialBrushDeclarationV2, index: u32) -> SpatialProgramV2 {
    program(vec![node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(index),
        Vec::new(),
        vec![brush],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        index - 1,
    )])
}

#[test]
fn solid_and_gradient_fields_follow_record_symbol_then_content_order() {
    let style = style();
    for winner in 0..3 {
        let base = 4700;
        let brush = SpatialBrushDeclarationV2::new(
            field(SpatialBrushSymbolV2::new(0), ordered_span(base, 1, winner)),
            SpatialBrushContentV2::Solid {
                color: color(base, 2, winner),
            },
            ordered_span(base, 0, winner),
        );
        assert_error(
            &style,
            brush_program(brush, 4710),
            IrValidationErrorKind::InvalidSourceSpan,
            invalid_span(base + u32::try_from(winner).unwrap()),
        );
    }

    for winner in 0..9 {
        let base = 4720;
        let brush = SpatialBrushDeclarationV2::new(
            field(SpatialBrushSymbolV2::new(0), ordered_span(base, 1, winner)),
            SpatialBrushContentV2::LinearGradient {
                start: SpatialPointRecipeV2::new(fixed(base, 2, winner), fixed(base, 3, winner)),
                end: SpatialPointRecipeV2::new(fixed(base, 4, winner), fixed(base, 5, winner)),
                stops: vec![SpatialGradientStopV2::new(
                    field(0, ordered_span(base, 7, winner)),
                    color(base, 8, winner),
                    ordered_span(base, 6, winner),
                )],
            },
            ordered_span(base, 0, winner),
        );
        assert_error(
            &style,
            brush_program(brush, 4740),
            IrValidationErrorKind::InvalidSourceSpan,
            invalid_span(base + u32::try_from(winner).unwrap()),
        );
    }
}

#[test]
fn image_record_symbol_and_literal_fields_follow_stored_order() {
    let style = style();
    let base = 4750;
    for winner in 0..5 {
        let image = SpatialImageDeclarationV2::new(
            field(SpatialImageSymbolV2::new(0), ordered_span(base, 1, winner)),
            field(1, ordered_span(base, 2, winner)),
            field(1, ordered_span(base, 3, winner)),
            field(4, ordered_span(base, 4, winner)),
            vec![0, 0, 0, 0].into_boxed_slice(),
            ordered_span(base, 0, winner),
        );
        let input = program_with(
            SUPPORTED_SPATIAL_FORMAT,
            NS,
            REV,
            viewport(4760),
            Vec::new(),
            vec![image],
            span(4766),
        );
        assert_error(
            &style,
            input,
            IrValidationErrorKind::InvalidSourceSpan,
            invalid_span(base + u32::try_from(winner).unwrap()),
        );
    }
}

#[test]
fn clip_record_symbol_parent_address_and_shape_follow_phase_eight_order() {
    let style = style();
    let base = 4770;
    for winner in 0..5 {
        let owner = node_with(
            0,
            ROOT,
            SpatialNodeParentV2::Viewport,
            placement(4780),
            vec![shape(0, 4790)],
            Vec::new(),
            vec![SpatialClipDeclarationV2::new(
                field(SpatialClipSymbolV2::new(0), span(4800)),
                None,
                field(SpatialShapeSymbolV2::new(0), span(4801)),
                SpatialFillRuleV2::NonZero,
                span(4802),
            )],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            4779,
        );
        let child_clip = SpatialClipDeclarationV2::new(
            field(SpatialClipSymbolV2::new(0), ordered_span(base, 1, winner)),
            Some(SpatialClipAddressV2::new(
                field(SpatialNodeSymbolV2::new(0), ordered_span(base, 2, winner)),
                field(SpatialClipSymbolV2::new(0), ordered_span(base, 3, winner)),
            )),
            field(SpatialShapeSymbolV2::new(0), ordered_span(base, 4, winner)),
            SpatialFillRuleV2::EvenOdd,
            ordered_span(base, 0, winner),
        );
        let child = node_with(
            1,
            STATIC_A,
            parent(0, 4810),
            placement(4811),
            vec![shape(0, 4820)],
            Vec::new(),
            vec![child_clip],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            4809,
        );
        assert_error(
            &style,
            program(vec![owner, child]),
            IrValidationErrorKind::InvalidSourceSpan,
            invalid_span(base + u32::try_from(winner).unwrap()),
        );
    }
}
