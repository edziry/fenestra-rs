use super::prepared_brush_support::{expect_gradient, gradient, stop, stop_location};
use super::validated_image_support::{blank_image, expect_image, fixture, image, limits, validate};
use crate::content_diagnostic::SpatialGradientErrorV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::{SpatialColorChannelV2, SpatialImageFieldV2};

#[test]
fn an_earlier_zero_extent_beats_a_later_pixel_failure() {
    let fixture = fixture(vec![
        image(0, 0, 0, 0, Vec::new()),
        image(1, 1, 1, 4, vec![0, 0, 2, 1]),
    ]);

    expect_image(
        validate(&fixture, limits(1, 1)),
        crate::content_diagnostic::SpatialImageErrorV2::ZeroExtent,
        super::validated_image_support::image_location(0, SpatialImageFieldV2::Width),
    );
}

#[test]
fn an_earlier_late_pixel_failure_beats_a_later_zero_extent() {
    let fixture = fixture(vec![
        image(0, 1, 1, 4, vec![0, 0, 2, 1]),
        image(1, 0, 0, 0, Vec::new()),
    ]);

    expect_image(
        validate(&fixture, limits(1, 1)),
        crate::content_diagnostic::SpatialImageErrorV2::InvalidPremultipliedPixel,
        SpatialErrorLocationV2::ImagePixel {
            image: 0,
            pixel: 0,
            channel: SpatialColorChannelV2::B,
        },
    );
}

#[test]
fn prepared_brush_failures_precede_image_structure_and_p4() {
    let fixture = super::prepared_brush_support::fixture(
        vec![gradient(0, 0, 4)],
        vec![stop(0), stop(40_000), stop(30_000), stop(u16::MAX)],
    )
    .with_images(vec![blank_image(u32::MAX, 0, 0)]);

    expect_gradient(
        validate(&fixture, limits(0, 0)),
        SpatialGradientErrorV2::DecreasingOffset,
        stop_location(0, 2),
    );
}
