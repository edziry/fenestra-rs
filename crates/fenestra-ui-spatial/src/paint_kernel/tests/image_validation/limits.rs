use super::*;

#[test]
fn registered_pixel_total_accepts_equality_and_rejects_one_over_atomically() {
    assert_eq!(IMAGE_PIXELS_MAXIMUM, 4_194_304);
    let image = image(3, 2, 12, vec![0; 24]);
    let mut accepted = IMAGE_PIXELS_MAXIMUM - 6;
    assert!(
        prepare_image_p4(
            &image,
            &mut accepted,
            IMAGE_EDGE_MAXIMUM,
            IMAGE_PIXELS_MAXIMUM,
        )
        .is_ok()
    );
    assert_eq!(accepted, IMAGE_PIXELS_MAXIMUM);

    expect_limit_rejection(
        &image,
        IMAGE_PIXELS_MAXIMUM - 5,
        IMAGE_EDGE_MAXIMUM,
        IMAGE_PIXELS_MAXIMUM,
        PaintP4LimitKind::ImagePixelsTotal,
        image_location(PaintP4Field::Pixel),
        (4_194_305, 4_194_304),
    );
}

#[test]
fn supplied_pixel_maximum_precedes_stride_length_and_pixel_semantics() {
    let invalid_later = image(2, 2, 9, vec![2, 3, 4, 1, 99]);
    expect_limit_rejection(
        &invalid_later,
        0,
        2,
        3,
        PaintP4LimitKind::ImagePixelsTotal,
        image_location(PaintP4Field::Pixel),
        (4, 3),
    );
}

#[test]
fn image_area_widens_before_the_pixel_limit_comparison() {
    let image = image(65_536, 65_536, 262_144, Vec::new());
    expect_limit_rejection(
        &image,
        0,
        65_536,
        u32::MAX as usize,
        PaintP4LimitKind::ImagePixelsTotal,
        image_location(PaintP4Field::Pixel),
        (4_294_967_296, u128::from(u32::MAX)),
    );
}

#[test]
fn cumulative_addition_widens_past_usize_on_every_supported_target() {
    let image = image(1, 1, 4, vec![0; 4]);
    expect_limit_rejection(
        &image,
        usize::MAX,
        1,
        usize::MAX,
        PaintP4LimitKind::ImagePixelsTotal,
        image_location(PaintP4Field::Pixel),
        (usize::MAX as u128 + 1, usize::MAX as u128),
    );
}

#[cfg(target_pointer_width = "64")]
#[test]
fn cumulative_pixel_candidate_widens_beyond_u64_and_usize() {
    let image = image(u32::MAX, u32::MAX, u32::MAX - 3, vec![0; 4]);
    expect_limit_rejection(
        &image,
        8_589_934_591,
        u32::MAX as usize,
        usize::MAX,
        PaintP4LimitKind::ImagePixelsTotal,
        image_location(PaintP4Field::Pixel),
        (18_446_744_073_709_551_616, 18_446_744_073_709_551_615),
    );
}
