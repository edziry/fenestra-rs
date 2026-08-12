use super::*;
use support::*;

fn terminal(owner: u32, owner_at: u32, clip: u32, clip_at: u32) -> SpatialClipAddressV2 {
    SpatialClipAddressV2::new(
        field(SpatialNodeSymbolV2::new(owner), span(owner_at)),
        field(SpatialClipSymbolV2::new(clip), span(clip_at)),
    )
}

fn local_clip(index: u32) -> SpatialClipDeclarationV2 {
    SpatialClipDeclarationV2::new(
        field(SpatialClipSymbolV2::new(0), span(index)),
        None,
        field(SpatialShapeSymbolV2::new(0), span(index + 1)),
        SpatialFillRuleV2::NonZero,
        span(index + 2),
    )
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
        vec![shape(0, index + 20)],
        vec![brush(0, index + 30)],
        vec![local_clip(index + 40)],
        paints,
        hits,
        semantics,
        index - 1,
    )
}

fn image(index: u32) -> SpatialImageDeclarationV2 {
    SpatialImageDeclarationV2::new(
        field(SpatialImageSymbolV2::new(0), span(index)),
        field(1, span(index + 1)),
        field(1, span(index + 2)),
        field(4, span(index + 3)),
        vec![0, 0, 0, 0].into_boxed_slice(),
        span(index + 4),
    )
}

fn image_program(
    paint: SpatialPaintRecipeV2,
    declaration_index: u32,
    include_image: bool,
) -> SpatialProgramV2 {
    program_with(
        SUPPORTED_SPATIAL_FORMAT,
        NS,
        REV,
        viewport(declaration_index + 60),
        vec![item_node(
            vec![paint],
            Vec::new(),
            Vec::new(),
            declaration_index,
        )],
        if include_image {
            vec![image(declaration_index + 50)]
        } else {
            Vec::new()
        },
        span(declaration_index + 66),
    )
}

#[test]
fn coverage_paint_semantics_follow_shape_width_brush_then_clip() {
    let style = style();
    let missing_shape = span(5700);
    let paint = SpatialPaintRecipeV2::CoveragePaint {
        coverage: SpatialCoverageRecipeV2::RoundStroke {
            shape: field(SpatialShapeSymbolV2::new(999), missing_shape),
            width: field(SpatialBindingV2::Property(COLOR), span(5701)),
        },
        brush: field(SpatialBrushSymbolV2::new(999), span(5702)),
        opacity: field(255, span(5703)),
        clip: Some(terminal(999, 5704, 999, 5705)),
        span: span(5699),
    };
    assert_error(
        &style,
        program(vec![item_node(vec![paint], Vec::new(), Vec::new(), 5720)]),
        IrValidationErrorKind::MissingSpatialShape,
        missing_shape,
    );

    let width = span(5730);
    let paint = SpatialPaintRecipeV2::CoveragePaint {
        coverage: SpatialCoverageRecipeV2::RoundStroke {
            shape: field(SpatialShapeSymbolV2::new(0), span(5731)),
            width: field(SpatialBindingV2::Property(COLOR), width),
        },
        brush: field(SpatialBrushSymbolV2::new(999), span(5732)),
        opacity: field(255, span(5733)),
        clip: Some(terminal(999, 5734, 999, 5735)),
        span: span(5729),
    };
    assert_error(
        &style,
        program(vec![item_node(vec![paint], Vec::new(), Vec::new(), 5750)]),
        IrValidationErrorKind::SpatialPropertyTypeMismatch,
        width,
    );

    let missing_brush = span(5760);
    let paint = SpatialPaintRecipeV2::CoveragePaint {
        coverage: coverage(0, 5761),
        brush: field(SpatialBrushSymbolV2::new(999), missing_brush),
        opacity: field(255, span(5762)),
        clip: Some(terminal(999, 5763, 999, 5764)),
        span: span(5759),
    };
    assert_error(
        &style,
        program(vec![item_node(vec![paint], Vec::new(), Vec::new(), 5780)]),
        IrValidationErrorKind::MissingSpatialBrush,
        missing_brush,
    );

    for (address, kind, expected) in [
        (
            terminal(999, 5790, 999, 5791),
            IrValidationErrorKind::MissingSpatialClipOwner,
            span(5790),
        ),
        (
            terminal(0, 5800, 999, 5801),
            IrValidationErrorKind::MissingSpatialClip,
            span(5801),
        ),
    ] {
        let paint = SpatialPaintRecipeV2::CoveragePaint {
            coverage: coverage(0, 5802),
            brush: field(SpatialBrushSymbolV2::new(0), span(5803)),
            opacity: field(255, span(5804)),
            clip: Some(address),
            span: span(5805),
        };
        assert_error(
            &style,
            program(vec![item_node(vec![paint], Vec::new(), Vec::new(), 5820)]),
            kind,
            expected,
        );
    }
}

