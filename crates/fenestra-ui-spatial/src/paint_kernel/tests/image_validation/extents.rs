use super::*;

#[test]
fn zero_extents_complete_width_then_height_before_edge_limits() {
    expect_semantic_rejection(
        &image(0, 0, 0, Vec::new()),
        11,
        IMAGE_EDGE_MAXIMUM,
        IMAGE_PIXELS_MAXIMUM,
        PaintP4ImageKind::ZeroExtent,
        image_location(PaintP4Field::Width),
    );
    expect_semantic_rejection(
        &image(0, 1, 0, Vec::new()),
        11,
        IMAGE_EDGE_MAXIMUM,
        IMAGE_PIXELS_MAXIMUM,
        PaintP4ImageKind::ZeroExtent,
        image_location(PaintP4Field::Width),
    );
    expect_semantic_rejection(
        &image(1, 0, 0, Vec::new()),
        11,
        IMAGE_EDGE_MAXIMUM,
        IMAGE_PIXELS_MAXIMUM,
        PaintP4ImageKind::ZeroExtent,
        image_location(PaintP4Field::Height),
    );
    expect_semantic_rejection(
        &image((IMAGE_EDGE_MAXIMUM + 1) as u32, 0, 0, Vec::new()),
        11,
        IMAGE_EDGE_MAXIMUM,
        IMAGE_PIXELS_MAXIMUM,
        PaintP4ImageKind::ZeroExtent,
        image_location(PaintP4Field::Height),
    );
}

#[test]
fn image_edges_complete_width_then_height_and_use_the_supplied_maximum() {
    expect_limit_rejection(
        &image(4_097, 4_097, 0, Vec::new()),
        13,
        IMAGE_EDGE_MAXIMUM,
        IMAGE_PIXELS_MAXIMUM,
        PaintP4LimitKind::ImageEdge,
        image_location(PaintP4Field::Width),
        (4_097, 4_096),
    );
    expect_limit_rejection(
        &image(4_096, 4_097, 0, Vec::new()),
        13,
        IMAGE_EDGE_MAXIMUM,
        IMAGE_PIXELS_MAXIMUM,
        PaintP4LimitKind::ImageEdge,
        image_location(PaintP4Field::Height),
        (4_097, 4_096),
    );
    expect_limit_rejection(
        &image(3, 1, 12, vec![0; 12]),
        0,
        2,
        3,
        PaintP4LimitKind::ImageEdge,
        image_location(PaintP4Field::Width),
        (3, 2),
    );
    expect_limit_rejection(
        &image(1, 3, 4, vec![0; 12]),
        0,
        2,
        3,
        PaintP4LimitKind::ImageEdge,
        image_location(PaintP4Field::Height),
        (3, 2),
    );
}

#[test]
fn registered_edge_equalities_are_valid_on_both_dimensions() {
    assert_eq!(IMAGE_EDGE_MAXIMUM, 4_096);
    for image in [
        image(4_096, 1, 16_384, vec![0; 16_384]),
        image(1, 4_096, 4, vec![0; 16_384]),
    ] {
        let mut accepted = 0;
        assert!(
            prepare_image_p4(
                &image,
                &mut accepted,
                IMAGE_EDGE_MAXIMUM,
                IMAGE_EDGE_MAXIMUM,
            )
            .is_ok()
        );
        assert_eq!(accepted, IMAGE_EDGE_MAXIMUM);
    }

    for (width, height, field) in [
        (4_097, 1, PaintP4Field::Width),
        (1, 4_097, PaintP4Field::Height),
    ] {
        expect_limit_rejection(
            &image(width, height, 0, Vec::new()),
            0,
            IMAGE_EDGE_MAXIMUM,
            IMAGE_PIXELS_MAXIMUM,
            PaintP4LimitKind::ImageEdge,
            image_location(field),
            (4_097, 4_096),
        );
    }
}
