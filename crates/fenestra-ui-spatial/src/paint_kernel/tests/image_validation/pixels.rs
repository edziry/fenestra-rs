use super::*;

#[test]
fn synthetic_pixel_error_preserves_a_reachable_wide_ordinal() {
    let pixel = u128::from(u32::MAX) + 1;
    let error = test_p4_pixel_error(IMAGE_INDEX, pixel, PaintP4Channel::B);

    assert_eq!(
        error.kind(),
        PaintP4ErrorKind::InvalidImage(PaintP4ImageKind::InvalidPremultipliedPixel)
    );
    assert_eq!(error.location(), pixel_location(pixel, PaintP4Channel::B));
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
}

#[test]
fn pixel_channels_are_checked_in_r_g_b_order() {
    let cases = [
        ([1, 0, 0, 0], PaintP4Channel::R),
        ([2, 3, 4, 1], PaintP4Channel::R),
        ([1, 3, 4, 1], PaintP4Channel::G),
        ([1, 1, 4, 1], PaintP4Channel::B),
    ];
    for (bytes, channel) in cases {
        expect_semantic_rejection(
            &image(1, 1, 4, bytes.to_vec()),
            23,
            IMAGE_EDGE_MAXIMUM,
            IMAGE_PIXELS_MAXIMUM,
            PaintP4ImageKind::InvalidPremultipliedPixel,
            pixel_location(0, channel),
        );
    }
}

#[test]
fn pixels_are_checked_in_row_major_order_before_later_channel_faults() {
    let bytes = vec![
        0, 0, 0, 0, // pixel 0: valid
        0, 0, 2, 1, // pixel 1: invalid B
        2, 0, 0, 1, // pixel 2: invalid R
        1, 1, 1, 1, // pixel 3: valid
    ];
    expect_semantic_rejection(
        &image(2, 2, 8, bytes),
        29,
        IMAGE_EDGE_MAXIMUM,
        IMAGE_PIXELS_MAXIMUM,
        PaintP4ImageKind::InvalidPremultipliedPixel,
        pixel_location(1, PaintP4Channel::B),
    );
}

#[test]
fn a_failure_in_the_last_pixel_does_not_commit_the_cumulative_count() {
    let bytes = vec![
        0, 0, 0, 0, // pixel 0
        1, 1, 1, 1, // pixel 1
        2, 2, 2, 2, // pixel 2
        3, 3, 3, 3, // pixel 3
        4, 4, 4, 4, // pixel 4
        1, 3, 1, 1, // pixel 5: invalid G
    ];
    expect_semantic_rejection(
        &image(3, 2, 12, bytes),
        31,
        IMAGE_EDGE_MAXIMUM,
        IMAGE_PIXELS_MAXIMUM,
        PaintP4ImageKind::InvalidPremultipliedPixel,
        pixel_location(5, PaintP4Channel::G),
    );
}
