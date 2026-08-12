use super::validated_image_support::{
    blank_image, expect_image, expect_limit, expect_valid, fixture, image, image_location, limits,
    validate,
};
use crate::content_diagnostic::SpatialImageErrorV2;
use crate::geometry_field::SpatialImageFieldV2;
use crate::limits::{REGISTERED_SPATIAL_LIMITS_V2, SpatialLimitKindV2};

#[test]
fn zero_extents_complete_width_then_height_before_edges() {
    let zero_width = fixture(vec![blank_image(0, 1, 1), image(1, 0, 0, 0, Vec::new())]);
    expect_image(
        validate(&zero_width, limits(2, usize::MAX)),
        SpatialImageErrorV2::ZeroExtent,
        image_location(1, SpatialImageFieldV2::Width),
    );

    let zero_height = fixture(vec![blank_image(0, 1, 1), image(1, 3, 0, 0, Vec::new())]);
    expect_image(
        validate(&zero_height, limits(2, usize::MAX)),
        SpatialImageErrorV2::ZeroExtent,
        image_location(1, SpatialImageFieldV2::Height),
    );
}

#[test]
fn edge_limits_precede_the_cumulative_pixel_limit() {
    let fixture = fixture(vec![image(0, 2, 3, 0, Vec::new())]);

    expect_limit(
        validate(&fixture, limits(2, 5)),
        SpatialLimitKindV2::ImageEdge,
        image_location(0, SpatialImageFieldV2::Height),
        3,
        2,
    );
}

#[test]
fn edge_limits_complete_width_then_height_with_exact_evidence() {
    let width = fixture(vec![blank_image(0, 1, 1), image(1, 3, 3, 0, Vec::new())]);
    expect_limit(
        validate(&width, limits(2, usize::MAX)),
        SpatialLimitKindV2::ImageEdge,
        image_location(1, SpatialImageFieldV2::Width),
        3,
        2,
    );

    let height = fixture(vec![blank_image(0, 1, 1), image(1, 2, 3, 0, Vec::new())]);
    expect_limit(
        validate(&height, limits(2, usize::MAX)),
        SpatialLimitKindV2::ImageEdge,
        image_location(1, SpatialImageFieldV2::Height),
        3,
        2,
    );

    let equality = fixture(vec![blank_image(0, 2, 2)]);
    expect_valid(validate(&equality, limits(2, 4)));
}

#[test]
fn caller_edge_limits_are_not_capped_by_the_registered_profile() {
    let registered = REGISTERED_SPATIAL_LIMITS_V2.limit(SpatialLimitKindV2::ImageEdge);
    assert_eq!(registered, 4_096);
    let fixture = fixture(vec![blank_image(0, 4_097, 1)]);

    expect_limit(
        validate(&fixture, limits(registered, 4_097)),
        SpatialLimitKindV2::ImageEdge,
        image_location(0, SpatialImageFieldV2::Width),
        4_097,
        4_096,
    );
    expect_valid(validate(&fixture, limits(4_097, 4_097)));
}

#[cfg(target_pointer_width = "64")]
#[test]
fn caller_edge_maxima_above_u32_are_not_narrowed() {
    let above_u32 = u32::MAX as usize + 1;
    let fixture = fixture(vec![image(0, u32::MAX, 1, 0, Vec::new())]);

    expect_limit(
        validate(&fixture, limits(above_u32, 0)),
        SpatialLimitKindV2::ImagePixelsTotal,
        image_location(0, SpatialImageFieldV2::Pixel),
        u128::from(u32::MAX),
        0,
    );
}
