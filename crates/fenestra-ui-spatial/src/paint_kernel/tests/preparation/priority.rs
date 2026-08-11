use super::*;

#[test]
fn scalar_fields_complete_in_declared_order_before_gradient_semantics() {
    let low = SpatialScalarV2::MIN_RAW - 1;
    let high = SpatialScalarV2::MAX_RAW + 1;
    let cases = [
        (
            point(low, high),
            point(low, high),
            PaintP2Field::GradientStartX,
        ),
        (
            point(0, high),
            point(low, high),
            PaintP2Field::GradientStartY,
        ),
        (point(0, 1), point(low, high), PaintP2Field::GradientEndX),
        (point(0, 1), point(2, high), PaintP2Field::GradientEndY),
    ];

    for (start, end, field) in cases {
        expect_p2_error(
            prepare_gradient_p2(BRUSH_INDEX, STOP_START, 0, start, end, &[], 8),
            PaintP2ErrorKind::ScalarOutOfDomain,
            brush_location(field),
        );
    }

    let bad_offsets = [
        stop(1, color(255, 255, 255, 255)),
        stop(65_534, color(255, 255, 255, 255)),
    ];
    expect_p2_error(
        prepare_gradient_p2(
            BRUSH_INDEX,
            STOP_START,
            2,
            point(low, 0),
            point(1, 1),
            &bad_offsets,
            8,
        ),
        PaintP2ErrorKind::ScalarOutOfDomain,
        brush_location(PaintP2Field::GradientStartX),
    );
}

#[test]
fn coincident_endpoints_precede_too_few_stops() {
    expect_p2_error(
        prepare_gradient_p2(
            BRUSH_INDEX,
            STOP_START,
            0,
            point(9, 10),
            point(9, 10),
            &[],
            8,
        ),
        PaintP2ErrorKind::InvalidGradient(PaintP2GradientKind::CoincidentEndpoints),
        brush_location(PaintP2Field::GradientEndX),
    );

    let bad_offsets = [
        stop(1, color(255, 255, 255, 255)),
        stop(65_534, color(255, 255, 255, 255)),
    ];
    expect_p2_error(
        prepare_gradient_p2(
            BRUSH_INDEX,
            STOP_START,
            2,
            point(9, 10),
            point(9, 10),
            &bad_offsets,
            8,
        ),
        PaintP2ErrorKind::InvalidGradient(PaintP2GradientKind::CoincidentEndpoints),
        brush_location(PaintP2Field::GradientEndX),
    );
}

#[test]
fn zero_and_one_stop_precede_offset_semantics() {
    expect_p2_error(
        prepare_gradient_p2(BRUSH_INDEX, STOP_START, 0, point(0, 0), point(1, 1), &[], 8),
        PaintP2ErrorKind::InvalidGradient(PaintP2GradientKind::TooFewStops),
        brush_location(PaintP2Field::GradientStopLength),
    );

    let one = [stop(1_234, color(255, 255, 255, 255))];
    expect_p2_error(
        prepare_gradient_p2(
            BRUSH_INDEX,
            STOP_START,
            1,
            point(0, 0),
            point(1, 1),
            &one,
            8,
        ),
        PaintP2ErrorKind::InvalidGradient(PaintP2GradientKind::TooFewStops),
        brush_location(PaintP2Field::GradientStopLength),
    );
}
