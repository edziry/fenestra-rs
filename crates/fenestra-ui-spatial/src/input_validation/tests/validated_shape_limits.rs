use super::map_shape_k1_error_stage;
use super::validated_shape_support::{
    expect_content, expect_limit, expect_valid, fixture, limits, outside_high, point, polygon,
    rect, triangle, validate,
};
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::{SpatialPolygonPointFieldV2, SpatialShapeFieldV2};
use crate::geometry_kernel::{
    GeometryK1Error, GeometryK1Field, GeometryK1LimitKind, GeometryK1Location,
};
use crate::limits::{REGISTERED_SPATIAL_LIMITS_V2, SpatialLimitKindV2};

#[test]
fn polygon_limit_is_per_shape_instead_of_cumulative() {
    let mut points = triangle(0).to_vec();
    points.extend_from_slice(&triangle(10));
    let fixture = fixture(vec![polygon(0, 1, 0, 3), polygon(1, 2, 3, 3)], points);

    expect_valid(validate(&fixture, limits(3)));
}

#[test]
fn custom_polygon_limit_reports_the_current_shape_and_full_count() {
    let mut points = triangle(0).to_vec();
    points.extend_from_slice(&[point(10, 10), point(14, 10), point(14, 14), point(10, 14)]);
    let target_fixture = fixture(vec![polygon(0, 1, 0, 3), polygon(1, 2, 3, 4)], points);

    expect_limit(
        validate(&target_fixture, limits(3)),
        SpatialErrorLocationV2::Shape {
            index: 1,
            field: SpatialShapeFieldV2::PolygonPointLength,
        },
        4,
        3,
    );
    let full_count = fixture(
        vec![rect(0, 1), polygon(1, 2, 0, 4)],
        vec![point(10, 10), point(14, 10), point(14, 14), point(10, 14)],
    );
    expect_limit(
        validate(&full_count, limits(1)),
        SpatialErrorLocationV2::Shape {
            index: 1,
            field: SpatialShapeFieldV2::PolygonPointLength,
        },
        4,
        1,
    );
    expect_valid(validate(&target_fixture, limits(4)));
}

#[test]
fn polygon_scalars_precede_limit_and_limit_precedes_semantics() {
    let scalar_fixture = fixture(
        vec![polygon(0, 1, 0, 4)],
        vec![
            point(0, 0),
            point(4, 0),
            point(outside_high(), 4),
            point(0, 4),
        ],
    );
    expect_content(
        validate(&scalar_fixture, limits(3)),
        SpatialContentErrorKindV2::ScalarOutOfDomain,
        SpatialErrorLocationV2::PolygonPoint {
            shape: 0,
            point: 2,
            field: SpatialPolygonPointFieldV2::X,
        },
    );

    let repeated = fixture(
        vec![polygon(0, 1, 0, 4)],
        vec![point(0, 0), point(4, 0), point(4, 4), point(0, 0)],
    );
    expect_limit(
        validate(&repeated, limits(3)),
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::PolygonPointLength,
        },
        4,
        3,
    );

    let short = fixture(vec![polygon(0, 1, 0, 2)], vec![point(0, 0), point(1, 1)]);
    expect_limit(
        validate(&short, limits(1)),
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::PolygonPointLength,
        },
        2,
        1,
    );
}

#[test]
fn caller_polygon_limit_is_not_replaced_by_the_registered_profile() {
    let registered = REGISTERED_SPATIAL_LIMITS_V2.limit(SpatialLimitKindV2::PolygonPointsPerShape);
    assert_eq!(registered, 256);

    let edge_points = unique_collinear_points(registered);
    let edge = fixture(vec![polygon(0, 1, 0, registered as u32)], edge_points);
    expect_valid(validate(&edge, limits(registered)));

    let one_over = registered + 1;
    let over = fixture(
        vec![polygon(0, 1, 0, one_over as u32)],
        unique_collinear_points(one_over),
    );
    expect_limit(
        validate(&over, limits(registered)),
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::PolygonPointLength,
        },
        one_over as u128,
        registered as u128,
    );
    expect_valid(validate(&over, limits(one_over)));
}

#[test]
fn mapper_preserves_representable_usize_limit_evidence_as_u128() {
    let observed = usize::MAX as u128;
    let maximum = (usize::MAX - 1) as u128;
    let error = GeometryK1Error::limit(
        GeometryK1LimitKind::PolygonPointsPerShape,
        GeometryK1Location::Shape {
            index: 7,
            field: GeometryK1Field::PolygonPointLength,
        },
        observed,
        maximum,
    );
    let mapped = map_shape_k1_error_stage(error);

    expect_limit(
        Err::<(), _>(mapped),
        SpatialErrorLocationV2::Shape {
            index: 7,
            field: SpatialShapeFieldV2::PolygonPointLength,
        },
        observed,
        maximum,
    );
}

fn unique_collinear_points(count: usize) -> Vec<crate::model::SpatialPointV2> {
    (0..count).map(|index| point(index as i64, 0)).collect()
}
