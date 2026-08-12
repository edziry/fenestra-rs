use super::*;
use support::*;

fn custom_node(
    symbol: u32,
    target: TemplateNodeId,
    parent: SpatialNodeParentV2,
    placement: SpatialPlacementRecipeV2,
    container: SpatialContainerRecipeV2,
    shapes: Vec<SpatialShapeDeclarationV2>,
    brushes: Vec<SpatialBrushDeclarationV2>,
    index: u32,
) -> SpatialNodeDeclarationV2 {
    SpatialNodeDeclarationV2::new(
        field(SpatialNodeSymbolV2::new(symbol), span(index)),
        field(target, span(index + 1)),
        parent,
        placement,
        container,
        shapes,
        brushes,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        span(index + 2),
    )
}

fn wrong_dimension(source: SourceSpan) -> SpatialDimensionRecipeV2 {
    SpatialDimensionRecipeV2::new(
        field(SpatialBindingV2::Property(COLOR), source),
        lit_i(1, 6201),
        lit_i(2, 6202),
    )
}

fn wrong_container(source: SourceSpan) -> SpatialContainerRecipeV2 {
    SpatialContainerRecipeV2::new(
        SpatialAxisV2::Row,
        SpatialPaddingRecipeV2::new(
            field(SpatialBindingV2::Property(COLOR), source),
            lit_i(0, 6211),
            lit_i(0, 6212),
            lit_i(0, 6213),
        ),
        field(SpatialBindingV2::Property(COLOR), span(6214)),
    )
}

#[test]
fn phase_four_semantics_follow_node_and_stored_field_order() {
    let style = style();
    let layout_source = span(6220);
    let faulty_placement = SpatialPlacementRecipeV2::Layout(SpatialLayoutPlacementRecipeV2::new(
        wrong_dimension(layout_source),
        wrong_dimension(span(6221)),
        transform(6222),
    ));
    let first = custom_node(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        faulty_placement,
        wrong_container(span(6230)),
        Vec::new(),
        Vec::new(),
        6219,
    );
    let second = custom_node(
        1,
        STATIC_A,
        parent(0, 6240),
        SpatialPlacementRecipeV2::Layout(SpatialLayoutPlacementRecipeV2::new(
            wrong_dimension(span(6241)),
            wrong_dimension(span(6242)),
            transform(6243),
        )),
        container(6250),
        Vec::new(),
        Vec::new(),
        6239,
    );
    assert_error(
        &style,
        program(vec![first, second]),
        IrValidationErrorKind::SpatialPropertyTypeMismatch,
        layout_source,
    );

    let container_source = span(6260);
    let first = custom_node(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(6261),
        wrong_container(container_source),
        Vec::new(),
        Vec::new(),
        6259,
    );
    let second = custom_node(
        1,
        STATIC_A,
        parent(0, 6270),
        SpatialPlacementRecipeV2::Layout(SpatialLayoutPlacementRecipeV2::new(
            wrong_dimension(span(6271)),
            wrong_dimension(span(6272)),
            transform(6273),
        )),
        container(6280),
        Vec::new(),
        Vec::new(),
        6269,
    );
    assert_error(
        &style,
        program(vec![first, second]),
        IrValidationErrorKind::SpatialPropertyTypeMismatch,
        container_source,
    );
}

#[test]
fn phase_five_semantics_precede_later_local_duplicate_detection() {
    let style = style();
    let source = span(6290);
    let first_shape = SpatialShapeDeclarationV2::new(
        field(SpatialShapeSymbolV2::new(0), span(6291)),
        SpatialShapeGeometryV2::Rect {
            origin: point(0, 0, 6292),
            width: field(SpatialBindingV2::Property(COLOR), source),
            height: lit_f(1, 6295),
        },
        span(6296),
    );
    let declaration = custom_node(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(6300),
        container(6310),
        vec![first_shape, shape(0, 6320)],
        Vec::new(),
        6299,
    );
    assert_error(
        &style,
        program(vec![declaration]),
        IrValidationErrorKind::SpatialPropertyTypeMismatch,
        source,
    );
}

#[test]
fn phase_six_semantics_precede_later_local_duplicate_detection() {
    let style = style();
    let source = span(6330);
    let first_brush = SpatialBrushDeclarationV2::new(
        field(SpatialBrushSymbolV2::new(0), span(6331)),
        SpatialBrushContentV2::Solid {
            color: field(SpatialBindingV2::Property(SCALAR), source),
        },
        span(6332),
    );
    let declaration = custom_node(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(6340),
        container(6350),
        Vec::new(),
        vec![first_brush, brush(0, 6360)],
        6339,
    );
    assert_error(
        &style,
        program(vec![declaration]),
        IrValidationErrorKind::SpatialPropertyTypeMismatch,
        source,
    );
}

#[test]
fn phase_seven_fields_precede_later_global_duplicate_detection() {
    let style = style();
    let source = invalid_span(6371);
    let first = SpatialImageDeclarationV2::new(
        field(SpatialImageSymbolV2::new(0), span(6370)),
        field(1, source),
        field(1, invalid_span(6372)),
        field(4, invalid_span(6373)),
        vec![0, 0, 0, 0].into_boxed_slice(),
        span(6374),
    );
    let second = SpatialImageDeclarationV2::new(
        field(SpatialImageSymbolV2::new(0), span(6380)),
        field(1, span(6381)),
        field(1, span(6382)),
        field(4, span(6383)),
        vec![0, 0, 0, 0].into_boxed_slice(),
        span(6384),
    );
    assert_error(
        &style,
        program_with(
            SUPPORTED_SPATIAL_FORMAT,
            NS,
            REV,
            viewport(6390),
            Vec::new(),
            vec![first, second],
            span(6396),
        ),
        IrValidationErrorKind::InvalidSourceSpan,
        source,
    );
}

fn anchor_node(
    symbol: u32,
    target: TemplateNodeId,
    parent: SpatialNodeParentV2,
    anchor: u32,
    anchor_at: u32,
    index: u32,
) -> SpatialNodeDeclarationV2 {
    node_with(
        symbol,
        target,
        parent,
        free_placement(
            SpatialAnchorTargetRecipeV2::Node(field(
                SpatialNodeSymbolV2::new(anchor),
                span(anchor_at),
            )),
            index + 10,
        ),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        index,
    )
}

fn anchor_ladder(first: usize) -> SpatialProgramV2 {
    let root_anchor = if first == 0 { 999 } else { 1 };
    let static_a_anchor = if first <= 1 { 1 } else { 0 };
    let static_b_anchor = if first <= 2 { 3 } else { 0 };
    program(vec![
        anchor_node(
            0,
            ROOT,
            SpatialNodeParentV2::Viewport,
            root_anchor,
            6400,
            6401,
        ),
        anchor_node(1, STATIC_A, parent(0, 6410), static_a_anchor, 6411, 6412),
        anchor_node(2, STATIC_B, parent(0, 6420), static_b_anchor, 6421, 6422),
        node(3, OUTER, parent(0, 6430), 6431),
    ])
}

#[test]
fn phase_ten_checks_existence_then_self_then_context_in_node_order() {
    let style = style();
    for (first, kind, source) in [
        (
            0,
            IrValidationErrorKind::MissingSpatialAnchorTarget,
            span(6400),
        ),
        (1, IrValidationErrorKind::SelfAnchorTarget, span(6411)),
        (
            2,
            IrValidationErrorKind::SpatialAnchorContextMismatch,
            span(6421),
        ),
    ] {
        assert_error(&style, anchor_ladder(first), kind, source);
    }
}
