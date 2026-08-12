use super::validated_image_support::{
    blank_image, expect_image, expect_limit, fixture, image, image_location, limits, validate,
};
use crate::content_diagnostic::SpatialImageErrorV2;
use crate::geometry_field::SpatialImageFieldV2;
use crate::limits::SpatialLimitKindV2;

#[test]
fn cumulative_pixels_precede_stride_validation() {
    let fixture = fixture(vec![blank_image(0, 1, 1), image(1, 2, 2, 9, Vec::new())]);

    expect_limit(
        validate(&fixture, limits(2, 4)),
        SpatialLimitKindV2::ImagePixelsTotal,
        image_location(1, SpatialImageFieldV2::Pixel),
        5,
        4,
    );
}

#[test]
fn stride_precedes_byte_length_and_uses_widened_width() {
    let mismatch = fixture(vec![blank_image(0, 1, 1), image(1, 2, 1, 7, Vec::new())]);
    expect_image(
        validate(&mismatch, limits(2, 3)),
        SpatialImageErrorV2::StrideMismatch,
        image_location(1, SpatialImageFieldV2::Stride),
    );

    let widened = fixture(vec![
        blank_image(0, 1, 1),
        image(1, 1_073_741_825, 1, 4, vec![0; 4]),
    ]);
    expect_image(
        validate(&widened, limits(1_073_741_825, 1_073_741_826)),
        SpatialImageErrorV2::StrideMismatch,
        image_location(1, SpatialImageFieldV2::Stride),
    );
}

#[test]
fn byte_length_precedes_pixels_and_uses_widened_product() {
    let mismatch = fixture(vec![
        blank_image(0, 1, 1),
        image(1, 1, 1, 4, vec![2, 3, 4, 1, 99]),
    ]);
    expect_image(
        validate(&mismatch, limits(1, 2)),
        SpatialImageErrorV2::LengthMismatch,
        image_location(1, SpatialImageFieldV2::ByteLength),
    );

    let widened = fixture(vec![
        blank_image(0, 1, 1),
        image(1, 536_870_913, 2, 2_147_483_652, vec![0; 8]),
    ]);
    expect_image(
        validate(&widened, limits(536_870_913, 1_073_741_827)),
        SpatialImageErrorV2::LengthMismatch,
        image_location(1, SpatialImageFieldV2::ByteLength),
    );
}
