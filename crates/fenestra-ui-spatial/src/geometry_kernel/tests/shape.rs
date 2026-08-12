use super::*;

#[test]
fn rect_preflights_every_scalar_before_extent_semantics() {
    let low = SpatialScalarV2::MIN_RAW - 1;
    let high = SpatialScalarV2::MAX_RAW + 1;
    let cases = [
        (point(low, 0), scalar(1), scalar(1), GeometryK1Field::RectX),
        (point(0, high), scalar(1), scalar(1), GeometryK1Field::RectY),
        (
            point(0, 0),
            scalar(low),
            scalar(1),
            GeometryK1Field::RectWidth,
        ),
        (
            point(0, 0),
            scalar(1),
            scalar(high),
            GeometryK1Field::RectHeight,
        ),
    ];

    for (origin, width, height, field) in cases {
        expect_error(
            validate_rect_k1(SHAPE_INDEX, origin, width, height),
            GeometryK1ErrorKind::ScalarOutOfDomain,
            shape_location(field),
        );
    }

    expect_error(
        validate_rect_k1(SHAPE_INDEX, point(high, high), scalar(low), scalar(low)),
        GeometryK1ErrorKind::ScalarOutOfDomain,
        shape_location(GeometryK1Field::RectX),
    );
}

#[test]
fn rect_rejects_negative_extents_in_width_then_height_order() {
    expect_error(
        validate_rect_k1(SHAPE_INDEX, point(0, 0), scalar(-1), scalar(-1)),
        GeometryK1ErrorKind::InvalidShape(GeometryK1ShapeKind::NegativeExtent),
        shape_location(GeometryK1Field::RectWidth),
    );
    expect_error(
        validate_rect_k1(SHAPE_INDEX, point(0, 0), scalar(1), scalar(-1)),
        GeometryK1ErrorKind::InvalidShape(GeometryK1ShapeKind::NegativeExtent),
        shape_location(GeometryK1Field::RectHeight),
    );

    expect_error(
        validate_rect_k1(
            SHAPE_INDEX,
            point(0, 0),
            scalar(-1),
            scalar(SpatialScalarV2::MAX_RAW + 1),
        ),
        GeometryK1ErrorKind::ScalarOutOfDomain,
        shape_location(GeometryK1Field::RectHeight),
    );
}

#[test]
fn rect_proof_accepts_zero_and_defers_bound_arithmetic_to_k3() {
    for (width, height) in [(0, 1), (1, 0), (0, 0)] {
        let proof: ValidatedRectK1 = expect_valid(validate_rect_k1(
            SHAPE_INDEX,
            point(2, 3),
            scalar(width),
            scalar(height),
        ));
        assert_eq!(proof.width(), scalar(width));
        assert_eq!(proof.height(), scalar(height));
    }

    let origin = point(SpatialScalarV2::MAX_RAW, SpatialScalarV2::MIN_RAW);
    let proof: ValidatedRectK1 = expect_valid(validate_rect_k1(
        SHAPE_INDEX,
        origin,
        scalar(SpatialScalarV2::MAX_RAW),
        scalar(0),
    ));
    assert_eq!(proof.origin(), origin);
    assert_eq!(proof.width(), scalar(SpatialScalarV2::MAX_RAW));
    assert_eq!(proof.height(), scalar(0));
}

#[test]
fn circle_preflights_every_scalar_before_radius_semantics() {
    let low = SpatialScalarV2::MIN_RAW - 1;
    let high = SpatialScalarV2::MAX_RAW + 1;
    let cases = [
        (point(low, 0), scalar(0), GeometryK1Field::CircleCenterX),
        (point(0, high), scalar(0), GeometryK1Field::CircleCenterY),
        (point(0, 0), scalar(low), GeometryK1Field::CircleRadius),
    ];

    for (center, radius, field) in cases {
        expect_error(
            validate_circle_k1(SHAPE_INDEX, center, radius),
            GeometryK1ErrorKind::ScalarOutOfDomain,
            shape_location(field),
        );
    }
}

