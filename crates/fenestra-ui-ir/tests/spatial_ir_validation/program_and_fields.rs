use super::*;
use support::*;

#[test]
fn phase_one_checks_program_span_format_then_schema_identity() {
    let style = style();
    let bad_span = invalid_span(60);
    let invalid = program_with(
        SpatialFormatVersion::new(99),
        SchemaNamespace::new(999),
        REV,
        viewport(61),
        Vec::new(),
        Vec::new(),
        bad_span,
    );
    assert_error(
        &style,
        invalid,
        IrValidationErrorKind::InvalidSourceSpan,
        bad_span,
    );

    let unsupported = program_with(
        SpatialFormatVersion::new(99),
        SchemaNamespace::new(999),
        REV,
        viewport(62),
        Vec::new(),
        Vec::new(),
        span(63),
    );
    assert_error(
        &style,
        unsupported,
        IrValidationErrorKind::UnsupportedSpatialFormat,
        span(63),
    );

    for (namespace, revision) in [
        (SchemaNamespace::new(999), REV),
        (NS, SchemaRevision::new(999)),
    ] {
        let mismatch = program_with(
            SUPPORTED_SPATIAL_FORMAT,
            namespace,
            revision,
            viewport(64),
            Vec::new(),
            Vec::new(),
            span(65),
        );
        assert_error(
            &style,
            mismatch,
            IrValidationErrorKind::SchemaIdentityMismatch,
            span(65),
        );
    }
}

#[test]
fn phase_four_checks_viewport_before_node_bindings() {
    let style = style();
    let viewport_span = invalid_span(70);
    let bad_viewport = SpatialViewportContainerV2::new(
        SpatialAxisV2::Row,
        field(0, viewport_span),
        field(0, span(71)),
        field(0, span(72)),
        field(0, span(73)),
        field(0, span(74)),
        span(75),
    );
    let target_span = span(82);
    let bad_dimension = SpatialDimensionRecipeV2::new(
        field(
            SpatialBindingV2::Property(PropertyId::new(999)),
            target_span,
        ),
        lit_i(10, 83),
        lit_i(20, 84),
    );
    let bad_node = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        SpatialPlacementRecipeV2::Layout(SpatialLayoutPlacementRecipeV2::new(
            bad_dimension,
            bad_dimension,
            transform(85),
        )),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        80,
    );
    let both = program_with(
        SUPPORTED_SPATIAL_FORMAT,
        NS,
        REV,
        bad_viewport,
        vec![bad_node],
        Vec::new(),
        span(69),
    );
    assert_error(
        &style,
        both,
        IrValidationErrorKind::InvalidSourceSpan,
        viewport_span,
    );
}

#[test]
fn phase_four_reports_unknown_type_and_fixed_literal_at_the_leaf() {
    let style = style();
    let cases = [
        (
            SpatialBindingV2::Property(PropertyId::new(999)),
            IrValidationErrorKind::UnknownSpatialProperty,
        ),
        (
            SpatialBindingV2::Property(COLOR),
            IrValidationErrorKind::SpatialPropertyTypeMismatch,
        ),
    ];
    for (binding, expected) in cases {
        let source = span(90);
        let dimension =
            SpatialDimensionRecipeV2::new(field(binding, source), lit_i(10, 91), lit_i(20, 92));
        let valid_dimension =
            SpatialDimensionRecipeV2::new(lit_i(0, 116), lit_i(10, 117), lit_i(20, 118));
        let declaration = node_with(
            0,
            ROOT,
            SpatialNodeParentV2::Viewport,
            SpatialPlacementRecipeV2::Layout(SpatialLayoutPlacementRecipeV2::new(
                dimension,
                valid_dimension,
                transform(93),
            )),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            89,
        );
        assert_error(&style, program(vec![declaration]), expected, source);
    }

    let source = span(100);
    let bad_transform = SpatialTransformRecipeV2::new(
        field(SpatialBindingV2::Literal(MAX_FIXED + 1), source),
        lit_f(0, 101),
        lit_f(0, 102),
        lit_f(65_536, 103),
        lit_f(0, 104),
        lit_f(0, 105),
        point(0, 0, 106),
    );
    let declaration = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        SpatialPlacementRecipeV2::Layout(SpatialLayoutPlacementRecipeV2::new(
            SpatialDimensionRecipeV2::new(lit_i(0, 110), lit_i(1, 111), lit_i(2, 112)),
            SpatialDimensionRecipeV2::new(lit_i(0, 113), lit_i(1, 114), lit_i(2, 115)),
            bad_transform,
        )),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        109,
    );
    assert_error(
        &style,
        program(vec![declaration]),
        IrValidationErrorKind::SpatialFixed16OutOfRange,
        source,
    );
}

