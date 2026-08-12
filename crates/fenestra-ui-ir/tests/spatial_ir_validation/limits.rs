use super::*;
use support::*;

struct LimitCase {
    program: SpatialProgramV2,
    count: usize,
    kind: ValidationLimitKind,
    crossing_span: SourceSpan,
}

fn hit(index: u32) -> SpatialHitRecipeV2 {
    SpatialHitRecipeV2::new(
        coverage(0, index),
        None,
        field(
            SpatialBindingV2::Literal(InputPolicy::Accept),
            span(index + 1),
        ),
        span(index + 2),
    )
}

fn semantic(index: u32) -> SpatialSemanticRecipeV2 {
    SpatialSemanticRecipeV2::new(
        field(SpatialShapeSymbolV2::new(0), span(index)),
        SpatialFillRuleV2::NonZero,
        None,
        span(index + 1),
    )
}

fn image(symbol: u32, bytes: Vec<u8>, index: u32) -> SpatialImageDeclarationV2 {
    SpatialImageDeclarationV2::new(
        field(SpatialImageSymbolV2::new(symbol), span(index)),
        field(1, span(index + 1)),
        field(1, span(index + 2)),
        field(4, span(index + 3)),
        bytes.into_boxed_slice(),
        span(index + 4),
    )
}

fn cases() -> Vec<LimitCase> {
    let nodes = program(vec![
        node(0, ROOT, SpatialNodeParentV2::Viewport, 1100),
        node(1, STATIC_A, SpatialNodeParentV2::Viewport, 1110),
    ]);
    let shapes = program(vec![node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(1120),
        vec![shape(0, 1130), shape(1, 1140)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        1119,
    )]);
    let brushes = program(vec![node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(1150),
        Vec::new(),
        vec![brush(0, 1160), brush(1, 1170)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        1149,
    )]);
    let clips = program(vec![node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(1180),
        vec![shape(0, 1190)],
        Vec::new(),
        vec![
            SpatialClipDeclarationV2::new(
                field(SpatialClipSymbolV2::new(0), span(1200)),
                None,
                field(SpatialShapeSymbolV2::new(0), span(1201)),
                SpatialFillRuleV2::NonZero,
                span(1202),
            ),
            SpatialClipDeclarationV2::new(
                field(SpatialClipSymbolV2::new(1), span(1210)),
                None,
                field(SpatialShapeSymbolV2::new(0), span(1211)),
                SpatialFillRuleV2::NonZero,
                span(1212),
            ),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        1179,
    )]);
    let paints = program(vec![node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(1220),
        vec![shape(0, 1230)],
        vec![brush(0, 1240)],
        Vec::new(),
        vec![paint(0, 0, 1250), paint(0, 0, 1260)],
        Vec::new(),
        Vec::new(),
        1219,
    )]);
    let hits = program(vec![node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(1270),
        vec![shape(0, 1280)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![hit(1290), hit(1300)],
        Vec::new(),
        1269,
    )]);
    let semantics = program(vec![node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(1310),
        vec![shape(0, 1320)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![semantic(1330), semantic(1340)],
        1309,
    )]);
    let paths = program(vec![node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(1350),
        vec![
            SpatialShapeDeclarationV2::new(
                field(SpatialShapeSymbolV2::new(0), span(1360)),
                SpatialShapeGeometryV2::Path { verbs: Vec::new() },
                span(1361),
            ),
            SpatialShapeDeclarationV2::new(
                field(SpatialShapeSymbolV2::new(1), span(1370)),
                SpatialShapeGeometryV2::Path { verbs: Vec::new() },
                span(1371),
            ),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        1349,
    )]);
    let verbs = program(vec![node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(1380),
        vec![SpatialShapeDeclarationV2::new(
            field(SpatialShapeSymbolV2::new(0), span(1390)),
            SpatialShapeGeometryV2::Path {
                verbs: vec![
                    SpatialPathVerbRecipeV2::MoveTo {
                        to: point(0, 0, 1391),
                        span: span(1393),
                    },
                    SpatialPathVerbRecipeV2::LineTo {
                        to: point(1, 1, 1400),
                        span: span(1402),
                    },
                ],
            },
            span(1403),
        )],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        1379,
    )]);
    let points = program(vec![node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(1410),
        vec![SpatialShapeDeclarationV2::new(
            field(SpatialShapeSymbolV2::new(0), span(1420)),
            SpatialShapeGeometryV2::Polygon {
                points: vec![
                    SpatialPolygonPointV2::new(point(0, 0, 1421), span(1423)),
                    SpatialPolygonPointV2::new(point(1, 1, 1430), span(1432)),
                ],
            },
            span(1433),
        )],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        1409,
    )]);
    let stops = program(vec![node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(1440),
        Vec::new(),
        vec![SpatialBrushDeclarationV2::new(
            field(SpatialBrushSymbolV2::new(0), span(1450)),
            SpatialBrushContentV2::LinearGradient {
                start: point(0, 0, 1451),
                end: point(1, 1, 1453),
                stops: vec![
                    SpatialGradientStopV2::new(
                        field(0, span(1455)),
                        field(SpatialBindingV2::Literal([0, 0, 0, 0]), span(1456)),
                        span(1457),
                    ),
                    SpatialGradientStopV2::new(
                        field(u16::MAX, span(1460)),
                        field(SpatialBindingV2::Literal([0, 0, 0, 0]), span(1461)),
                        span(1462),
                    ),
                ],
            },
            span(1463),
        )],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        1439,
    )]);
    let images = program_with(
        SUPPORTED_SPATIAL_FORMAT,
        NS,
        REV,
        viewport(1470),
        Vec::new(),
        vec![image(0, vec![0], 1480), image(1, vec![0], 1490)],
        span(1469),
    );
    let bytes = program_with(
        SUPPORTED_SPATIAL_FORMAT,
        NS,
        REV,
        viewport(1500),
        Vec::new(),
        vec![image(0, vec![0, 1], 1510)],
        span(1499),
    );

    vec![
        LimitCase {
            program: nodes,
            count: 2,
            kind: ValidationLimitKind::SpatialNodes,
            crossing_span: span(1118),
        },
        LimitCase {
            program: shapes,
            count: 2,
            kind: ValidationLimitKind::SpatialShapes,
            crossing_span: span(1145),
        },
        LimitCase {
            program: brushes,
            count: 2,
            kind: ValidationLimitKind::SpatialBrushes,
            crossing_span: span(1172),
        },
        LimitCase {
            program: clips,
            count: 2,
            kind: ValidationLimitKind::SpatialClips,
            crossing_span: span(1212),
        },
        LimitCase {
            program: paints,
            count: 2,
            kind: ValidationLimitKind::SpatialPaintItems,
            crossing_span: span(1263),
        },
        LimitCase {
            program: hits,
            count: 2,
            kind: ValidationLimitKind::SpatialHitItems,
            crossing_span: span(1302),
        },
        LimitCase {
            program: semantics,
            count: 2,
            kind: ValidationLimitKind::SpatialSemanticItems,
            crossing_span: span(1341),
        },
        LimitCase {
            program: paths,
            count: 2,
            kind: ValidationLimitKind::SpatialPaths,
            crossing_span: span(1371),
        },
        LimitCase {
            program: verbs,
            count: 2,
            kind: ValidationLimitKind::SpatialPathVerbs,
            crossing_span: span(1402),
        },
        LimitCase {
            program: points,
            count: 2,
            kind: ValidationLimitKind::SpatialPolygonPoints,
            crossing_span: span(1432),
        },
        LimitCase {
            program: stops,
            count: 2,
            kind: ValidationLimitKind::SpatialGradientStops,
            crossing_span: span(1462),
        },
        LimitCase {
            program: images,
            count: 2,
            kind: ValidationLimitKind::SpatialImages,
            crossing_span: span(1494),
        },
        LimitCase {
            program: bytes,
            count: 2,
            kind: ValidationLimitKind::SpatialImageBytes,
            crossing_span: span(1514),
        },
    ]
}

