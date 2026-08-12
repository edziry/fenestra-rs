use super::validated_shape_support::{
    circle, circle_values, expect_content, fixture, limits, outside_high, outside_low, point,
    polygon, rect, rect_values, triangle, validate,
};
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::{SpatialPolygonPointFieldV2, SpatialShapeFieldV2};

#[test]
fn every_rect_scalar_maps_both_domain_sides_on_a_later_shape() {
    for raw in [outside_low(), outside_high()] {
        let cases = [
            (rect_values(1, 1, raw, 0, 1, 1), SpatialShapeFieldV2::RectX),
            (rect_values(1, 1, 0, raw, 1, 1), SpatialShapeFieldV2::RectY),
            (
                rect_values(1, 1, 0, 0, raw, 1),
                SpatialShapeFieldV2::RectWidth,
            ),
            (
                rect_values(1, 1, 0, 0, 1, raw),
                SpatialShapeFieldV2::RectHeight,
            ),
        ];

        for (target, field) in cases {
            let fixture = fixture(vec![circle(0, 1), target], Vec::new());
            expect_content(
                validate(&fixture, limits(usize::MAX)),
                SpatialContentErrorKindV2::ScalarOutOfDomain,
                SpatialErrorLocationV2::Shape { index: 1, field },
            );
        }
    }
}

#[test]
fn every_circle_scalar_maps_both_domain_sides_on_a_later_shape() {
    for raw in [outside_low(), outside_high()] {
        let cases = [
            (
                circle_values(1, 1, raw, 0, 1),
                SpatialShapeFieldV2::CircleCenterX,
            ),
            (
                circle_values(1, 1, 0, raw, 1),
                SpatialShapeFieldV2::CircleCenterY,
            ),
            (
                circle_values(1, 1, 0, 0, raw),
                SpatialShapeFieldV2::CircleRadius,
            ),
        ];

        for (target, field) in cases {
            let fixture = fixture(vec![rect(0, 1), target], Vec::new());
            expect_content(
                validate(&fixture, limits(usize::MAX)),
                SpatialContentErrorKindV2::ScalarOutOfDomain,
                SpatialErrorLocationV2::Shape { index: 1, field },
            );
        }
    }
}

#[test]
fn polygon_scalars_map_both_sides_with_shape_local_point_ordinals() {
    for raw in [outside_low(), outside_high()] {
        for (x, y, field) in [
            (raw, 1, SpatialPolygonPointFieldV2::X),
            (1, raw, SpatialPolygonPointFieldV2::Y),
        ] {
            let mut points = triangle(0).to_vec();
            points.extend_from_slice(&triangle(10));
            points[4] = point(x, y);
            let fixture = fixture(vec![polygon(0, 1, 0, 3), polygon(1, 2, 3, 3)], points);

            expect_content(
                validate(&fixture, limits(3)),
                SpatialContentErrorKindV2::ScalarOutOfDomain,
                SpatialErrorLocationV2::PolygonPoint {
                    shape: 1,
                    point: 1,
                    field,
                },
            );
        }
    }
}

#[test]
fn scalar_priority_is_field_order_then_polygon_point_order() {
    let rect_fixture = fixture(
        vec![rect_values(
            0,
            1,
            outside_low(),
            outside_high(),
            outside_low(),
            outside_high(),
        )],
        Vec::new(),
    );
    expect_content(
        validate(&rect_fixture, limits(usize::MAX)),
        SpatialContentErrorKindV2::ScalarOutOfDomain,
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::RectX,
        },
    );

    let circle_fixture = fixture(
        vec![circle_values(
            0,
            1,
            outside_low(),
            outside_high(),
            outside_low(),
        )],
        Vec::new(),
    );
    expect_content(
        validate(&circle_fixture, limits(usize::MAX)),
        SpatialContentErrorKindV2::ScalarOutOfDomain,
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::CircleCenterX,
        },
    );

    let polygon_fixture = fixture(
        vec![polygon(0, 1, 0, 3)],
        vec![
            point(0, outside_high()),
            point(outside_low(), 0),
            point(2, 2),
        ],
    );
    expect_content(
        validate(&polygon_fixture, limits(0)),
        SpatialContentErrorKindV2::ScalarOutOfDomain,
        SpatialErrorLocationV2::PolygonPoint {
            shape: 0,
            point: 0,
            field: SpatialPolygonPointFieldV2::Y,
        },
    );
}
