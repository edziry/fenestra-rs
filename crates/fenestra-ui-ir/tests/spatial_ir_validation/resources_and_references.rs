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

fn address(
    owner: u32,
    owner_span: SourceSpan,
    clip: u32,
    clip_span: SourceSpan,
) -> SpatialClipAddressV2 {
    SpatialClipAddressV2::new(
        field(SpatialNodeSymbolV2::new(owner), owner_span),
        field(SpatialClipSymbolV2::new(clip), clip_span),
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

#[test]
fn phases_five_through_seven_enforce_local_and_global_symbol_uniqueness() {
    let style = style();
    let cases = vec![
        (
            program(vec![node_with(
                0,
                ROOT,
                SpatialNodeParentV2::Viewport,
                placement(600),
                vec![shape(5, 610), shape(5, 620)],
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                599,
            )]),
            IrValidationErrorKind::DuplicateSpatialShape,
            span(620),
        ),
        (
            program(vec![node_with(
                0,
                ROOT,
                SpatialNodeParentV2::Viewport,
                placement(630),
                Vec::new(),
                vec![brush(5, 640), brush(5, 650)],
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                629,
            )]),
            IrValidationErrorKind::DuplicateSpatialBrush,
            span(650),
        ),
        (
            program_with(
                SUPPORTED_SPATIAL_FORMAT,
                NS,
                REV,
                viewport(660),
                Vec::new(),
                vec![
                    image(5, vec![0, 0, 0, 0], 670),
                    image(5, vec![0, 0, 0, 0], 680),
                ],
                span(659),
            ),
            IrValidationErrorKind::DuplicateSpatialImage,
            span(680),
        ),
    ];
    for (program, expected, source) in cases {
        assert_error(&style, program, expected, source);
    }
}

#[test]
fn invalid_resource_record_span_precedes_its_symbol_and_payload() {
    let style = style();
    let bad_span = invalid_span(690);
    let invalid_shape = SpatialShapeDeclarationV2::new(
        field(SpatialShapeSymbolV2::new(0), invalid_span(691)),
        SpatialShapeGeometryV2::Rect {
            origin: point(MAX_FIXED + 1, 0, 692),
            width: lit_f(1, 694),
            height: lit_f(1, 695),
        },
        bad_span,
    );
    let declaration = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(700),
        vec![invalid_shape],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        699,
    );
    assert_error(
        &style,
        program(vec![declaration]),
        IrValidationErrorKind::InvalidSourceSpan,
        bad_span,
    );
}

#[test]
fn phase_eight_reports_clip_symbol_owner_order_ancestry_and_shape_failures() {
    let style = style();
    let cases = vec![
        (
            vec![node_with(
                0,
                ROOT,
                SpatialNodeParentV2::Viewport,
                placement(710),
                vec![shape(0, 720)],
                Vec::new(),
                vec![clip(0, None, 0, 730), clip(0, None, 0, 740)],
                Vec::new(),
                Vec::new(),
                Vec::new(),
                709,
            )],
            IrValidationErrorKind::DuplicateSpatialClip,
            span(740),
        ),
        (
            vec![node_with(
                0,
                ROOT,
                SpatialNodeParentV2::Viewport,
                placement(750),
                vec![shape(0, 760)],
                Vec::new(),
                vec![clip(0, Some(address(999, span(776), 0, span(777))), 0, 769)],
                Vec::new(),
                Vec::new(),
                Vec::new(),
                749,
            )],
            IrValidationErrorKind::MissingSpatialClipOwner,
            span(776),
        ),
        (
            vec![node_with(
                0,
                ROOT,
                SpatialNodeParentV2::Viewport,
                placement(780),
                vec![shape(0, 790)],
                Vec::new(),
                vec![clip(0, Some(address(0, span(806), 999, span(807))), 0, 799)],
                Vec::new(),
                Vec::new(),
                Vec::new(),
                779,
            )],
            IrValidationErrorKind::MissingSpatialClip,
            span(807),
        ),
        (
            vec![node_with(
                0,
                ROOT,
                SpatialNodeParentV2::Viewport,
                placement(810),
                vec![shape(0, 820)],
                Vec::new(),
                vec![
                    clip(0, Some(address(0, span(836), 1, span(837))), 0, 829),
                    clip(1, None, 0, 840),
                ],
                Vec::new(),
                Vec::new(),
                Vec::new(),
                809,
            )],
            IrValidationErrorKind::SpatialClipParentNotEarlier,
            span(837),
        ),
        (
            vec![node_with(
                0,
                ROOT,
                SpatialNodeParentV2::Viewport,
                placement(850),
                vec![shape(0, 860)],
                Vec::new(),
                vec![clip(0, None, 999, 870)],
                Vec::new(),
                Vec::new(),
                Vec::new(),
                849,
            )],
            IrValidationErrorKind::MissingSpatialShape,
            span(871),
        ),
    ];
    for (nodes, expected, source) in cases {
        assert_error(&style, program(nodes), expected, source);
    }

    let owner_span = span(900);
    let siblings = vec![
        node_with(
            0,
            STATIC_A,
            SpatialNodeParentV2::Viewport,
            placement(880),
            vec![shape(0, 890)],
            Vec::new(),
            vec![clip(0, None, 0, 895)],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            879,
        ),
        node_with(
            1,
            STATIC_B,
            SpatialNodeParentV2::Viewport,
            placement(910),
            vec![shape(0, 920)],
            Vec::new(),
            vec![clip(0, Some(address(0, owner_span, 0, span(901))), 0, 930)],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            909,
        ),
    ];
    assert_error(
        &style,
        program(siblings),
        IrValidationErrorKind::SpatialClipOwnerNotAncestor,
        owner_span,
    );
}

#[test]
fn clip_parent_may_address_an_earlier_spatial_ancestor() {
    let style = style();
    let ancestor = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(940),
        vec![shape(0, 950)],
        Vec::new(),
        vec![clip(0, None, 0, 960)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        939,
    );
    let child = node_with(
        1,
        STATIC_A,
        parent(0, 970),
        placement(971),
        vec![shape(0, 980)],
        Vec::new(),
        vec![clip(0, Some(address(0, span(990), 0, span(991))), 0, 992)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        969,
    );

    validate(&style, program(vec![ancestor, child])).expect("ancestor clip parent should validate");
}

#[test]
fn same_owner_clip_parent_accepts_an_earlier_local_clip() {
    let style = style();
    let clips = vec![
        clip(0, None, 0, 6460),
        clip(1, Some(address(0, span(6463), 0, span(6464))), 0, 6465),
    ];
    let declaration = node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(6470),
        vec![shape(0, 6480)],
        Vec::new(),
        clips,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        6459,
    );
    validate(&style, program(vec![declaration]))
        .expect("a same-owner clip may use an earlier local clip as parent");
}
