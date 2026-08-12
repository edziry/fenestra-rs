use super::prepared_brush_support::{
    brush_location, expect_content, expect_gradient, expect_invalid_range, expect_non_dense,
    expect_scalar, fixture, fixture_with_shapes, gradient, gradient_values, limits, ordered_stops,
    outside_high, point, rect_values, solid, stop, stop_location, valid_stops, validate,
};
use crate::content_diagnostic::SpatialGradientErrorV2;
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::{SpatialBrushFieldV2, SpatialShapeFieldV2};
use crate::model::SpatialScalarV2;

#[test]
fn complete_brush_keys_precede_any_p2_failure() {
    let outside = SpatialScalarV2::MAX_RAW + 1;
    let fixture = fixture(
        vec![
            gradient_values(0, 0, 2, point(outside, outside), point(outside, outside)),
            solid(0),
        ],
        valid_stops(),
    );

    expect_non_dense(
        validate(&fixture, limits(1)),
        SpatialErrorLocationV2::Brush {
            index: 1,
            field: SpatialBrushFieldV2::Key,
        },
    );
}

#[test]
fn complete_stop_partition_precedes_any_p2_failure() {
    let outside = SpatialScalarV2::MAX_RAW + 1;
    let fixture = fixture(
        vec![gradient_values(
            0,
            1,
            2,
            point(outside, outside),
            point(outside, outside),
        )],
        valid_stops(),
    );

    expect_invalid_range(
        validate(&fixture, limits(1)),
        brush_location(0, SpatialBrushFieldV2::GradientStopStart),
    );
}

#[test]
fn validated_shape_k1_precedes_p2_limits_and_semantics() {
    let fixture = fixture_with_shapes(
        vec![rect_values(0, 1, outside_high(), 0, 1, 1)],
        Vec::new(),
        vec![gradient(0, 0, 3)],
        vec![stop(1); 3],
    );

    expect_content(
        validate(&fixture, limits(2)),
        SpatialContentErrorKindV2::ScalarOutOfDomain,
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::RectX,
        },
    );
}

#[test]
fn an_earlier_brush_semantic_failure_beats_a_later_limit_and_scalar() {
    let outside = SpatialScalarV2::MAX_RAW + 1;
    let mut stops = vec![stop(0), stop(u16::MAX - 1)];
    stops.extend(ordered_stops(4));
    let fixture = fixture(
        vec![
            gradient(0, 0, 2),
            gradient_values(1, 2, 4, point(outside, outside), point(outside, outside)),
        ],
        stops,
    );

    expect_gradient(
        validate(&fixture, limits(3)),
        SpatialGradientErrorV2::LastOffset,
        stop_location(0, 1),
    );
}

#[test]
fn an_earlier_last_offset_beats_a_later_first_offset() {
    let fixture = fixture(
        vec![gradient(0, 0, 2), gradient(1, 2, 2)],
        vec![stop(0), stop(u16::MAX - 1), stop(1), stop(u16::MAX)],
    );

    expect_gradient(
        validate(&fixture, limits(2)),
        SpatialGradientErrorV2::LastOffset,
        stop_location(0, 1),
    );
}

#[test]
fn a_later_brush_restarts_p2_priority_at_its_first_scalar() {
    let outside = SpatialScalarV2::MIN_RAW - 1;
    let mut stops = valid_stops();
    stops.extend(valid_stops());
    let fixture = fixture(
        vec![
            gradient(0, 0, 2),
            gradient_values(1, 2, 2, point(outside, 0), point(1, 1)),
        ],
        stops,
    );

    expect_scalar(
        validate(&fixture, limits(2)),
        1,
        SpatialBrushFieldV2::GradientStartX,
    );
}
