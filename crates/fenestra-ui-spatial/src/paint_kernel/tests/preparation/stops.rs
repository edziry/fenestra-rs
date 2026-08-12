use super::*;

#[test]
fn first_offset_precedes_last_offset_and_decreasing_order() {
    let two = [
        stop(1, color(1, 1, 1, 255)),
        stop(65_535, color(2, 2, 2, 255)),
    ];
    expect_p2_error(
        prepare_gradient_p2(
            BRUSH_INDEX,
            STOP_START,
            two.len() as u32,
            point(0, 0),
            point(1, 1),
            &two,
            8,
        ),
        PaintP2ErrorKind::InvalidGradient(PaintP2GradientKind::FirstOffset),
        stop_location(0),
    );

    let stops = [
        stop(1, color(1, 1, 1, 255)),
        stop(40_000, color(2, 2, 2, 255)),
        stop(30_000, color(3, 3, 3, 255)),
        stop(65_534, color(4, 4, 4, 255)),
    ];

    expect_p2_error(
        prepare_gradient_p2(
            BRUSH_INDEX,
            STOP_START,
            stops.len() as u32,
            point(0, 0),
            point(1, 1),
            &stops,
            8,
        ),
        PaintP2ErrorKind::InvalidGradient(PaintP2GradientKind::FirstOffset),
        stop_location(0),
    );
}

#[test]
fn last_offset_precedes_decreasing_order() {
    let two = [
        stop(0, color(1, 1, 1, 255)),
        stop(65_534, color(2, 2, 2, 255)),
    ];
    expect_p2_error(
        prepare_gradient_p2(
            BRUSH_INDEX,
            STOP_START,
            two.len() as u32,
            point(0, 0),
            point(1, 1),
            &two,
            8,
        ),
        PaintP2ErrorKind::InvalidGradient(PaintP2GradientKind::LastOffset),
        stop_location(1),
    );

    let stops = [
        stop(0, color(1, 1, 1, 255)),
        stop(40_000, color(2, 2, 2, 255)),
        stop(30_000, color(3, 3, 3, 255)),
        stop(65_534, color(4, 4, 4, 255)),
    ];

    expect_p2_error(
        prepare_gradient_p2(
            BRUSH_INDEX,
            STOP_START,
            stops.len() as u32,
            point(0, 0),
            point(1, 1),
            &stops,
            8,
        ),
        PaintP2ErrorKind::InvalidGradient(PaintP2GradientKind::LastOffset),
        stop_location(3),
    );
}

#[test]
fn decreasing_offset_names_the_first_later_local_ordinal() {
    let stops = [
        stop(0, color(1, 1, 1, 255)),
        stop(30_000, color(2, 2, 2, 255)),
        stop(20_000, color(3, 3, 3, 255)),
        stop(10_000, color(4, 4, 4, 255)),
        stop(65_535, color(5, 5, 5, 255)),
    ];

    expect_p2_error(
        prepare_gradient_p2(
            BRUSH_INDEX,
            STOP_START,
            stops.len() as u32,
            point(0, 0),
            point(1, 1),
            &stops,
            8,
        ),
        PaintP2ErrorKind::InvalidGradient(PaintP2GradientKind::DecreasingOffset),
        stop_location(2),
    );
}
