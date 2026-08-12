use super::*;

#[test]
fn stride_must_equal_width_times_four_before_length_is_checked() {
    for (stride, bytes) in [(7, vec![0; 7]), (9, Vec::new())] {
        expect_semantic_rejection(
            &image(2, 1, stride, bytes),
            17,
            IMAGE_EDGE_MAXIMUM,
            IMAGE_PIXELS_MAXIMUM,
            PaintP4ImageKind::StrideMismatch,
            image_location(PaintP4Field::Stride),
        );
    }
}

#[test]
fn stride_derivation_widens_before_comparing_the_raw_u32() {
    expect_semantic_rejection(
        &image(1_073_741_825, 1, 4, vec![0; 4]),
        0,
        1_073_741_825,
        1_073_741_825,
        PaintP4ImageKind::StrideMismatch,
        image_location(PaintP4Field::Stride),
    );
}

#[test]
fn byte_length_rejects_empty_and_extra_storage_before_scanning_pixels() {
    for bytes in [Vec::new(), vec![2, 3, 4, 1, 99]] {
        expect_semantic_rejection(
            &image(1, 1, 4, bytes),
            19,
            IMAGE_EDGE_MAXIMUM,
            IMAGE_PIXELS_MAXIMUM,
            PaintP4ImageKind::LengthMismatch,
            image_location(PaintP4Field::ByteLength),
        );
    }
}

#[test]
fn byte_length_derivation_widens_stride_times_height() {
    expect_semantic_rejection(
        &image(536_870_913, 2, 2_147_483_652, vec![0; 8]),
        0,
        536_870_913,
        1_073_741_826,
        PaintP4ImageKind::LengthMismatch,
        image_location(PaintP4Field::ByteLength),
    );
}