#[test]
fn circle_rejects_negative_radius_and_accepts_zero() {
    expect_error(
        validate_circle_k1(SHAPE_INDEX, point(0, 0), scalar(-1)),
        GeometryK1ErrorKind::InvalidShape(GeometryK1ShapeKind::NegativeRadius),
        shape_location(GeometryK1Field::CircleRadius),
    );

    let center = point(SpatialScalarV2::MAX_RAW, SpatialScalarV2::MIN_RAW);
    let proof: ValidatedCircleK1 = expect_valid(validate_circle_k1(SHAPE_INDEX, center, scalar(0)));
    assert_eq!(proof.center(), center);
    assert_eq!(proof.radius(), scalar(0));
}

#[test]
fn circle_proof_defers_canonical_bound_overflow_to_k3() {
    let proof: ValidatedCircleK1 = expect_valid(validate_circle_k1(
        SHAPE_INDEX,
        point(SpatialScalarV2::MAX_RAW, 0),
        scalar(SpatialScalarV2::MAX_RAW),
    ));
    assert_eq!(proof.center(), point(SpatialScalarV2::MAX_RAW, 0));
    assert_eq!(proof.radius(), scalar(SpatialScalarV2::MAX_RAW));
}

#[test]
fn polygon_scalar_scan_is_point_then_axis_and_precedes_limits() {
    let low = SpatialScalarV2::MIN_RAW - 1;
    let high = SpatialScalarV2::MAX_RAW + 1;
    let invalid_x = [point(0, 0), point(low, 1), point(2, 2)];
    expect_error(
        validate_polygon_k1(SHAPE_INDEX, &invalid_x, 2),
        GeometryK1ErrorKind::ScalarOutOfDomain,
        polygon_location(1, GeometryK1Field::X),
    );

    let invalid_y = [point(0, 0), point(1, high), point(2, 2)];
    expect_error(
        validate_polygon_k1(SHAPE_INDEX, &invalid_y, 2),
        GeometryK1ErrorKind::ScalarOutOfDomain,
        polygon_location(1, GeometryK1Field::Y),
    );

    let priority = [point(0, high), point(low, 0), point(2, 2)];
    expect_error(
        validate_polygon_k1(SHAPE_INDEX, &priority, 2),
        GeometryK1ErrorKind::ScalarOutOfDomain,
        polygon_location(0, GeometryK1Field::Y),
    );

    let axis_priority = [point(low, high), point(1, 1), point(2, 2)];
    expect_error(
        validate_polygon_k1(SHAPE_INDEX, &axis_priority, 2),
        GeometryK1ErrorKind::ScalarOutOfDomain,
        polygon_location(0, GeometryK1Field::X),
    );

    let short = [point(0, 0), point(1, high)];
    expect_error(
        validate_polygon_k1(SHAPE_INDEX, &short, 8),
        GeometryK1ErrorKind::ScalarOutOfDomain,
        polygon_location(1, GeometryK1Field::Y),
    );
}

#[test]
fn polygon_limit_precedes_too_short_and_accepts_its_edge() {
    let triangle = [point(0, 0), point(3, 0), point(0, 3)];
    let proof: ValidatedPolygonK1<'_> =
        expect_valid(validate_polygon_k1(SHAPE_INDEX, &triangle, 3));
    assert_eq!(proof.points(), triangle.as_slice());

    let four = [point(0, 0), point(3, 0), point(3, 3), point(0, 3)];
    expect_limit(
        validate_polygon_k1(SHAPE_INDEX, &four, 3),
        GeometryK1LimitKind::PolygonPointsPerShape,
        shape_location(GeometryK1Field::PolygonPointLength),
        4,
        3,
    );

    let two = [point(0, 0), point(1, 1)];
    expect_limit(
        validate_polygon_k1(SHAPE_INDEX, &two, 1),
        GeometryK1LimitKind::PolygonPointsPerShape,
        shape_location(GeometryK1Field::PolygonPointLength),
        2,
        1,
    );
}

