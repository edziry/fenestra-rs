use super::*;

#[test]
fn aabb_construction_distinguishes_empty_points_and_invalid_edges() {
    let empty = SpatialAabbV2::empty();
    assert!(empty.is_empty());
    assert_eq!(
        (empty.min_x(), empty.min_y(), empty.max_x(), empty.max_y()),
        (scalar(0), scalar(0), scalar(0), scalar(0))
    );
    let zero_point = aabb(0, 0, 0, 0);
    assert!(!zero_point.is_empty());
    assert_ne!(empty, zero_point);

    let point = aabb(7, 11, 7, 11);
    assert!(!point.is_empty());
    assert_eq!(point, aabb(7, 11, 7, 11));

    let rectangle = aabb(1, 2, 3, 4);
    assert_eq!(rectangle.min_x(), scalar(1));
    assert_eq!(rectangle.min_y(), scalar(2));
    assert_eq!(rectangle.max_x(), scalar(3));
    assert_eq!(rectangle.max_y(), scalar(4));

    assert_eq!(
        SpatialAabbV2::from_edges(scalar(1), scalar(0), scalar(0), scalar(1)),
        None
    );
    assert_eq!(
        SpatialAabbV2::from_edges(scalar(0), scalar(1), scalar(1), scalar(0)),
        None
    );

    let minimum = SpatialScalarV2::MIN_RAW;
    let maximum = SpatialScalarV2::MAX_RAW;
    let invalid_edges = [
        [minimum - 1, 0, 0, 0],
        [0, minimum - 1, 0, 0],
        [0, 0, maximum + 1, 0],
        [0, 0, 0, maximum + 1],
    ];
    for [min_x, min_y, max_x, max_y] in invalid_edges {
        assert_eq!(
            SpatialAabbV2::from_edges(scalar(min_x), scalar(min_y), scalar(max_x), scalar(max_y),),
            None
        );
    }
}

#[test]
fn aabb_intersection_is_closed_and_canonicalizes_disjoint_results() {
    assert_eq!(
        aabb(0, 0, 10, 10).intersection(aabb(5, -5, 15, 5)),
        aabb(5, 0, 10, 5)
    );
    assert_eq!(
        aabb(0, 0, 10, 10).intersection(aabb(10, 10, 20, 20)),
        aabb(10, 10, 10, 10)
    );
    assert_eq!(
        aabb(0, 0, 1, 1).intersection(aabb(2, 2, 3, 3)),
        SpatialAabbV2::empty()
    );
    assert_eq!(
        SpatialAabbV2::empty().intersection(aabb(0, 0, 1, 1)),
        SpatialAabbV2::empty()
    );
}

#[test]
fn transformed_aabb_rounds_exact_extrema_outward() {
    let positive_half = Affine2V2::scale(scalar(SCALE / 2), integer(1));
    assert_eq!(
        positive_half.checked_transform_aabb(aabb(1, 0, 1, 0)),
        Ok(aabb(0, 0, 1, 0))
    );

    let negative_half = Affine2V2::scale(scalar(-SCALE / 2), integer(1));
    assert_eq!(
        negative_half.checked_transform_aabb(aabb(1, 0, 1, 0)),
        Ok(aabb(-1, 0, 0, 0))
    );

    assert_eq!(
        Affine2V2::quarter_turn_clockwise().checked_transform_aabb(aabb(
            SCALE,
            2 * SCALE,
            3 * SCALE,
            5 * SCALE,
        )),
        Ok(aabb(-5 * SCALE, SCALE, -2 * SCALE, 3 * SCALE))
    );
    assert_eq!(
        affine([i64::MAX, 0, 0, 0, 0, 0]).checked_transform_aabb(SpatialAabbV2::empty()),
        Ok(SpatialAabbV2::empty())
    );

    let cancelled = affine([i64::MAX, 0, -i64::MAX, 0, 0, 0]);
    assert_eq!(
        cancelled.checked_transform_aabb(aabb(1, 1, 1, 1)),
        Ok(aabb(0, 0, 0, 0))
    );
}

#[test]
fn transformed_aabb_reports_edges_in_contract_order() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let cases = [
        (
            Affine2V2::scale(scalar(maximum), integer(1)),
            aabb(-2 * SCALE, 0, 0, 0),
            SpatialArithmeticOperationV2::AabbMinX,
        ),
        (
            Affine2V2::scale(integer(1), scalar(maximum)),
            aabb(0, -2 * SCALE, 0, 0),
            SpatialArithmeticOperationV2::AabbMinY,
        ),
        (
            Affine2V2::scale(scalar(maximum), integer(1)),
            aabb(0, 0, 2 * SCALE, 0),
            SpatialArithmeticOperationV2::AabbMaxX,
        ),
        (
            Affine2V2::scale(integer(1), scalar(maximum)),
            aabb(0, 0, 0, 2 * SCALE),
            SpatialArithmeticOperationV2::AabbMaxY,
        ),
    ];

    for (transform, local, expected) in cases {
        assert_eq!(transform.checked_transform_aabb(local), Err(expected));
    }

    let both = Affine2V2::scale(scalar(maximum), scalar(maximum));
    assert_eq!(
        both.checked_transform_aabb(aabb(-2 * SCALE, -2 * SCALE, 0, 0)),
        Err(SpatialArithmeticOperationV2::AabbMinX)
    );
}
