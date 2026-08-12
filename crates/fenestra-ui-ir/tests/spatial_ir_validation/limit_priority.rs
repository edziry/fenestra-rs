use super::*;
use support::*;

const KINDS: [ValidationLimitKind; 13] = [
    ValidationLimitKind::SpatialNodes,
    ValidationLimitKind::SpatialShapes,
    ValidationLimitKind::SpatialBrushes,
    ValidationLimitKind::SpatialClips,
    ValidationLimitKind::SpatialPaintItems,
    ValidationLimitKind::SpatialHitItems,
    ValidationLimitKind::SpatialSemanticItems,
    ValidationLimitKind::SpatialPaths,
    ValidationLimitKind::SpatialPathVerbs,
    ValidationLimitKind::SpatialPolygonPoints,
    ValidationLimitKind::SpatialGradientStops,
    ValidationLimitKind::SpatialImages,
    ValidationLimitKind::SpatialImageBytes,
];

// Two paths plus one polygon necessarily make three total shapes. Every other
// category has exactly two authored records in the composite fixture.
const OBSERVED: [usize; 13] = [2, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2];

fn composite_program() -> SpatialProgramV2 {
    let path = SpatialShapeDeclarationV2::new(
        field(SpatialShapeSymbolV2::new(0), span(2200)),
        SpatialShapeGeometryV2::Path {
            verbs: vec![
                SpatialPathVerbRecipeV2::MoveTo {
                    to: point(0, 0, 2201),
                    span: span(2203),
                },
                SpatialPathVerbRecipeV2::LineTo {
                    to: point(1, 1, 2204),
                    span: span(2206),
                },
            ],
        },
        span(2207),
    );
    let polygon = SpatialShapeDeclarationV2::new(
        field(SpatialShapeSymbolV2::new(1), span(2210)),
        SpatialShapeGeometryV2::Polygon {
            points: vec![
                SpatialPolygonPointV2::new(point(0, 0, 2211), span(2213)),
                SpatialPolygonPointV2::new(point(1, 1, 2214), span(2216)),
            ],
        },
        span(2217),
    );
    let second_path = SpatialShapeDeclarationV2::new(
        field(SpatialShapeSymbolV2::new(2), span(2220)),
        SpatialShapeGeometryV2::Path { verbs: Vec::new() },
        span(2221),
    );
    let gradient = SpatialBrushDeclarationV2::new(
        field(SpatialBrushSymbolV2::new(0), span(2230)),
        SpatialBrushContentV2::LinearGradient {
            start: point(0, 0, 2231),
            end: point(1, 1, 2233),
            stops: vec![
                SpatialGradientStopV2::new(
                    field(0, span(2235)),
                    field(SpatialBindingV2::Literal([0, 0, 0, 0]), span(2236)),
                    span(2237),
                ),
                SpatialGradientStopV2::new(
                    field(u16::MAX, span(2238)),
                    field(SpatialBindingV2::Literal([0, 0, 0, 0]), span(2239)),
                    span(2240),
                ),
            ],
        },
        span(2241),
    );
    let solid = brush(1, 2242);
    let clips = [0_u32, 1].map(|symbol| {
        SpatialClipDeclarationV2::new(
            field(SpatialClipSymbolV2::new(symbol), span(2250 + symbol * 3)),
            None,
            field(SpatialShapeSymbolV2::new(0), span(2251 + symbol * 3)),
            SpatialFillRuleV2::NonZero,
            span(2252 + symbol * 3),
        )
    });
    let hits = [0_u32, 1].map(|offset| {
        SpatialHitRecipeV2::new(
            coverage(0, 2260 + offset * 3),
            None,
            field(
                SpatialBindingV2::Literal(InputPolicy::Accept),
                span(2261 + offset * 3),
            ),
            span(2262 + offset * 3),
        )
    });
    let semantics = [0_u32, 1].map(|offset| {
        SpatialSemanticRecipeV2::new(
            field(SpatialShapeSymbolV2::new(0), span(2270 + offset * 2)),
            SpatialFillRuleV2::NonZero,
            None,
            span(2271 + offset * 2),
        )
    });
    let root = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(2280),
        vec![path, second_path, polygon],
        vec![gradient, solid],
        clips.into(),
        vec![paint(0, 0, 2290), paint(0, 0, 2294)],
        hits.into(),
        semantics.into(),
        2279,
    );
    let second = node(1, STATIC_A, SpatialNodeParentV2::Viewport, 2300);
    let images = [0_u32, 1].map(|symbol| {
        SpatialImageDeclarationV2::new(
            field(SpatialImageSymbolV2::new(symbol), span(2310 + symbol * 5)),
            field(1, span(2311 + symbol * 5)),
            field(1, span(2312 + symbol * 5)),
            field(1, span(2313 + symbol * 5)),
            if symbol == 0 {
                vec![0, 0].into_boxed_slice()
            } else {
                Vec::new().into_boxed_slice()
            },
            span(2314 + symbol * 5),
        )
    });
    program_with(
        SUPPORTED_SPATIAL_FORMAT,
        NS,
        REV,
        viewport(2320),
        vec![root, second],
        images.into(),
        span(2330),
    )
}

