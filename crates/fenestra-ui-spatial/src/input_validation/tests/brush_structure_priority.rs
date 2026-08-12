use super::brush_structure_support::{
    expect_content, fixture_with_shapes, gradient, limits, outside_high, rect_values, stop,
    validate,
};
use crate::content_diagnostic::SpatialPayloadTableV2;
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::SpatialShapeFieldV2;

#[test]
fn validated_shape_k1_precedes_brush_keys_and_ranges() {
    let fixture = fixture_with_shapes(
        vec![rect_values(0, 1, outside_high(), 0, 1, 1)],
        Vec::new(),
        vec![gradient(u32::MAX, 1, u32::MAX)],
        vec![stop(0)],
    );

    expect_content(
        validate(&fixture, limits()),
        SpatialContentErrorKindV2::ScalarOutOfDomain,
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::RectX,
        },
    );
}

#[test]
fn complete_prior_shape_structure_precedes_brush_structure() {
    let fixture = fixture_with_shapes(
        Vec::new(),
        vec![super::brush_structure_support::point(0, 0)],
        vec![gradient(u32::MAX, 1, u32::MAX)],
        Vec::new(),
    );

    expect_content(
        validate(&fixture, limits()),
        SpatialContentErrorKindV2::InvalidRange(SpatialPayloadTableV2::PolygonPoint),
        SpatialErrorLocationV2::Input,
    );
}
