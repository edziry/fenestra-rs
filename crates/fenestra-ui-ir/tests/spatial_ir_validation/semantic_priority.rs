use super::*;
use support::*;

fn clip(
    symbol: u32,
    parent: Option<SpatialClipAddressV2>,
    shape: u32,
    index: u32,
) -> SpatialClipDeclarationV2 {
    SpatialClipDeclarationV2::new(
        field(SpatialClipSymbolV2::new(symbol), span(index)),
        parent,
        field(SpatialShapeSymbolV2::new(shape), span(index + 1)),
        SpatialFillRuleV2::NonZero,
        span(index + 2),
    )
}

fn address(owner: u32, owner_index: u32, clip: u32, clip_index: u32) -> SpatialClipAddressV2 {
    SpatialClipAddressV2::new(
        field(SpatialNodeSymbolV2::new(owner), span(owner_index)),
        field(SpatialClipSymbolV2::new(clip), span(clip_index)),
    )
}

#[test]
fn node_linkage_subchecks_have_an_observable_priority_ladder() {
    let style = style();
    let cases = vec![
        (
            program(vec![
                node(0, ROOT, SpatialNodeParentV2::Viewport, 5300),
                node(0, ROOT, SpatialNodeParentV2::Viewport, 5310),
            ]),
            IrValidationErrorKind::DuplicateSpatialNode,
            span(5310),
        ),
        (
            program(vec![
                node(0, ROOT, SpatialNodeParentV2::Viewport, 5320),
                node(1, ROOT, SpatialNodeParentV2::Viewport, 5330),
                node(
                    2,
                    TemplateNodeId::new(999),
                    SpatialNodeParentV2::Viewport,
                    5340,
                ),
            ]),
            IrValidationErrorKind::DuplicateSpatialTemplate,
            span(5331),
        ),
        (
            program(vec![node(
                0,
                TemplateNodeId::new(999),
                parent(999, 5355),
                5350,
            )]),
            IrValidationErrorKind::MissingSpatialTemplate,
            span(5351),
        ),
        (
            program(vec![
                node(0, OUTER, SpatialNodeParentV2::Viewport, 5360),
                node(1, ROOT, parent(999, 5375), 5370),
                node(2, STATIC_A, parent(0, 5385), 5380),
            ]),
            IrValidationErrorKind::MissingSpatialParent,
            span(5375),
        ),
        (
            program(vec![
                node(0, OUTER, SpatialNodeParentV2::Viewport, 5390),
                node(1, STATIC_A, parent(0, 5405), 5400),
                node(2, ROOT, parent(3, 5415), 5410),
                node(3, STATIC_B, SpatialNodeParentV2::Viewport, 5420),
            ]),
            IrValidationErrorKind::SpatialParentContextMismatch,
            span(5405),
        ),
        (
            program(vec![
                node(0, STATIC_A, parent(1, 5435), 5430),
                node(1, ROOT, SpatialNodeParentV2::Viewport, 5440),
                node(2, STATIC_B, parent(1, 5455), 5450),
            ]),
            IrValidationErrorKind::SpatialParentNotEarlier,
            span(5435),
        ),
        (
            program_with(
                SUPPORTED_SPATIAL_FORMAT,
                NS,
                REV,
                SpatialViewportContainerV2::new(
                    SpatialAxisV2::Row,
                    field(0, invalid_span(5500)),
                    field(0, span(5501)),
                    field(0, span(5502)),
                    field(0, span(5503)),
                    field(0, span(5504)),
                    span(5505),
                ),
                vec![
                    node(0, ROOT, SpatialNodeParentV2::Viewport, 5460),
                    node(1, STATIC_A, parent(0, 5475), 5470),
                    node(2, STATIC_B, parent(0, 5485), 5480),
                    node(3, STATIC_C, parent(1, 5495), 5490),
                ],
                Vec::new(),
                span(5459),
            ),
            IrValidationErrorKind::InvalidSpatialPreorder,
            span(5495),
        ),
    ];
    for (program, kind, source) in cases {
        assert_error(&style, program, kind, source);
    }
}

#[test]
fn clip_resolution_subchecks_have_an_observable_priority_ladder() {
    let style = style();
    let cases = vec![
        (
            vec![node_with(
                0,
                ROOT,
                SpatialNodeParentV2::Viewport,
                placement(5510),
                vec![shape(0, 5520)],
                Vec::new(),
                vec![
                    clip(0, None, 0, 5530),
                    clip(0, Some(address(999, 5540, 0, 5541)), 0, 5535),
                ],
                Vec::new(),
                Vec::new(),
                Vec::new(),
                5509,
            )],
            IrValidationErrorKind::DuplicateSpatialClip,
            span(5535),
        ),
        (
            vec![node_with(
                0,
                ROOT,
                SpatialNodeParentV2::Viewport,
                placement(5550),
                vec![shape(0, 5560)],
                Vec::new(),
                vec![
                    clip(0, Some(address(999, 5570, 0, 5571)), 0, 5565),
                    clip(1, Some(address(0, 5572, 999, 5573)), 0, 5568),
                ],
                Vec::new(),
                Vec::new(),
                Vec::new(),
                5549,
            )],
            IrValidationErrorKind::MissingSpatialClipOwner,
            span(5570),
        ),
        (
            vec![
                node_with(
                    0,
                    STATIC_A,
                    SpatialNodeParentV2::Viewport,
                    placement(5580),
                    vec![shape(0, 5590)],
                    Vec::new(),
                    vec![clip(0, None, 0, 5595)],
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    5579,
                ),
                node_with(
                    1,
                    STATIC_B,
                    SpatialNodeParentV2::Viewport,
                    placement(5600),
                    vec![shape(0, 5610)],
                    Vec::new(),
                    vec![clip(0, Some(address(0, 5620, 999, 5621)), 999, 5615)],
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    5599,
                ),
            ],
            IrValidationErrorKind::MissingSpatialClip,
            span(5621),
        ),
        (
            vec![
                node_with(
                    0,
                    STATIC_A,
                    SpatialNodeParentV2::Viewport,
                    placement(5630),
                    vec![shape(0, 5640)],
                    Vec::new(),
                    vec![clip(0, None, 0, 5645)],
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    5629,
                ),
                node_with(
                    1,
                    STATIC_B,
                    SpatialNodeParentV2::Viewport,
                    placement(5650),
                    Vec::new(),
                    Vec::new(),
                    vec![clip(0, Some(address(0, 5660, 0, 5661)), 999, 5655)],
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    5649,
                ),
            ],
            IrValidationErrorKind::SpatialClipOwnerNotAncestor,
            span(5660),
        ),
        (
            vec![node_with(
                0,
                ROOT,
                SpatialNodeParentV2::Viewport,
                placement(5670),
                Vec::new(),
                Vec::new(),
                vec![
                    clip(0, Some(address(0, 5680, 1, 5681)), 999, 5675),
                    clip(1, None, 999, 5685),
                ],
                Vec::new(),
                Vec::new(),
                Vec::new(),
                5669,
            )],
            IrValidationErrorKind::SpatialClipParentNotEarlier,
            span(5681),
        ),
        (
            vec![node_with(
                0,
                ROOT,
                SpatialNodeParentV2::Viewport,
                placement(5690),
                Vec::new(),
                Vec::new(),
                vec![clip(0, None, 999, 5695)],
                Vec::new(),
                Vec::new(),
                Vec::new(),
                5689,
            )],
            IrValidationErrorKind::MissingSpatialShape,
            span(5696),
        ),
    ];
    for (nodes, kind, source) in cases {
        assert_error(&style, program(nodes), kind, source);
    }
}