#[test]
fn every_binding_family_resolves_against_the_owner_template() {
    let style = style();
    let scalar = field(SpatialBindingV2::Property(SCALAR), span(120));
    let fixed = field(SpatialBindingV2::Property(SCALAR), span(121));
    let shape = SpatialShapeDeclarationV2::new(
        field(SpatialShapeSymbolV2::new(0), span(122)),
        SpatialShapeGeometryV2::Rect {
            origin: SpatialPointRecipeV2::new(fixed, fixed),
            width: fixed,
            height: fixed,
        },
        span(123),
    );
    let brush = SpatialBrushDeclarationV2::new(
        field(SpatialBrushSymbolV2::new(0), span(124)),
        SpatialBrushContentV2::Solid {
            color: field(SpatialBindingV2::Property(COLOR), span(125)),
        },
        span(126),
    );
    let hit = SpatialHitRecipeV2::new(
        coverage(0, 127),
        None,
        field(SpatialBindingV2::Property(POLICY), span(128)),
        span(129),
    );
    let dimensions = SpatialDimensionRecipeV2::new(scalar, scalar, scalar);
    let declaration = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        SpatialPlacementRecipeV2::Layout(SpatialLayoutPlacementRecipeV2::new(
            dimensions,
            dimensions,
            transform(130),
        )),
        vec![shape],
        vec![brush],
        Vec::new(),
        vec![paint(0, 0, 140)],
        vec![hit],
        Vec::new(),
        119,
    );
    validate(&style, program(vec![declaration])).expect("all binding families should validate");
}

#[test]
fn fixed16_literal_domain_is_inclusive_at_both_ends() {
    let style = style();
    let boundary_transform = SpatialTransformRecipeV2::new(
        lit_f(-MAX_FIXED, 160),
        lit_f(MAX_FIXED, 161),
        lit_f(0, 162),
        lit_f(65_536, 163),
        lit_f(-MAX_FIXED, 164),
        lit_f(MAX_FIXED, 165),
        point(-MAX_FIXED, MAX_FIXED, 166),
    );
    let dimensions = SpatialDimensionRecipeV2::new(lit_i(0, 168), lit_i(1, 169), lit_i(2, 170));
    let valid = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        SpatialPlacementRecipeV2::Layout(SpatialLayoutPlacementRecipeV2::new(
            dimensions,
            dimensions,
            boundary_transform,
        )),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        159,
    );
    validate(&style, program(vec![valid])).expect("both Fixed16 boundaries should validate");

    let source = span(180);
    let below = SpatialTransformRecipeV2::new(
        field(SpatialBindingV2::Literal(-MAX_FIXED - 1), source),
        lit_f(0, 181),
        lit_f(0, 182),
        lit_f(65_536, 183),
        lit_f(0, 184),
        lit_f(0, 185),
        point(0, 0, 186),
    );
    let invalid = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        SpatialPlacementRecipeV2::Layout(SpatialLayoutPlacementRecipeV2::new(
            dimensions, dimensions, below,
        )),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        179,
    );
    assert_error(
        &style,
        program(vec![invalid]),
        IrValidationErrorKind::SpatialFixed16OutOfRange,
        source,
    );
}

#[test]
fn synthetic_spans_are_valid_source_anchors() {
    let style = style();
    let synthetic = SourceSpan::synthetic();
    let viewport = SpatialViewportContainerV2::new(
        SpatialAxisV2::Row,
        field(0, synthetic),
        field(0, synthetic),
        field(0, synthetic),
        field(0, synthetic),
        field(0, synthetic),
        synthetic,
    );
    let input = program_with(
        SUPPORTED_SPATIAL_FORMAT,
        NS,
        REV,
        viewport,
        Vec::new(),
        Vec::new(),
        synthetic,
    );
    validate(&style, input).expect("synthetic spans are valid on records and fields");
}