#[test]
fn registered_polygon_limit_accepts_256_and_rejects_257() {
    assert_eq!(POLYGON_POINT_MAXIMUM, 256);
    let mut points = (0..POLYGON_POINT_MAXIMUM)
        .map(|index| point(index as i64, 0))
        .collect::<Vec<_>>();

    let proof: ValidatedPolygonK1<'_> = expect_valid(validate_polygon_k1(
        SHAPE_INDEX,
        &points,
        POLYGON_POINT_MAXIMUM,
    ));
    assert_eq!(proof.points(), points.as_slice());

    points.push(point(POLYGON_POINT_MAXIMUM as i64, 0));
    expect_limit(
        validate_polygon_k1(SHAPE_INDEX, &points, POLYGON_POINT_MAXIMUM),
        GeometryK1LimitKind::PolygonPointsPerShape,
        shape_location(GeometryK1Field::PolygonPointLength),
        (POLYGON_POINT_MAXIMUM + 1) as u128,
        POLYGON_POINT_MAXIMUM as u128,
    );
}

#[test]
fn polygon_lengths_below_three_are_too_short() {
    let cases: [&[SpatialPointV2]; 3] = [&[], &[point(0, 0)], &[point(0, 0), point(1, 1)]];
    for points in cases {
        expect_error(
            validate_polygon_k1(SHAPE_INDEX, points, 8),
            GeometryK1ErrorKind::InvalidShape(GeometryK1ShapeKind::PolygonTooShort),
            shape_location(GeometryK1Field::PolygonPointLength),
        );
    }
}

#[test]
fn polygon_duplicate_errors_have_exact_priority_and_locations() {
    let first = point(0, 0);
    let repeated = [first, point(3, 0), point(3, 3), first];
    expect_error(
        validate_polygon_k1(SHAPE_INDEX, &repeated, 8),
        GeometryK1ErrorKind::InvalidShape(GeometryK1ShapeKind::PolygonRepeatedFirst),
        polygon_location(3, GeometryK1Field::X),
    );

    let adjacent = [first, point(3, 0), point(3, 0), point(0, 3)];
    expect_error(
        validate_polygon_k1(SHAPE_INDEX, &adjacent, 8),
        GeometryK1ErrorKind::InvalidShape(GeometryK1ShapeKind::PolygonAdjacentEqual),
        polygon_location(2, GeometryK1Field::X),
    );

    let both = [first, first, point(3, 3), first];
    expect_error(
        validate_polygon_k1(SHAPE_INDEX, &both, 8),
        GeometryK1ErrorKind::InvalidShape(GeometryK1ShapeKind::PolygonRepeatedFirst),
        polygon_location(3, GeometryK1Field::X),
    );
}

#[test]
fn polygon_proof_borrows_allowed_collinear_and_nonadjacent_duplicates() {
    let points = [
        point(SpatialScalarV2::MIN_RAW, SpatialScalarV2::MAX_RAW),
        point(0, 0),
        point(SpatialScalarV2::MIN_RAW, SpatialScalarV2::MAX_RAW),
        point(4, 4),
    ];
    let proof: ValidatedPolygonK1<'_> = expect_valid(validate_polygon_k1(SHAPE_INDEX, &points, 4));
    assert_eq!(proof.points(), points.as_slice());
    assert!(std::ptr::eq(proof.points(), points.as_slice()));
}

#[test]
fn reversed_and_self_intersecting_polygons_are_retained() {
    let reversed = [point(0, 0), point(0, 4), point(4, 4), point(4, 0)];
    let bow_tie = [point(0, 0), point(4, 4), point(0, 4), point(4, 0)];

    for points in [&reversed[..], &bow_tie[..]] {
        let proof: ValidatedPolygonK1<'_> =
            expect_valid(validate_polygon_k1(SHAPE_INDEX, points, 4));
        assert_eq!(proof.points(), points);
    }
}
