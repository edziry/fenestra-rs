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

fn fixed(
    base: u32,
    position: usize,
    first_invalid: usize,
) -> SpatialFieldV2<SpatialBindingV2<i64>> {
    field(
        SpatialBindingV2::Literal(1),
        ordered_span(base, position, first_invalid),
    )
}

fn shape_program(shape: SpatialShapeDeclarationV2, index: u32) -> SpatialProgramV2 {
    program(vec![node_with(
        0,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(index),
        vec![shape],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        index - 1,
    )])
}

#[test]
fn rectangle_and_circle_fields_follow_record_symbol_then_geometry_order() {
    let style = style();
    for winner in 0..6 {
        let base = 4500;
        let shape = SpatialShapeDeclarationV2::new(
            field(SpatialShapeSymbolV2::new(0), ordered_span(base, 1, winner)),
            SpatialShapeGeometryV2::Rect {
                origin: SpatialPointRecipeV2::new(fixed(base, 2, winner), fixed(base, 3, winner)),
                width: fixed(base, 4, winner),
                height: fixed(base, 5, winner),
            },
            ordered_span(base, 0, winner),
        );
        assert_error(
            &style,
            shape_program(shape, 4510),
            IrValidationErrorKind::InvalidSourceSpan,
            invalid_span(base + u32::try_from(winner).unwrap()),
        );
    }

    for winner in 0..5 {
        let base = 4520;
        let shape = SpatialShapeDeclarationV2::new(
            field(SpatialShapeSymbolV2::new(0), ordered_span(base, 1, winner)),
            SpatialShapeGeometryV2::Circle {
                center: SpatialPointRecipeV2::new(fixed(base, 2, winner), fixed(base, 3, winner)),
                radius: fixed(base, 4, winner),
            },
            ordered_span(base, 0, winner),
        );
        assert_error(
            &style,
            shape_program(shape, 4530),
            IrValidationErrorKind::InvalidSourceSpan,
            invalid_span(base + u32::try_from(winner).unwrap()),
        );
    }
}

#[test]
fn polygon_point_record_precedes_its_coordinate_fields() {
    let style = style();
    for winner in 0..5 {
        let base = 4540;
        let point = SpatialPolygonPointV2::new(
            SpatialPointRecipeV2::new(fixed(base, 3, winner), fixed(base, 4, winner)),
            ordered_span(base, 2, winner),
        );
        let shape = SpatialShapeDeclarationV2::new(
            field(SpatialShapeSymbolV2::new(0), ordered_span(base, 1, winner)),
            SpatialShapeGeometryV2::Polygon {
                points: vec![point],
            },
            ordered_span(base, 0, winner),
        );
        assert_error(
            &style,
            shape_program(shape, 4550),
            IrValidationErrorKind::InvalidSourceSpan,
            invalid_span(base + u32::try_from(winner).unwrap()),
        );
    }
}

fn path_shape(base: u32, winner: usize) -> SpatialShapeDeclarationV2 {
    SpatialShapeDeclarationV2::new(
        field(SpatialShapeSymbolV2::new(0), ordered_span(base, 1, winner)),
        SpatialShapeGeometryV2::Path {
            verbs: vec![
                SpatialPathVerbRecipeV2::MoveTo {
                    to: SpatialPointRecipeV2::new(fixed(base, 3, winner), fixed(base, 4, winner)),
                    span: ordered_span(base, 2, winner),
                },
                SpatialPathVerbRecipeV2::LineTo {
                    to: SpatialPointRecipeV2::new(fixed(base, 6, winner), fixed(base, 7, winner)),
                    span: ordered_span(base, 5, winner),
                },
                SpatialPathVerbRecipeV2::QuadraticTo {
                    control: SpatialPointRecipeV2::new(
                        fixed(base, 9, winner),
                        fixed(base, 10, winner),
                    ),
                    to: SpatialPointRecipeV2::new(fixed(base, 11, winner), fixed(base, 12, winner)),
                    span: ordered_span(base, 8, winner),
                },
                SpatialPathVerbRecipeV2::CubicTo {
                    control1: SpatialPointRecipeV2::new(
                        fixed(base, 14, winner),
                        fixed(base, 15, winner),
                    ),
                    control2: SpatialPointRecipeV2::new(
                        fixed(base, 16, winner),
                        fixed(base, 17, winner),
                    ),
                    to: SpatialPointRecipeV2::new(fixed(base, 18, winner), fixed(base, 19, winner)),
                    span: ordered_span(base, 13, winner),
                },
                SpatialPathVerbRecipeV2::Close {
                    span: ordered_span(base, 20, winner),
                },
            ],
        },
        ordered_span(base, 0, winner),
    )
}

#[test]
fn every_path_verb_record_and_coordinate_follows_stored_order() {
    let style = style();
    let base = 4560;
    for winner in 0..21 {
        assert_error(
            &style,
            shape_program(path_shape(base, winner), 4590),
            IrValidationErrorKind::InvalidSourceSpan,
            invalid_span(base + u32::try_from(winner).unwrap()),
        );
    }
}
