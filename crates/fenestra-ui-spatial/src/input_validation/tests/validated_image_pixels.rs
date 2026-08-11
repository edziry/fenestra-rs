use super::validated_image_support::{blank_image, expect_image, fixture, image, limits, validate};
use crate::content_diagnostic::SpatialImageErrorV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::SpatialColorChannelV2;

#[test]
fn every_premultiplied_channel_maps_to_its_exact_location() {
    let cases = [
        ([2, 3, 4, 1], SpatialColorChannelV2::R),
        ([1, 3, 4, 1], SpatialColorChannelV2::G),
        ([1, 1, 4, 1], SpatialColorChannelV2::B),
    ];

    for (rgba, channel) in cases {
        let fixture = fixture(vec![blank_image(0, 1, 1), image(1, 1, 1, 4, rgba.to_vec())]);
        expect_image(
            validate(&fixture, limits(1, 2)),
            SpatialImageErrorV2::InvalidPremultipliedPixel,
            SpatialErrorLocationV2::ImagePixel {
                image: 1,
                pixel: 0,
                channel,
            },
        );
    }
}

#[test]
fn pixels_are_checked_row_major_with_r_g_b_channel_priority() {
    let bytes = vec![
        0, 0, 0, 0, // pixel 0
        1, 1, 2, 1, // pixel 1: B
        2, 3, 4, 1, // pixel 2: R, G, and B
        0, 0, 0, 0, // pixel 3
    ];
    let fixture = fixture(vec![blank_image(0, 1, 1), image(1, 2, 2, 8, bytes)]);

    expect_image(
        validate(&fixture, limits(2, 5)),
        SpatialImageErrorV2::InvalidPremultipliedPixel,
        SpatialErrorLocationV2::ImagePixel {
            image: 1,
            pixel: 1,
            channel: SpatialColorChannelV2::B,
        },
    );
}
