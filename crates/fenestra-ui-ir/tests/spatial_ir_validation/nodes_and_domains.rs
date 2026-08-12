use super::*;
use support::*;

#[test]
fn phase_three_reports_node_linkage_and_preorder_failures_at_reference_fields() {
    let style = style();
    let cases = vec![
        (
            program(vec![
                node(7, ROOT, SpatialNodeParentV2::Viewport, 300),
                node(7, STATIC_A, parent(7, 311), 310),
            ]),
            IrValidationErrorKind::DuplicateSpatialNode,
            span(310),
        ),
        (
            program(vec![
                node(7, ROOT, SpatialNodeParentV2::Viewport, 320),
                node(8, ROOT, parent(7, 335), 330),
            ]),
            IrValidationErrorKind::DuplicateSpatialTemplate,
            span(331),
        ),
        (
            program(vec![node(
                7,
                TemplateNodeId::new(999),
                SpatialNodeParentV2::Viewport,
                340,
            )]),
            IrValidationErrorKind::MissingSpatialTemplate,
            span(341),
        ),
        (
            program(vec![node(7, ROOT, parent(999, 355), 349)]),
            IrValidationErrorKind::MissingSpatialParent,
            span(355),
        ),
        (
            program(vec![
                node(7, OUTER, SpatialNodeParentV2::Viewport, 360),
                node(8, STATIC_A, parent(7, 375), 370),
            ]),
            IrValidationErrorKind::SpatialParentContextMismatch,
            span(375),
        ),
        (
            program(vec![
                node(7, STATIC_A, parent(8, 385), 380),
                node(8, ROOT, SpatialNodeParentV2::Viewport, 390),
            ]),
            IrValidationErrorKind::SpatialParentNotEarlier,
            span(385),
        ),
        (
            program(vec![
                node(7, ROOT, SpatialNodeParentV2::Viewport, 400),
                node(8, STATIC_A, parent(7, 411), 410),
                node(9, STATIC_B, parent(7, 421), 420),
                node(10, STATIC_C, parent(8, 435), 430),
            ]),
            IrValidationErrorKind::InvalidSpatialPreorder,
            span(435),
        ),
    ];

    for (program, kind, source) in cases {
        assert_error(&style, program, kind, source);
    }
}

#[test]
fn keyed_context_requires_the_exact_region_prefix_not_only_equal_depth() {
    let style = style();
    let parent_span = span(435);
    let divergent_parent = vec![
        node(0, OTHER, SpatialNodeParentV2::Viewport, 434),
        node(1, OUTER, parent(0, 435), 440),
    ];
    assert_error(
        &style,
        program(divergent_parent),
        IrValidationErrorKind::SpatialParentContextMismatch,
        parent_span,
    );

    let anchor_span = span(445);
    let divergent_anchor = vec![
        node_with(
            0,
            OUTER,
            SpatialNodeParentV2::Viewport,
            free_placement(
                SpatialAnchorTargetRecipeV2::Node(field(SpatialNodeSymbolV2::new(1), anchor_span)),
                446,
            ),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            444,
        ),
        node(1, OTHER, SpatialNodeParentV2::Viewport, 470),
    ];
    assert_error(
        &style,
        program(divergent_anchor),
        IrValidationErrorKind::SpatialAnchorContextMismatch,
        anchor_span,
    );
}

#[test]
fn omitted_logical_wrappers_do_not_break_a_valid_spatial_parent_prefix() {
    let style = style();
    let nodes = vec![
        node(0, ROOT, SpatialNodeParentV2::Viewport, 475),
        node(1, INNER, parent(0, 485), 484),
    ];
    let validated = validate(&style, program(nodes)).expect("logical wrappers may be omitted");
    assert_eq!(
        validated.region_signature(SpatialNodeSymbolV2::new(1)),
        Some([OUTER_REGION, INNER_REGION].as_slice())
    );
}

