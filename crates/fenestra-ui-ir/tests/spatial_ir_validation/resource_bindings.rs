use super::*;
use support::*;

#[test]
fn shape_fixed16_fields_are_validated_in_phase_five() {
    let style = style();
    let source = span(2000);
    let invalid = SpatialShapeDeclarationV2::new(
        field(SpatialShapeSymbolV2::new(0), span(2001)),
        SpatialShapeGeometryV2::Rect {
            origin: SpatialPointRecipeV2::new(
                field(SpatialBindingV2::Literal(MAX_FIXED + 1), source),
                lit_f(0, 2002),
            ),
            width: lit_f(1, 2003),
            height: lit_f(1, 2004),
        },
        span(2005),
    );
    let declaration = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(2010),
        vec![invalid],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        2009,
    );
    assert_error(
        &style,
        program(vec![declaration]),
        IrValidationErrorKind::SpatialFixed16OutOfRange,
        source,
    );
}

#[test]
fn brush_coordinates_and_colors_are_typed_in_phase_six() {
    let style = style();
    let fixed_span = span(2020);
    let invalid_fixed = SpatialBrushDeclarationV2::new(
        field(SpatialBrushSymbolV2::new(0), span(2021)),
        SpatialBrushContentV2::LinearGradient {
            start: SpatialPointRecipeV2::new(
                field(SpatialBindingV2::Literal(MAX_FIXED + 1), fixed_span),
                lit_f(0, 2022),
            ),
            end: point(1, 1, 2023),
            stops: Vec::new(),
        },
        span(2025),
    );
    let declaration = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(2030),
        Vec::new(),
        vec![invalid_fixed],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        2029,
    );
    assert_error(
        &style,
        program(vec![declaration]),
        IrValidationErrorKind::SpatialFixed16OutOfRange,
        fixed_span,
    );

    let color_span = span(2040);
    let invalid_color = SpatialBrushDeclarationV2::new(
        field(SpatialBrushSymbolV2::new(0), span(2041)),
        SpatialBrushContentV2::Solid {
            color: field(SpatialBindingV2::Property(SCALAR), color_span),
        },
        span(2042),
    );
    let declaration = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(2050),
        Vec::new(),
        vec![invalid_color],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        2049,
    );
    assert_error(
        &style,
        program(vec![declaration]),
        IrValidationErrorKind::SpatialPropertyTypeMismatch,
        color_span,
    );
}

#[test]
fn hit_input_policy_requires_an_input_policy_property() {
    let style = style();
    let source = span(2060);
    let hit = SpatialHitRecipeV2::new(
        coverage(0, 2061),
        None,
        field(SpatialBindingV2::Property(COLOR), source),
        span(2062),
    );
    let declaration = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(2070),
        vec![shape(0, 2080)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![hit],
        Vec::new(),
        2069,
    );
    assert_error(
        &style,
        program(vec![declaration]),
        IrValidationErrorKind::SpatialPropertyTypeMismatch,
        source,
    );
}

#[test]
fn phase_nine_checks_round_stroke_width_and_image_destination_bindings() {
    let style = style();
    let stroke_span = span(2090);
    let stroke = SpatialHitRecipeV2::new(
        SpatialCoverageRecipeV2::RoundStroke {
            shape: field(SpatialShapeSymbolV2::new(0), span(2091)),
            width: field(SpatialBindingV2::Property(COLOR), stroke_span),
        },
        None,
        field(SpatialBindingV2::Literal(InputPolicy::Accept), span(2092)),
        span(2093),
    );
    let declaration = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(2100),
        vec![shape(0, 2110)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![stroke],
        Vec::new(),
        2089,
    );
    assert_error(
        &style,
        program(vec![declaration]),
        IrValidationErrorKind::SpatialPropertyTypeMismatch,
        stroke_span,
    );

    let destination_span = span(2120);
    let paint = SpatialPaintRecipeV2::ImagePaint {
        image: field(SpatialImageSymbolV2::new(0), span(2121)),
        source_x: field(0, span(2122)),
        source_y: field(0, span(2123)),
        source_width: field(1, span(2124)),
        source_height: field(1, span(2125)),
        destination_origin: point(0, 0, 2126),
        destination_width: field(SpatialBindingV2::Property(COLOR), destination_span),
        destination_height: lit_f(1, 2128),
        opacity: field(255, span(2129)),
        clip: None,
        span: span(2130),
    };
    let declaration = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(2140),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![paint],
        Vec::new(),
        Vec::new(),
        2139,
    );
    let image = SpatialImageDeclarationV2::new(
        field(SpatialImageSymbolV2::new(0), span(2150)),
        field(1, span(2151)),
        field(1, span(2152)),
        field(4, span(2153)),
        vec![0, 0, 0, 0].into_boxed_slice(),
        span(2154),
    );
    let input = program_with(
        SUPPORTED_SPATIAL_FORMAT,
        NS,
        REV,
        viewport(2160),
        vec![declaration],
        vec![image],
        span(2166),
    );
    assert_error(
        &style,
        input,
        IrValidationErrorKind::SpatialPropertyTypeMismatch,
        destination_span,
    );
}
