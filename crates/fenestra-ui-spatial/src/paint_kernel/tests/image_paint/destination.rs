use super::*;

#[test]
fn destination_scalars_complete_x_y_width_height_before_semantics() {
    let low = SpatialScalarV2::MIN_RAW - 1;
    let high = SpatialScalarV2::MAX_RAW + 1;
    with_p4_image!(raw_image(IMAGE_KEY, 4, 3, [0, 0, 0, 0]), proof => {
        for (destination, field) in [
            (destination(low, high, low, high), PaintP5Field::DestinationX),
            (destination(0, high, low, high), PaintP5Field::DestinationY),
            (destination(0, 0, low, high), PaintP5Field::DestinationWidth),
            (destination(0, 0, 1, high), PaintP5Field::DestinationHeight),
        ] {
            expect_p5_error(
                prepare_image_paint_p5(
                    PAINT_INDEX,
                    &proof,
                    valid_source(),
                    destination,
                    73,
                ),
                PaintP5ErrorKind::ScalarOutOfDomain,
                field,
            );
        }

        let alternate_index = PAINT_INDEX + 4;
        expect_p5_error_at(
            prepare_image_paint_p5(
                alternate_index,
                &proof,
                valid_source(),
                destination(0, 0, -1, high),
                73,
            ),
            alternate_index,
            PaintP5ErrorKind::ScalarOutOfDomain,
            PaintP5Field::DestinationHeight,
        );
    });
}

#[test]
fn every_destination_scalar_rejects_both_sides_of_the_canonical_domain() {
    let low = SpatialScalarV2::MIN_RAW - 1;
    let high = SpatialScalarV2::MAX_RAW + 1;
    with_p4_image!(raw_image(IMAGE_KEY, 4, 3, [0, 0, 0, 0]), proof => {
        for (destination, field) in [
            (destination(low, 0, 1, 1), PaintP5Field::DestinationX),
            (destination(high, 0, 1, 1), PaintP5Field::DestinationX),
            (destination(0, low, 1, 1), PaintP5Field::DestinationY),
            (destination(0, high, 1, 1), PaintP5Field::DestinationY),
            (destination(0, 0, low, 1), PaintP5Field::DestinationWidth),
            (destination(0, 0, high, 1), PaintP5Field::DestinationWidth),
            (destination(0, 0, 1, low), PaintP5Field::DestinationHeight),
            (destination(0, 0, 1, high), PaintP5Field::DestinationHeight),
        ] {
            expect_p5_error(
                prepare_image_paint_p5(
                    PAINT_INDEX,
                    &proof,
                    valid_source(),
                    destination,
                    83,
                ),
                PaintP5ErrorKind::ScalarOutOfDomain,
                field,
            );
        }
    });
}

#[test]
fn negative_extents_complete_width_then_height_before_empty_extents() {
    with_p4_image!(raw_image(IMAGE_KEY, 4, 3, [0, 0, 0, 0]), proof => {
        for (width, height, extent, field) in [
            (-1, -1, SpatialExtentV2::Width, PaintP5Field::DestinationWidth),
            (-1, 1, SpatialExtentV2::Width, PaintP5Field::DestinationWidth),
            (1, -1, SpatialExtentV2::Height, PaintP5Field::DestinationHeight),
            (0, -1, SpatialExtentV2::Height, PaintP5Field::DestinationHeight),
        ] {
            expect_p5_error(
                prepare_image_paint_p5(
                    PAINT_INDEX,
                    &proof,
                    valid_source(),
                    destination(0, 0, width, height),
                    91,
                ),
                PaintP5ErrorKind::InvalidImage(
                    PaintP5ImageKind::NegativeDestinationExtent(extent),
                ),
                field,
            );
        }
    });
}

#[test]
fn empty_destination_extents_complete_width_then_height() {
    with_p4_image!(raw_image(IMAGE_KEY, 4, 3, [0, 0, 0, 0]), proof => {
        for (width, height, field) in [
            (0, 0, PaintP5Field::DestinationWidth),
            (0, 1, PaintP5Field::DestinationWidth),
            (1, 0, PaintP5Field::DestinationHeight),
        ] {
            expect_p5_error(
                prepare_image_paint_p5(
                    PAINT_INDEX,
                    &proof,
                    valid_source(),
                    destination(0, 0, width, height),
                    113,
                ),
                PaintP5ErrorKind::InvalidImage(PaintP5ImageKind::EmptyDestination),
                field,
            );
        }
    });
}

#[test]
fn canonical_destination_near_edges_are_accepted_before_bounds_are_derived() {
    with_p4_image!(raw_image(IMAGE_KEY, 4, 3, [0, 0, 0, 0]), proof => {
        assert!(
            prepare_image_paint_p5(
                PAINT_INDEX,
                &proof,
                valid_source(),
                destination(
                    SpatialScalarV2::MIN_RAW,
                    SpatialScalarV2::MAX_RAW,
                    1,
                    1,
                ),
                137,
            )
            .is_ok()
        );
    });
}
