use super::shape_structure_support::expect_non_dense;
use super::validated_path_support::path;
use super::validated_shape_support::{
    circle_values, expect_content, expect_limit, fixture, fixture_with_paths, limits, outside_high,
    point, polygon, rect_values, validate,
};
use crate::content_diagnostic::{SpatialPathGrammarErrorV2, SpatialShapeErrorV2};
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::{
    SpatialPathVerbFieldV2, SpatialPolygonPointFieldV2, SpatialShapeFieldV2,
};
use crate::path::SpatialPathVerbV2;

#[test]
fn complete_shapes_run_record_major_instead_of_batching_by_variant() {
    let rect_first = fixture(
        vec![
            rect_values(0, 1, 0, 0, -1, 1),
            circle_values(1, 2, outside_high(), 0, 1),
        ],
        Vec::new(),
    );
    expect_content(
        validate(&rect_first, limits(usize::MAX)),
        SpatialContentErrorKindV2::InvalidShape(SpatialShapeErrorV2::NegativeExtent),
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::RectWidth,
        },
    );

    let circle_first = fixture(
        vec![circle_values(0, 1, 0, 0, -1), polygon(1, 2, 0, 3)],
        vec![point(outside_high(), 0), point(2, 0), point(0, 2)],
    );
    expect_content(
        validate(&circle_first, limits(3)),
        SpatialContentErrorKindV2::InvalidShape(SpatialShapeErrorV2::NegativeRadius),
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::CircleRadius,
        },
    );

    let polygon_first = fixture(
        vec![
            polygon(0, 1, 0, 0),
            rect_values(1, 2, outside_high(), 0, 1, 1),
        ],
        Vec::new(),
    );
    expect_content(
        validate(&polygon_first, limits(usize::MAX)),
        SpatialContentErrorKindV2::InvalidShape(SpatialShapeErrorV2::PolygonTooShort),
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::PolygonPointLength,
        },
    );
}

#[test]
fn an_earlier_record_finishes_before_a_later_records_first_scalar() {
    let fixture = fixture(
        vec![
            rect_values(0, 1, 0, 0, 1, outside_high()),
            rect_values(1, 2, outside_high(), 0, 1, 1),
        ],
        Vec::new(),
    );

    expect_content(
        validate(&fixture, limits(usize::MAX)),
        SpatialContentErrorKindV2::ScalarOutOfDomain,
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::RectHeight,
        },
    );
}

#[test]
fn an_earlier_polygon_limit_precedes_a_later_scalar_failure() {
    let fixture = fixture(
        vec![
            polygon(0, 1, 0, 4),
            rect_values(1, 2, outside_high(), 0, 1, 1),
        ],
        vec![point(0, 0), point(4, 0), point(4, 4), point(0, 4)],
    );

    expect_limit(
        validate(&fixture, limits(3)),
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::PolygonPointLength,
        },
        4,
        3,
    );
}

#[test]
fn an_earlier_semantic_failure_precedes_a_later_polygon_limit() {
    let fixture = fixture(
        vec![rect_values(0, 1, 0, 0, -1, 1), polygon(1, 2, 0, 4)],
        vec![point(0, 0), point(4, 0), point(4, 4), point(0, 4)],
    );

    expect_content(
        validate(&fixture, limits(3)),
        SpatialContentErrorKindV2::InvalidShape(SpatialShapeErrorV2::NegativeExtent),
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::RectWidth,
        },
    );
}

#[test]
fn complete_shape_structure_precedes_any_shape_k1_failure() {
    let fixture = fixture(
        vec![
            rect_values(0, 1, outside_high(), 0, 1, 1),
            rect_values(2, 0, 0, 0, -1, -1),
        ],
        Vec::new(),
    );

    expect_non_dense(
        prepare_validated_shapes!(
            &fixture,
            super::local_transform_support::VIEWPORT,
            limits(0)
        ),
        SpatialErrorLocationV2::Shape {
            index: 1,
            field: SpatialShapeFieldV2::Key,
        },
    );
}

#[test]
fn validated_path_k1_precedes_shape_k1() {
    let fixture = fixture_with_paths(
        vec![rect_values(0, 1, outside_high(), 0, 1, 1)],
        Vec::new(),
        vec![path(0, 0, 1)],
        vec![SpatialPathVerbV2::Close],
    );

    expect_content(
        validate(&fixture, limits(usize::MAX)),
        SpatialContentErrorKindV2::InvalidPathGrammar(SpatialPathGrammarErrorV2::FirstNotMove),
        SpatialErrorLocationV2::PathVerb {
            path: 0,
            verb: 0,
            field: SpatialPathVerbFieldV2::Kind,
        },
    );
}

#[test]
fn an_earlier_polygon_point_finishes_before_a_later_polygons_first_point() {
    let fixture = fixture(
        vec![polygon(0, 1, 0, 3), polygon(1, 2, 3, 3)],
        vec![
            point(0, 0),
            point(2, 0),
            point(0, outside_high()),
            point(outside_high(), 0),
            point(12, 0),
            point(10, 2),
        ],
    );

    expect_content(
        validate(&fixture, limits(3)),
        SpatialContentErrorKindV2::ScalarOutOfDomain,
        SpatialErrorLocationV2::PolygonPoint {
            shape: 0,
            point: 2,
            field: SpatialPolygonPointFieldV2::Y,
        },
    );
}
