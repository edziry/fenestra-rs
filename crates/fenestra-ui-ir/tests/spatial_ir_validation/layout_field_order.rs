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

fn integer(
    base: u32,
    position: usize,
    first_invalid: usize,
) -> SpatialFieldV2<SpatialBindingV2<i32>> {
    field(
        SpatialBindingV2::Literal(0),
        ordered_span(base, position, first_invalid),
    )
}

fn fixed(
    base: u32,
    position: usize,
    first_invalid: usize,
) -> SpatialFieldV2<SpatialBindingV2<i64>> {
    field(
        SpatialBindingV2::Literal(if position == 0 || position == 3 {
            65_536
        } else {
            0
        }),
        ordered_span(base, position, first_invalid),
    )
}

#[test]
fn viewport_record_and_fields_follow_the_frozen_order() {
    let style = style();
    let base = 4200;
    for winner in 0..6 {
        let record = ordered_span(base, 0, winner);
        let viewport = SpatialViewportContainerV2::new(
            SpatialAxisV2::Row,
            field(0, ordered_span(base, 1, winner)),
            field(0, ordered_span(base, 2, winner)),
            field(0, ordered_span(base, 3, winner)),
            field(0, ordered_span(base, 4, winner)),
            field(0, ordered_span(base, 5, winner)),
            record,
        );
        let input = program_with(
            SUPPORTED_SPATIAL_FORMAT,
            NS,
            REV,
            viewport,
            Vec::new(),
            Vec::new(),
            span(4210),
        );
        assert_error(
            &style,
            input,
            IrValidationErrorKind::InvalidSourceSpan,
            invalid_span(base + u32::try_from(winner).unwrap()),
        );
    }
}

#[test]
fn node_record_symbol_template_and_parent_spans_follow_phase_three_order() {
    let style = style();
    let base = 4220;
    for winner in 0..4 {
        let root = node(0, ROOT, SpatialNodeParentV2::Viewport, 4230);
        let child = SpatialNodeDeclarationV2::new(
            field(SpatialNodeSymbolV2::new(1), ordered_span(base, 1, winner)),
            field(STATIC_A, ordered_span(base, 2, winner)),
            SpatialNodeParentV2::Node(field(
                SpatialNodeSymbolV2::new(0),
                ordered_span(base, 3, winner),
            )),
            placement(4240),
            container(4250),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            ordered_span(base, 0, winner),
        );
        assert_error(
            &style,
            program(vec![root, child]),
            IrValidationErrorKind::InvalidSourceSpan,
            invalid_span(base + u32::try_from(winner).unwrap()),
        );
    }
}

fn layout_program(first_invalid: usize) -> SpatialProgramV2 {
    let base = 4300;
    let width = SpatialDimensionRecipeV2::new(
        integer(base, 0, first_invalid),
        integer(base, 1, first_invalid),
        integer(base, 2, first_invalid),
    );
    let height = SpatialDimensionRecipeV2::new(
        integer(base, 3, first_invalid),
        integer(base, 4, first_invalid),
        integer(base, 5, first_invalid),
    );
    let transform = SpatialTransformRecipeV2::new(
        fixed(base, 6, first_invalid),
        fixed(base, 7, first_invalid),
        fixed(base, 8, first_invalid),
        fixed(base, 9, first_invalid),
        fixed(base, 10, first_invalid),
        fixed(base, 11, first_invalid),
        SpatialPointRecipeV2::new(
            fixed(base, 12, first_invalid),
            fixed(base, 13, first_invalid),
        ),
    );
    let container = SpatialContainerRecipeV2::new(
        SpatialAxisV2::Row,
        SpatialPaddingRecipeV2::new(
            integer(base, 14, first_invalid),
            integer(base, 15, first_invalid),
            integer(base, 16, first_invalid),
            integer(base, 17, first_invalid),
        ),
        integer(base, 18, first_invalid),
    );
    let declaration = SpatialNodeDeclarationV2::new(
        field(SpatialNodeSymbolV2::new(0), span(4290)),
        field(ROOT, span(4291)),
        SpatialNodeParentV2::Viewport,
        SpatialPlacementRecipeV2::Layout(SpatialLayoutPlacementRecipeV2::new(
            width, height, transform,
        )),
        container,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        span(4292),
    );
    program(vec![declaration])
}

#[test]
fn layout_transform_and_container_leaves_follow_stored_order() {
    let style = style();
    for winner in 0..19 {
        assert_error(
            &style,
            layout_program(winner),
            IrValidationErrorKind::InvalidSourceSpan,
            invalid_span(4300 + u32::try_from(winner).unwrap()),
        );
    }
}

fn free_program(first_invalid: usize) -> SpatialProgramV2 {
    let base = 4400;
    let target = SpatialAnchorTargetRecipeV2::Node(field(
        SpatialNodeSymbolV2::new(0),
        ordered_span(base, 2, first_invalid),
    ));
    let transform = SpatialTransformRecipeV2::new(
        fixed(base, 5, first_invalid),
        fixed(base, 6, first_invalid),
        fixed(base, 7, first_invalid),
        fixed(base, 8, first_invalid),
        fixed(base, 9, first_invalid),
        fixed(base, 10, first_invalid),
        SpatialPointRecipeV2::new(
            fixed(base, 11, first_invalid),
            fixed(base, 12, first_invalid),
        ),
    );
    let free = SpatialFreePlacementRecipeV2::new(
        integer(base, 0, first_invalid),
        integer(base, 1, first_invalid),
        [
            SpatialAnchorComponentV2::Start,
            SpatialAnchorComponentV2::Start,
        ],
        target,
        [SpatialAnchorComponentV2::End, SpatialAnchorComponentV2::End],
        SpatialPointRecipeV2::new(fixed(base, 3, first_invalid), fixed(base, 4, first_invalid)),
        transform,
    );
    let root = node(0, ROOT, SpatialNodeParentV2::Viewport, 4390);
    let child = node_with(
        1,
        STATIC_A,
        parent(0, 4398),
        SpatialPlacementRecipeV2::Free(free),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        4397,
    );
    program(vec![root, child])
}

#[test]
fn free_placement_leaves_follow_stored_order() {
    let style = style();
    for winner in 0..13 {
        assert_error(
            &style,
            free_program(winner),
            IrValidationErrorKind::InvalidSourceSpan,
            invalid_span(4400 + u32::try_from(winner).unwrap()),
        );
    }
}
