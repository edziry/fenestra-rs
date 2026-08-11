use super::*;

const INVALID: i64 = SpatialScalarV2::MAX_RAW + 1;

fn suffix_raw(first_invalid: usize, ordinal: usize) -> i64 {
    if ordinal >= first_invalid { INVALID } else { 0 }
}

fn expect_path_scalar(verb: SpatialPathVerbV2, field: GeometryK1Field) {
    expect_error(
        validate_path_k1(PATH_INDEX, &[verb], 0, 8),
        GeometryK1ErrorKind::ScalarOutOfDomain,
        path_location(0, field),
    );
}

#[test]
fn move_and_line_scalar_suffixes_scan_to_x_then_to_y() {
    let fields = [GeometryK1Field::ToX, GeometryK1Field::ToY];

    for (first_invalid, expected) in fields.iter().copied().enumerate() {
        let to = point(suffix_raw(first_invalid, 0), suffix_raw(first_invalid, 1));
        expect_path_scalar(SpatialPathVerbV2::MoveTo { to }, expected);
        expect_path_scalar(SpatialPathVerbV2::LineTo { to }, expected);
    }
}

#[test]
fn quadratic_scalar_suffixes_scan_control_then_destination() {
    let fields = [
        GeometryK1Field::ControlX,
        GeometryK1Field::ControlY,
        GeometryK1Field::ToX,
        GeometryK1Field::ToY,
    ];

    for (first_invalid, expected) in fields.iter().copied().enumerate() {
        let verb = SpatialPathVerbV2::QuadraticTo {
            control: point(suffix_raw(first_invalid, 0), suffix_raw(first_invalid, 1)),
            to: point(suffix_raw(first_invalid, 2), suffix_raw(first_invalid, 3)),
        };
        expect_path_scalar(verb, expected);
    }
}

#[test]
fn cubic_scalar_suffixes_scan_both_controls_then_destination() {
    let fields = [
        GeometryK1Field::Control1X,
        GeometryK1Field::Control1Y,
        GeometryK1Field::Control2X,
        GeometryK1Field::Control2Y,
        GeometryK1Field::ToX,
        GeometryK1Field::ToY,
    ];

    for (first_invalid, expected) in fields.iter().copied().enumerate() {
        let verb = SpatialPathVerbV2::CubicTo {
            control1: point(suffix_raw(first_invalid, 0), suffix_raw(first_invalid, 1)),
            control2: point(suffix_raw(first_invalid, 2), suffix_raw(first_invalid, 3)),
            to: point(suffix_raw(first_invalid, 4), suffix_raw(first_invalid, 5)),
        };
        expect_path_scalar(verb, expected);
    }
}

#[test]
fn rect_scalar_suffixes_scan_origin_then_extents() {
    let fields = [
        GeometryK1Field::RectX,
        GeometryK1Field::RectY,
        GeometryK1Field::RectWidth,
        GeometryK1Field::RectHeight,
    ];

    for (first_invalid, expected) in fields.iter().copied().enumerate() {
        expect_error(
            validate_rect_k1(
                SHAPE_INDEX,
                point(suffix_raw(first_invalid, 0), suffix_raw(first_invalid, 1)),
                scalar(suffix_raw(first_invalid, 2)),
                scalar(suffix_raw(first_invalid, 3)),
            ),
            GeometryK1ErrorKind::ScalarOutOfDomain,
            shape_location(expected),
        );
    }
}

#[test]
fn circle_scalar_suffixes_scan_center_then_radius() {
    let fields = [
        GeometryK1Field::CircleCenterX,
        GeometryK1Field::CircleCenterY,
        GeometryK1Field::CircleRadius,
    ];

    for (first_invalid, expected) in fields.iter().copied().enumerate() {
        expect_error(
            validate_circle_k1(
                SHAPE_INDEX,
                point(suffix_raw(first_invalid, 0), suffix_raw(first_invalid, 1)),
                scalar(suffix_raw(first_invalid, 2)),
            ),
            GeometryK1ErrorKind::ScalarOutOfDomain,
            shape_location(expected),
        );
    }
}
