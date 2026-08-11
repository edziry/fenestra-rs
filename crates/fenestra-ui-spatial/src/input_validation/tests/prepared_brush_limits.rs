use super::prepared_brush_support::{
    expect_limit, expect_valid, fixture, gradient, gradient_values, limits, ordered_stops, point,
    registered_limits, solid, validate,
};
use crate::limits::{REGISTERED_SPATIAL_LIMITS_V2, SpatialLimitKindV2};
use crate::model::SpatialScalarV2;

#[test]
fn registered_per_brush_limit_accepts_32_and_rejects_33() {
    let maximum = REGISTERED_SPATIAL_LIMITS_V2.limit(SpatialLimitKindV2::GradientStopsPerBrush);
    assert_eq!(maximum, 32);

    let accepted = fixture(vec![gradient(0, 0, maximum as u32)], ordered_stops(maximum));
    expect_valid(validate(&accepted, registered_limits()));

    let rejected = fixture(
        vec![gradient(0, 0, (maximum + 1) as u32)],
        ordered_stops(maximum + 1),
    );
    expect_limit(
        validate(&rejected, registered_limits()),
        0,
        (maximum + 1) as u128,
        maximum as u128,
    );

    let caller_accepts = fixture(
        vec![gradient(0, 0, (maximum + 1) as u32)],
        ordered_stops(maximum + 1),
    );
    expect_valid(validate(&caller_accepts, limits(maximum + 1)));
}

#[test]
fn caller_maximum_accepts_equality_and_is_not_replaced_by_the_profile() {
    let accepted = fixture(vec![gradient(0, 0, 3)], ordered_stops(3));
    expect_valid(validate(&accepted, limits(3)));

    let rejected = fixture(vec![gradient(0, 0, 4)], ordered_stops(4));
    expect_limit(validate(&rejected, limits(3)), 0, 4, 3);

    let profile_would_accept = fixture(vec![gradient(0, 0, 2)], ordered_stops(2));
    expect_limit(validate(&profile_would_accept, limits(1)), 0, 2, 1);
}

#[test]
fn the_limit_is_per_brush_instead_of_a_cumulative_stop_limit() {
    let mut stops = ordered_stops(3);
    stops.extend(ordered_stops(3));
    let accepted = fixture(vec![gradient(0, 0, 3), solid(1), gradient(2, 3, 3)], stops);
    expect_valid(validate(&accepted, limits(3)));

    let mut stops = ordered_stops(3);
    stops.extend(ordered_stops(4));
    let rejected = fixture(vec![gradient(0, 0, 3), solid(1), gradient(2, 3, 4)], stops);
    expect_limit(validate(&rejected, limits(3)), 2, 4, 3);
}

#[test]
fn per_brush_limit_precedes_all_scalar_and_gradient_semantics() {
    let outside = SpatialScalarV2::MAX_RAW + 1;
    let invalid = point(outside, outside);
    let fixture = fixture(
        vec![gradient_values(0, 0, 3, invalid, invalid)],
        vec![super::prepared_brush_support::stop(1); 3],
    );

    expect_limit(validate(&fixture, limits(2)), 0, 3, 2);
}

#[test]
fn an_earlier_brush_limit_precedes_a_later_brush_scalar_failure() {
    let outside = SpatialScalarV2::MAX_RAW + 1;
    let mut stops = ordered_stops(3);
    stops.extend(ordered_stops(2));
    let fixture = fixture(
        vec![
            gradient(0, 0, 3),
            gradient_values(1, 3, 2, point(outside, 0), point(1, 1)),
        ],
        stops,
    );

    expect_limit(validate(&fixture, limits(2)), 0, 3, 2);
}
