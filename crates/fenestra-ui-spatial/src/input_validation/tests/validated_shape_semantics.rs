use super::validated_shape_support::{
    circle_values, expect_content, fixture, limits, outside_high, point, polygon, rect,
    rect_values, triangle, validate,
};
use crate::content_diagnostic::SpatialShapeErrorV2;
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::{SpatialPolygonPointFieldV2, SpatialShapeFieldV2};

#[test]
fn rect_scalars_complete_before_width_then_height_semantics() {
    let cases = [
        (
            rect_values(0, 1, 0, 0, -1, -1),
            SpatialContentErrorKindV2::InvalidShape(SpatialShapeErrorV2::NegativeExtent),
            SpatialShapeFieldV2::RectWidth,
        ),
        (
            rect_values(0, 1, 0, 0, 0, -1),
            SpatialContentErrorKindV2::InvalidShape(SpatialShapeErrorV2::NegativeExtent),
            SpatialShapeFieldV2::RectHeight,
        ),
        (
            rect_values(0, 1, 0, 0, -1, outside_high()),
            SpatialContentErrorKindV2::ScalarOutOfDomain,
            SpatialShapeFieldV2::RectHeight,
        ),
    ];

    for (shape, kind, field) in cases {
        let fixture = fixture(vec![shape], Vec::new());
        expect_content(
            validate(&fixture, limits(usize::MAX)),
            kind,
            SpatialErrorLocationV2::Shape { index: 0, field },
        );
    }
}

#[test]
fn circle_scalars_complete_before_negative_radius() {
    let negative = fixture(vec![circle_values(0, 1, 0, 0, -1)], Vec::new());
    expect_content(
        validate(&negative, limits(usize::MAX)),
        SpatialContentErrorKindV2::InvalidShape(SpatialShapeErrorV2::NegativeRadius),
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::CircleRadius,
        },
    );

    let scalar = fixture(vec![circle_values(0, 1, outside_high(), 0, -1)], Vec::new());
    expect_content(
        validate(&scalar, limits(usize::MAX)),
        SpatialContentErrorKindV2::ScalarOutOfDomain,
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::CircleCenterX,
        },
    );
}

#[test]
fn polygon_lengths_below_three_are_too_short_after_scalar_validation() {
    let cases = [
        Vec::new(),
        vec![point(0, 0)],
        vec![point(0, 0), point(0, 0)],
    ];
    for points in cases {
        let fixture = fixture(
            vec![rect(0, 1), polygon(1, 2, 0, points.len() as u32)],
            points,
        );
        expect_content(
            validate(&fixture, limits(usize::MAX)),
            SpatialContentErrorKindV2::InvalidShape(SpatialShapeErrorV2::PolygonTooShort),
            SpatialErrorLocationV2::Shape {
                index: 1,
                field: SpatialShapeFieldV2::PolygonPointLength,
            },
        );
    }
}

#[test]
fn repeated_first_precedes_adjacent_pairs_at_the_exact_local_point() {
    let first = point(10, 10);
    for points in [
        vec![first, point(12, 10), point(12, 12), first],
        vec![first, first, point(12, 12), first],
    ] {
        let mut all_points = triangle(0).to_vec();
        all_points.extend_from_slice(&points);
        let fixture = fixture(vec![polygon(0, 1, 0, 3), polygon(1, 2, 3, 4)], all_points);

        expect_content(
            validate(&fixture, limits(4)),
            SpatialContentErrorKindV2::InvalidShape(SpatialShapeErrorV2::PolygonRepeatedFirst),
            SpatialErrorLocationV2::PolygonPoint {
                shape: 1,
                point: 3,
                field: SpatialPolygonPointFieldV2::X,
            },
        );
    }
}

#[test]
fn the_first_adjacent_pair_names_its_later_local_point() {
    let a = point(10, 10);
    let b = point(12, 10);
    let c = point(10, 12);
    let target = [a, b, b, b, c];
    let mut points = triangle(0).to_vec();
    points.extend_from_slice(&target);
    let fixture = fixture(vec![polygon(0, 1, 0, 3), polygon(1, 2, 3, 5)], points);

    expect_content(
        validate(&fixture, limits(5)),
        SpatialContentErrorKindV2::InvalidShape(SpatialShapeErrorV2::PolygonAdjacentEqual),
        SpatialErrorLocationV2::PolygonPoint {
            shape: 1,
            point: 2,
            field: SpatialPolygonPointFieldV2::X,
        },
    );
}