#[test]
fn image_paint_semantics_check_image_before_destination_and_clip() {
    let style = style();
    let make_paint = |image_symbol, image_at, destination| SpatialPaintRecipeV2::ImagePaint {
        image: field(SpatialImageSymbolV2::new(image_symbol), span(image_at)),
        source_x: field(0, span(5840)),
        source_y: field(0, span(5841)),
        source_width: field(1, span(5842)),
        source_height: field(1, span(5843)),
        destination_origin: point(0, 0, 5844),
        destination_width: destination,
        destination_height: lit_f(1, 5847),
        opacity: field(255, span(5848)),
        clip: Some(terminal(999, 5849, 999, 5850)),
        span: span(5839),
    };
    let missing_image = span(5851);
    let destination = field(SpatialBindingV2::Property(COLOR), span(5852));
    assert_error(
        &style,
        image_program(make_paint(999, 5851, destination), 5860, false),
        IrValidationErrorKind::MissingSpatialImage,
        missing_image,
    );
    assert_error(
        &style,
        image_program(make_paint(0, 5853, destination), 5880, true),
        IrValidationErrorKind::SpatialPropertyTypeMismatch,
        span(5852),
    );
}

#[test]
fn hit_and_semantic_semantics_follow_their_stored_reference_order() {
    let style = style();
    let missing_shape = span(5900);
    let hit = SpatialHitRecipeV2::new(
        SpatialCoverageRecipeV2::RoundStroke {
            shape: field(SpatialShapeSymbolV2::new(999), missing_shape),
            width: field(SpatialBindingV2::Property(COLOR), span(5901)),
        },
        Some(terminal(999, 5902, 999, 5903)),
        field(SpatialBindingV2::Property(COLOR), span(5904)),
        span(5899),
    );
    assert_error(
        &style,
        program(vec![item_node(Vec::new(), vec![hit], Vec::new(), 5920)]),
        IrValidationErrorKind::MissingSpatialShape,
        missing_shape,
    );

    let width = span(5930);
    let hit = SpatialHitRecipeV2::new(
        SpatialCoverageRecipeV2::RoundStroke {
            shape: field(SpatialShapeSymbolV2::new(0), span(5931)),
            width: field(SpatialBindingV2::Property(COLOR), width),
        },
        Some(terminal(999, 5932, 999, 5933)),
        field(SpatialBindingV2::Property(COLOR), span(5934)),
        span(5929),
    );
    assert_error(
        &style,
        program(vec![item_node(Vec::new(), vec![hit], Vec::new(), 5950)]),
        IrValidationErrorKind::SpatialPropertyTypeMismatch,
        width,
    );

    let owner = span(5960);
    let hit = SpatialHitRecipeV2::new(
        coverage(0, 5961),
        Some(terminal(999, 5960, 999, 5962)),
        field(SpatialBindingV2::Property(COLOR), span(5963)),
        span(5959),
    );
    assert_error(
        &style,
        program(vec![item_node(Vec::new(), vec![hit], Vec::new(), 5980)]),
        IrValidationErrorKind::MissingSpatialClipOwner,
        owner,
    );

    let input = span(5990);
    let hit = SpatialHitRecipeV2::new(
        coverage(0, 5991),
        Some(terminal(0, 5992, 0, 5993)),
        field(SpatialBindingV2::Property(COLOR), input),
        span(5989),
    );
    assert_error(
        &style,
        program(vec![item_node(Vec::new(), vec![hit], Vec::new(), 6010)]),
        IrValidationErrorKind::SpatialPropertyTypeMismatch,
        input,
    );

    let semantic_shape = span(6020);
    let semantic = SpatialSemanticRecipeV2::new(
        field(SpatialShapeSymbolV2::new(999), semantic_shape),
        SpatialFillRuleV2::NonZero,
        Some(terminal(999, 6021, 999, 6022)),
        span(6019),
    );
    assert_error(
        &style,
        program(vec![item_node(
            Vec::new(),
            Vec::new(),
            vec![semantic],
            6040,
        )]),
        IrValidationErrorKind::MissingSpatialShape,
        semantic_shape,
    );
}