#[test]
fn every_limit_position_wins_over_all_later_overflows() {
    let style = style();
    let program = composite_program();
    let crossing_spans = [
        span(2308),
        span(2217),
        span(2244),
        span(2255),
        span(2297),
        span(2265),
        span(2273),
        span(2221),
        span(2206),
        span(2216),
        span(2240),
        span(2319),
        span(2314),
    ];
    for winner in 0..KINDS.len() {
        let mut limits = OBSERVED;
        for (limit, observed) in limits[winner..].iter_mut().zip(&OBSERVED[winner..]) {
            *limit = observed - 1;
        }
        let error = validate_spatial(
            &style,
            program.clone(),
            SpatialValidationLimitsV2::new(limits),
        )
        .expect_err("the first configured overflow should win");
        assert_eq!(
            error.kind(),
            IrValidationErrorKind::LimitExceeded(KINDS[winner])
        );
        assert_eq!(error.span(), crossing_spans[winner]);
    }
}

#[test]
fn every_limit_reports_the_first_record_that_crosses_zero() {
    let style = style();
    let program = composite_program();
    let first_spans = [
        span(2287),
        span(2207),
        span(2241),
        span(2252),
        span(2293),
        span(2262),
        span(2271),
        span(2207),
        span(2203),
        span(2213),
        span(2237),
        span(2314),
        span(2314),
    ];
    for position in 0..KINDS.len() {
        let mut limits = [64; 13];
        limits[position] = 0;
        let error = validate_spatial(
            &style,
            program.clone(),
            SpatialValidationLimitsV2::new(limits),
        )
        .expect_err("the first authored record crosses a zero maximum");
        assert_eq!(
            error.kind(),
            IrValidationErrorKind::LimitExceeded(KINDS[position])
        );
        assert_eq!(error.span(), first_spans[position]);
    }
}

#[test]
fn static_counts_sum_across_owners_and_exclude_live_key_multiplicity() {
    let style = style();
    let declarations = vec![
        node_with(
            0,
            ROOT,
            SpatialNodeParentV2::Viewport,
            placement(2340),
            vec![shape(0, 2350)],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            2339,
        ),
        node_with(
            1,
            OUTER,
            parent(0, 2360),
            placement(2361),
            vec![shape(0, 2370)],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            2359,
        ),
    ];
    let input = program(declarations);
    let mut limits = [64; 13];
    limits[0] = 2;
    limits[1] = 2;
    validate_spatial(
        &style,
        input.clone(),
        SpatialValidationLimitsV2::new(limits),
    )
    .expect("three live repeat keys must not multiply two authored node/shape counts");

    limits[1] = 1;
    let error = validate_spatial(&style, input, SpatialValidationLimitsV2::new(limits))
        .expect_err("shape totals must sum across authored owners");
    assert_eq!(
        error.kind(),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::SpatialShapes)
    );
    assert_eq!(error.span(), span(2375));

    let images = [0_u32, 1].map(|symbol| {
        SpatialImageDeclarationV2::new(
            field(SpatialImageSymbolV2::new(symbol), span(2380 + symbol * 5)),
            field(1, span(2381 + symbol * 5)),
            field(1, span(2382 + symbol * 5)),
            field(1, span(2383 + symbol * 5)),
            vec![0].into_boxed_slice(),
            span(2384 + symbol * 5),
        )
    });
    let image_input = program_with(
        SUPPORTED_SPATIAL_FORMAT,
        NS,
        REV,
        viewport(2390),
        Vec::new(),
        images.into(),
        span(2396),
    );
    let mut limits = [64; 13];
    limits[12] = 1;
    let error = validate_spatial(&style, image_input, SpatialValidationLimitsV2::new(limits))
        .expect_err("image bytes must sum across declarations");
    assert_eq!(
        error.kind(),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::SpatialImageBytes)
    );
    assert_eq!(error.span(), span(2389));
}

#[test]
fn static_item_counts_do_not_filter_inactive_literal_values() {
    let style = style();
    let declaration = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(6510),
        vec![shape(0, 6520)],
        vec![brush(0, 6530)],
        Vec::new(),
        vec![SpatialPaintRecipeV2::CoveragePaint {
            coverage: coverage(0, 6540),
            brush: field(SpatialBrushSymbolV2::new(0), span(6541)),
            opacity: field(0, span(6542)),
            clip: None,
            span: span(6543),
        }],
        vec![SpatialHitRecipeV2::new(
            coverage(0, 6550),
            None,
            field(SpatialBindingV2::Literal(InputPolicy::Ignore), span(6551)),
            span(6552),
        )],
        Vec::new(),
        6509,
    );
    let input = program(vec![declaration]);
    let mut limits = [64; 13];
    limits[4] = 0;
    limits[5] = 0;
    let error = validate_spatial(
        &style,
        input.clone(),
        SpatialValidationLimitsV2::new(limits),
    )
    .expect_err("zero-opacity paints remain authored paint records");
    assert_eq!(
        error.kind(),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::SpatialPaintItems)
    );
    assert_eq!(error.span(), span(6543));

    limits[4] = 1;
    let error = validate_spatial(&style, input, SpatialValidationLimitsV2::new(limits))
        .expect_err("ignored hits remain authored hit records");
    assert_eq!(
        error.kind(),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::SpatialHitItems)
    );
    assert_eq!(error.span(), span(6552));
}
