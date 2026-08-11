use super::fixture::RawInputFixture;
use super::prepared_brush_support::{
    brush_location, expect_gradient, fixture, gradient, gradient_values, limits, point, stop,
    stop_location, valid_stops, validate,
};
use crate::content_diagnostic::SpatialGradientErrorV2;
use crate::geometry_field::SpatialBrushFieldV2;

#[test]
fn coincident_endpoints_precede_stop_count_and_offset_semantics() {
    let empty = fixture(
        vec![gradient_values(0, 0, 0, point(9, 10), point(9, 10))],
        Vec::new(),
    );
    expect_gradient(
        validate(&empty, limits(8)),
        SpatialGradientErrorV2::CoincidentEndpoints,
        brush_location(0, SpatialBrushFieldV2::GradientEndX),
    );

    let bad_offsets = fixture(
        vec![gradient_values(0, 0, 2, point(9, 10), point(9, 10))],
        vec![stop(1), stop(u16::MAX - 1)],
    );
    expect_gradient(
        validate(&bad_offsets, limits(8)),
        SpatialGradientErrorV2::CoincidentEndpoints,
        brush_location(0, SpatialBrushFieldV2::GradientEndX),
    );
}

#[test]
fn zero_and_one_stop_precede_offset_semantics() {
    let empty = fixture(vec![gradient(0, 0, 0)], Vec::new());
    expect_gradient(
        validate(&empty, limits(8)),
        SpatialGradientErrorV2::TooFewStops,
        brush_location(0, SpatialBrushFieldV2::GradientStopLength),
    );

    let one = fixture(vec![gradient(0, 0, 1)], vec![stop(1_234)]);
    expect_gradient(
        validate(&one, limits(8)),
        SpatialGradientErrorV2::TooFewStops,
        brush_location(0, SpatialBrushFieldV2::GradientStopLength),
    );
}

#[test]
fn first_offset_precedes_last_offset_and_decreasing_order() {
    let fixture = second_gradient(vec![stop(1), stop(40_000), stop(30_000), stop(65_534)]);

    expect_gradient(
        validate(&fixture, limits(8)),
        SpatialGradientErrorV2::FirstOffset,
        stop_location(1, 0),
    );
}

#[test]
fn last_offset_precedes_decreasing_order_and_uses_a_local_ordinal() {
    let fixture = second_gradient(vec![stop(0), stop(40_000), stop(30_000), stop(65_534)]);

    expect_gradient(
        validate(&fixture, limits(8)),
        SpatialGradientErrorV2::LastOffset,
        stop_location(1, 3),
    );
}

#[test]
fn decreasing_offset_names_the_first_later_local_ordinal() {
    let fixture = second_gradient(vec![
        stop(0),
        stop(30_000),
        stop(20_000),
        stop(10_000),
        stop(u16::MAX),
    ]);

    expect_gradient(
        validate(&fixture, limits(8)),
        SpatialGradientErrorV2::DecreasingOffset,
        stop_location(1, 2),
    );
}

fn second_gradient(mut second: Vec<crate::brush::SpatialGradientStopV2>) -> RawInputFixture {
    let mut stops = valid_stops();
    let start = stops.len() as u32;
    let length = second.len() as u32;
    stops.append(&mut second);
    fixture(
        vec![gradient(0, 0, start), gradient(1, start, length)],
        stops,
    )
}