#[test]
fn all_thirteen_static_limits_are_inclusive_and_report_the_crossing_record() {
    let style = style();
    for (index, case) in cases().into_iter().enumerate() {
        let mut exact = [64; 13];
        exact[index] = case.count;
        validate_spatial(
            &style,
            case.program.clone(),
            SpatialValidationLimitsV2::new(exact),
        )
        .expect("equality must pass");

        exact[index] -= 1;
        let error = validate_spatial(&style, case.program, SpatialValidationLimitsV2::new(exact))
            .expect_err("one over must fail");
        assert_eq!(
            error.kind(),
            IrValidationErrorKind::LimitExceeded(case.kind)
        );
        assert_eq!(error.span(), case.crossing_span);
    }
}

#[test]
fn count_preflight_precedes_symbol_lookup_and_uses_printed_limit_order() {
    let style = style();
    let duplicate = program(vec![
        node(0, ROOT, SpatialNodeParentV2::Viewport, 1520),
        node(0, ROOT, SpatialNodeParentV2::Viewport, 1530),
    ]);
    let error = validate_spatial(&style, duplicate, SpatialValidationLimitsV2::new([0; 13]))
        .expect_err("count phase should win");
    assert_eq!(
        error.kind(),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::SpatialNodes)
    );
    assert_eq!(error.span(), span(1528));

    validate_spatial(
        &style,
        program(Vec::new()),
        SpatialValidationLimitsV2::new([0; 13]),
    )
    .expect("zero limits should accept an empty spatial program");
}
