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

fn address(base: u32, owner_position: usize, winner: usize) -> SpatialClipAddressV2 {
    SpatialClipAddressV2::new(
        field(
            SpatialNodeSymbolV2::new(0),
            ordered_span(base, owner_position, winner),
        ),
        field(
            SpatialClipSymbolV2::new(0),
            ordered_span(base, owner_position + 1, winner),
        ),
    )
}

fn item_program(
    paints: Vec<SpatialPaintRecipeV2>,
    hits: Vec<SpatialHitRecipeV2>,
    semantics: Vec<SpatialSemanticRecipeV2>,
    image: bool,
    index: u32,
) -> SpatialProgramV2 {
    let declaration = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(index),
        vec![shape(0, index + 10)],
        vec![brush(0, index + 20)],
        vec![SpatialClipDeclarationV2::new(
            field(SpatialClipSymbolV2::new(0), span(index + 30)),
            None,
            field(SpatialShapeSymbolV2::new(0), span(index + 31)),
            SpatialFillRuleV2::NonZero,
            span(index + 32),
        )],
        paints,
        hits,
        semantics,
        index - 1,
    );
    let images = if image {
        vec![SpatialImageDeclarationV2::new(
            field(SpatialImageSymbolV2::new(0), span(index + 40)),
            field(1, span(index + 41)),
            field(1, span(index + 42)),
            field(4, span(index + 43)),
            vec![0, 0, 0, 0].into_boxed_slice(),
            span(index + 44),
        )]
    } else {
        Vec::new()
    };
    program_with(
        SUPPORTED_SPATIAL_FORMAT,
        NS,
        REV,
        viewport(index + 50),
        vec![declaration],
        images,
        span(index + 56),
    )
}

#[test]
fn coverage_paint_fields_follow_record_coverage_brush_opacity_clip_order() {
    let style = style();
    let base = 4900;
    for winner in 0..7 {
        let paint = SpatialPaintRecipeV2::CoveragePaint {
            coverage: SpatialCoverageRecipeV2::RoundStroke {
                shape: field(SpatialShapeSymbolV2::new(0), ordered_span(base, 1, winner)),
                width: fixed(base, 2, winner),
            },
            brush: field(SpatialBrushSymbolV2::new(0), ordered_span(base, 3, winner)),
            opacity: field(255, ordered_span(base, 4, winner)),
            clip: Some(address(base, 5, winner)),
            span: ordered_span(base, 0, winner),
        };
        assert_error(
            &style,
            item_program(vec![paint], Vec::new(), Vec::new(), false, 4920),
            IrValidationErrorKind::InvalidSourceSpan,
            invalid_span(base + u32::try_from(winner).unwrap()),
        );
    }
}

#[test]
fn image_paint_fields_follow_the_exact_stored_order() {
    let style = style();
    let base = 5000;
    for winner in 0..13 {
        let paint = SpatialPaintRecipeV2::ImagePaint {
            image: field(SpatialImageSymbolV2::new(0), ordered_span(base, 1, winner)),
            source_x: field(0, ordered_span(base, 2, winner)),
            source_y: field(0, ordered_span(base, 3, winner)),
            source_width: field(1, ordered_span(base, 4, winner)),
            source_height: field(1, ordered_span(base, 5, winner)),
            destination_origin: SpatialPointRecipeV2::new(
                fixed(base, 6, winner),
                fixed(base, 7, winner),
            ),
            destination_width: fixed(base, 8, winner),
            destination_height: fixed(base, 9, winner),
            opacity: field(255, ordered_span(base, 10, winner)),
            clip: Some(address(base, 11, winner)),
            span: ordered_span(base, 0, winner),
        };
        assert_error(
            &style,
            item_program(vec![paint], Vec::new(), Vec::new(), true, 5020),
            IrValidationErrorKind::InvalidSourceSpan,
            invalid_span(base + u32::try_from(winner).unwrap()),
        );
    }
}

#[test]
fn hit_fields_follow_record_coverage_clip_then_input_order() {
    let style = style();
    let base = 5100;
    for winner in 0..6 {
        let hit = SpatialHitRecipeV2::new(
            SpatialCoverageRecipeV2::RoundStroke {
                shape: field(SpatialShapeSymbolV2::new(0), ordered_span(base, 1, winner)),
                width: fixed(base, 2, winner),
            },
            Some(address(base, 3, winner)),
            field(
                SpatialBindingV2::Literal(InputPolicy::Accept),
                ordered_span(base, 5, winner),
            ),
            ordered_span(base, 0, winner),
        );
        assert_error(
            &style,
            item_program(Vec::new(), vec![hit], Vec::new(), false, 5120),
            IrValidationErrorKind::InvalidSourceSpan,
            invalid_span(base + u32::try_from(winner).unwrap()),
        );
    }
}

#[test]
fn semantic_fields_follow_record_shape_then_clip_order() {
    let style = style();
    let base = 5200;
    for winner in 0..4 {
        let semantic = SpatialSemanticRecipeV2::new(
            field(SpatialShapeSymbolV2::new(0), ordered_span(base, 1, winner)),
            SpatialFillRuleV2::NonZero,
            Some(address(base, 2, winner)),
            ordered_span(base, 0, winner),
        );
        assert_error(
            &style,
            item_program(Vec::new(), Vec::new(), vec![semantic], false, 5220),
            IrValidationErrorKind::InvalidSourceSpan,
            invalid_span(base + u32::try_from(winner).unwrap()),
        );
    }
}
