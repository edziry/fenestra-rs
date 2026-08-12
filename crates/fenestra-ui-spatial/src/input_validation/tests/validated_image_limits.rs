use super::map_image_p4_error_stage;
use super::validated_image_support::{
    blank_image, expect_image, expect_limit, expect_valid, fixture, image, image_location, limits,
    validate,
};
use crate::content_diagnostic::SpatialImageErrorV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::{SpatialColorChannelV2, SpatialImageFieldV2};
use crate::limits::{REGISTERED_SPATIAL_LIMITS_V2, SpatialLimitKindV2};
use crate::paint_kernel::{PaintP4Channel, prepare_image_p4, test_p4_pixel_error};

#[test]
fn pixel_limit_is_cumulative_and_commits_only_complete_images() {
    let fixture = fixture(vec![blank_image(0, 1, 2), blank_image(1, 2, 2)]);

    expect_limit(
        validate(&fixture, limits(2, 5)),
        SpatialLimitKindV2::ImagePixelsTotal,
        image_location(1, SpatialImageFieldV2::Pixel),
        6,
        5,
    );
    expect_valid(validate(&fixture, limits(2, 6)));
}

#[test]
fn pixel_area_is_widened_before_the_cumulative_limit_check() {
    let fixture = fixture(vec![image(0, 65_536, 65_536, 0, Vec::new())]);

    expect_limit(
        validate(&fixture, limits(65_536, u32::MAX as usize)),
        SpatialLimitKindV2::ImagePixelsTotal,
        image_location(0, SpatialImageFieldV2::Pixel),
        4_294_967_296,
        u128::from(u32::MAX),
    );
}

#[test]
fn caller_pixel_limits_are_not_capped_by_the_registered_profile() {
    let registered = REGISTERED_SPATIAL_LIMITS_V2.limit(SpatialLimitKindV2::ImagePixelsTotal);
    assert_eq!(registered, 4_194_304);
    let fixture = fixture(vec![blank_image(0, 4_096, 1_024), blank_image(1, 1, 1)]);

    expect_limit(
        validate(&fixture, limits(4_096, registered)),
        SpatialLimitKindV2::ImagePixelsTotal,
        image_location(1, SpatialImageFieldV2::Pixel),
        4_194_305,
        4_194_304,
    );
    expect_valid(validate(&fixture, limits(4_096, registered + 1)));
}

#[test]
fn mapper_preserves_real_p4_evidence_above_usize() {
    let image = image(7, 1, 1, 4, vec![0; 4]);
    let mut accepted = usize::MAX;
    let error = match prepare_image_p4(&image, &mut accepted, 1, usize::MAX) {
        Ok(_) => panic!("expected real P4 cumulative limit failure"),
        Err(error) => error,
    };
    assert_eq!(accepted, usize::MAX);

    expect_limit::<()>(
        Err(map_image_p4_error_stage(error)),
        SpatialLimitKindV2::ImagePixelsTotal,
        image_location(7, SpatialImageFieldV2::Pixel),
        usize::MAX as u128 + 1,
        usize::MAX as u128,
    );
}

#[test]
fn mapper_preserves_a_reachable_pixel_ordinal_above_u32() {
    let pixel = u128::from(u32::MAX) + 1;
    let error = test_p4_pixel_error(u32::MAX, pixel, PaintP4Channel::B);

    expect_image::<()>(
        Err(map_image_p4_error_stage(error)),
        SpatialImageErrorV2::InvalidPremultipliedPixel,
        SpatialErrorLocationV2::ImagePixel {
            image: u32::MAX,
            pixel,
            channel: SpatialColorChannelV2::B,
        },
    );
}