#[test]
fn validated_views_retain_sparse_symbols_signatures_style_and_private_domains() {
    let style = style();
    let maximum = u32::MAX;
    let nodes = vec![
        node(maximum, ROOT, SpatialNodeParentV2::Viewport, 440),
        node(1, OUTER, parent(maximum, 451), 450),
        node(2, OUTER_STATIC, parent(1, 461), 460),
        node(3, INNER, parent(1, 471), 470),
    ];
    let raw = program(nodes.clone());
    let retained = raw.clone();
    let first = validate(&style, raw).expect("linked program should validate");
    let clone = first.clone();
    let repeated = validate(&style, program(nodes)).expect("revalidation should validate");

    assert_eq!(
        first
            .node(SpatialNodeSymbolV2::new(maximum))
            .unwrap()
            .template()
            .value(),
        &ROOT
    );
    assert_eq!(
        first
            .node_for_template(INNER)
            .unwrap()
            .symbol()
            .value()
            .get(),
        3
    );
    assert_eq!(
        first.region_signature(SpatialNodeSymbolV2::new(maximum)),
        Some([].as_slice())
    );
    assert_eq!(
        first.region_signature(SpatialNodeSymbolV2::new(1)),
        Some([OUTER_REGION].as_slice())
    );
    assert_eq!(
        first.region_signature(SpatialNodeSymbolV2::new(2)),
        Some([OUTER_REGION].as_slice())
    );
    assert_eq!(
        first.region_signature(SpatialNodeSymbolV2::new(3)),
        Some([OUTER_REGION, INNER_REGION].as_slice())
    );
    assert!(first.style().shares_domain_with(&style));
    assert!(first.shares_domain_with(&clone));
    assert!(!first.shares_domain_with(&repeated));
    assert!(std::ptr::eq(first.program(), clone.program()));
    assert_eq!(first.program(), &retained);
    for declaration in retained.nodes() {
        assert_eq!(first.node(*declaration.symbol().value()), Some(declaration));
        assert_eq!(
            first.node_for_template(*declaration.template().value()),
            Some(declaration)
        );
    }
    assert_eq!(first.node(SpatialNodeSymbolV2::new(999)), None);
    assert_eq!(first.node_for_template(OTHER), None);
    assert_eq!(first.region_signature(SpatialNodeSymbolV2::new(999)), None);
    assert_eq!(format!("{first:?}"), "ValidatedSpatialProgramV2(..)");
}

#[test]
fn phase_ten_accepts_forward_anchors_and_rejects_missing_self_and_context() {
    let style = style();
    let target = |symbol, source| {
        SpatialAnchorTargetRecipeV2::Node(field(SpatialNodeSymbolV2::new(symbol), source))
    };
    let forward = vec![
        node_with(
            0,
            STATIC_A,
            SpatialNodeParentV2::Viewport,
            free_placement(target(1, span(500)), 501),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            490,
        ),
        node(1, STATIC_B, SpatialNodeParentV2::Viewport, 520),
    ];
    validate(&style, program(forward)).expect("compatible forward anchor should validate");

    let strict_prefix = vec![
        node(0, ROOT, SpatialNodeParentV2::Viewport, 521),
        node(1, OUTER, parent(0, 531), 530),
        node_with(
            2,
            INNER,
            parent(1, 541),
            free_placement(target(1, span(542)), 543),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            540,
        ),
    ];
    validate(&style, program(strict_prefix))
        .expect("an ancestor region signature is a valid strict anchor prefix");

    let source = span(530);
    let missing = node_with(
        0,
        STATIC_A,
        SpatialNodeParentV2::Viewport,
        free_placement(target(999, source), 531),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        529,
    );
    assert_error(
        &style,
        program(vec![missing]),
        IrValidationErrorKind::MissingSpatialAnchorTarget,
        source,
    );

    let source = span(550);
    let self_target = node_with(
        0,
        STATIC_A,
        SpatialNodeParentV2::Viewport,
        free_placement(target(0, source), 551),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        549,
    );
    assert_error(
        &style,
        program(vec![self_target]),
        IrValidationErrorKind::SelfAnchorTarget,
        source,
    );

    let source = span(560);
    let nodes = vec![
        node_with(
            0,
            STATIC_A,
            SpatialNodeParentV2::Viewport,
            free_placement(target(1, source), 561),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            559,
        ),
        node(1, OUTER, SpatialNodeParentV2::Viewport, 580),
    ];
    assert_error(
        &style,
        program(nodes),
        IrValidationErrorKind::SpatialAnchorContextMismatch,
        source,
    );
}
