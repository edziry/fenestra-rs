use super::*;

#[test]
fn stroke_expansion_uses_widened_ceil_half_for_widths_one_two_and_three() {
    let derived = expect_derived(derive_rect_bounds_k3(
        SHAPE_INDEX,
        rect(point(10, 20), 20, 20),
    ));
    let source = GeometryK1StrokeSource::Paint { index: 13 };
    let cases = [
        (1, aabb(9, 19, 31, 41)),
        (2, aabb(9, 19, 31, 41)),
        (3, aabb(8, 18, 32, 42)),
    ];

    for (width, expected) in cases {
        let actual = stroke_bounds_k3(&derived, source, stroke(source, width));
        assert_eq!(actual, Ok(expected));
    }
}

#[test]
fn canonical_maximum_width_adds_one_in_widened_arithmetic() {
    let derived = expect_derived(derive_rect_bounds_k3(SHAPE_INDEX, rect(point(0, 0), 0, 0)));
    let source = GeometryK1StrokeSource::Paint { index: 13 };
    let width = SpatialScalarV2::MAX_RAW;
    let expansion = (width + 1) / 2;

    let actual = stroke_bounds_k3(&derived, source, stroke(source, width));
    assert_eq!(
        actual,
        Ok(aabb(-expansion, -expansion, expansion, expansion))
    );
}

#[test]
fn stroke_expands_degenerate_rect_lines_and_points() {
    let source = GeometryK1StrokeSource::Paint { index: 13 };
    let vertical = expect_derived(derive_rect_bounds_k3(SHAPE_INDEX, rect(point(2, 3), 0, 5)));
    assert_eq!(
        stroke_bounds_k3(&vertical, source, stroke(source, 1)),
        Ok(aabb(1, 2, 3, 9))
    );

    let horizontal = expect_derived(derive_rect_bounds_k3(SHAPE_INDEX, rect(point(2, 3), 5, 0)));
    assert_eq!(
        stroke_bounds_k3(&horizontal, source, stroke(source, 1)),
        Ok(aabb(1, 2, 8, 4))
    );

    let point = expect_derived(derive_rect_bounds_k3(SHAPE_INDEX, rect(point(2, 3), 0, 0)));
    assert_eq!(
        stroke_bounds_k3(&point, source, stroke(source, 1)),
        Ok(aabb(1, 2, 3, 4))
    );
}

#[test]
fn stroke_expands_a_zero_radius_circle_from_its_point_base() {
    let source = GeometryK1StrokeSource::Hit { index: 17 };
    let derived = expect_derived(derive_circle_bounds_k3(
        SHAPE_INDEX,
        circle(point(4, -5), 0),
    ));

    assert_eq!(
        stroke_bounds_k3(&derived, source, stroke(source, 3)),
        Ok(aabb(2, -7, 6, -3))
    );
}

#[test]
fn stroke_expansion_reports_x_before_y_at_the_paint_width() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let derived = expect_derived(derive_rect_bounds_k3(
        SHAPE_INDEX,
        rect(point(maximum, maximum), 0, 0),
    ));
    let source = GeometryK1StrokeSource::Paint { index: 13 };

    expect_k3_error(
        stroke_bounds_k3(&derived, source, stroke(source, 1)),
        SpatialAxisV2::X,
        GeometryK1Location::Paint {
            index: 13,
            field: GeometryK1Field::StrokeWidth,
        },
    );
}

#[test]
fn stroke_expansion_reports_y_at_the_hit_width_when_x_fits() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let derived = expect_derived(derive_rect_bounds_k3(
        SHAPE_INDEX,
        rect(point(0, maximum), 0, 0),
    ));
    let source = GeometryK1StrokeSource::Hit { index: 17 };

    expect_k3_error(
        stroke_bounds_k3(&derived, source, stroke(source, 1)),
        SpatialAxisV2::Y,
        GeometryK1Location::Hit {
            index: 17,
            field: GeometryK1Field::StrokeWidth,
        },
    );
}

#[test]
fn stroke_underflow_reports_x_before_y_with_paint_and_hit_locations() {
    let minimum = SpatialScalarV2::MIN_RAW;
    let paint = GeometryK1StrokeSource::Paint { index: 13 };
    let both = expect_derived(derive_rect_bounds_k3(
        SHAPE_INDEX,
        rect(point(minimum, minimum), 0, 0),
    ));
    expect_k3_error(
        stroke_bounds_k3(&both, paint, stroke(paint, 1)),
        SpatialAxisV2::X,
        GeometryK1Location::Paint {
            index: 13,
            field: GeometryK1Field::StrokeWidth,
        },
    );

    let hit = GeometryK1StrokeSource::Hit { index: 17 };
    let y_only = expect_derived(derive_rect_bounds_k3(
        SHAPE_INDEX,
        rect(point(0, minimum), 0, 0),
    ));
    expect_k3_error(
        stroke_bounds_k3(&y_only, hit, stroke(hit, 1)),
        SpatialAxisV2::Y,
        GeometryK1Location::Hit {
            index: 17,
            field: GeometryK1Field::StrokeWidth,
        },
    );
}
