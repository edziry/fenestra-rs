use super::*;

#[test]
fn rect_base_overflow_completes_x_before_y_with_extent_locations() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let both = rect(point(maximum, maximum), 1, 1);
    expect_k3_error(
        derive_rect_bounds_k3(SHAPE_INDEX, both),
        SpatialAxisV2::X,
        shape_location(GeometryK1Field::RectWidth),
    );

    let y_only = rect(point(0, maximum), 0, 1);
    expect_k3_error(
        derive_rect_bounds_k3(SHAPE_INDEX, y_only),
        SpatialAxisV2::Y,
        shape_location(GeometryK1Field::RectHeight),
    );
}

#[test]
fn circle_base_overflow_completes_x_before_y_at_radius() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let both = circle(point(maximum, maximum), 1);
    expect_k3_error(
        derive_circle_bounds_k3(SHAPE_INDEX, both),
        SpatialAxisV2::X,
        shape_location(GeometryK1Field::CircleRadius),
    );

    let y_only = circle(point(0, maximum), 1);
    expect_k3_error(
        derive_circle_bounds_k3(SHAPE_INDEX, y_only),
        SpatialAxisV2::Y,
        shape_location(GeometryK1Field::CircleRadius),
    );
}

#[test]
fn circle_base_underflow_completes_x_before_y_at_radius() {
    let minimum = SpatialScalarV2::MIN_RAW;
    let both = circle(point(minimum, minimum), 1);
    expect_k3_error(
        derive_circle_bounds_k3(SHAPE_INDEX, both),
        SpatialAxisV2::X,
        shape_location(GeometryK1Field::CircleRadius),
    );

    let y_only = circle(point(0, minimum), 1);
    expect_k3_error(
        derive_circle_bounds_k3(SHAPE_INDEX, y_only),
        SpatialAxisV2::Y,
        shape_location(GeometryK1Field::CircleRadius),
    );
}

#[test]
fn empty_fill_eligibility_does_not_bypass_rect_base_failure() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let width_degenerate = rect(point(0, maximum), 0, 1);

    expect_k3_error(
        derive_rect_bounds_k3(SHAPE_INDEX, width_degenerate),
        SpatialAxisV2::Y,
        shape_location(GeometryK1Field::RectHeight),
    );
}

#[test]
fn rect_base_failure_precedes_stroke_expansion_failure() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let rect = rect(point(maximum, maximum), 0, 1);
    let source = GeometryK1StrokeSource::Paint { index: 13 };
    let stroke = stroke(source, SpatialScalarV2::MAX_RAW);

    expect_k3_error(
        rect_stroke_bounds_k3(SHAPE_INDEX, rect, source, stroke),
        SpatialAxisV2::Y,
        shape_location(GeometryK1Field::RectHeight),
    );
}
