use super::*;

#[test]
fn empty_source_extents_complete_before_any_near_or_destination_check() {
    with_p4_image!(raw_image(IMAGE_KEY, 4, 3, [0, 0, 0, 0]), proof => {
        let invalid_destination = destination(
            SpatialScalarV2::MAX_RAW + 1,
            SpatialScalarV2::MAX_RAW + 1,
            SpatialScalarV2::MAX_RAW + 1,
            SpatialScalarV2::MAX_RAW + 1,
        );
        expect_p5_error(
            prepare_image_paint_p5(
                PAINT_INDEX,
                &proof,
                source(u32::MAX, u32::MAX, 0, 0),
                invalid_destination,
                17,
            ),
            PaintP5ErrorKind::InvalidImage(PaintP5ImageKind::EmptySource),
            PaintP5Field::SourceWidth,
        );
        expect_p5_error(
            prepare_image_paint_p5(
                PAINT_INDEX,
                &proof,
                source(0, 0, 0, 1),
                invalid_destination,
                17,
            ),
            PaintP5ErrorKind::InvalidImage(PaintP5ImageKind::EmptySource),
            PaintP5Field::SourceWidth,
        );
        expect_p5_error(
            prepare_image_paint_p5(
                PAINT_INDEX,
                &proof,
                source(u32::MAX, u32::MAX, 1, 0),
                invalid_destination,
                17,
            ),
            PaintP5ErrorKind::InvalidImage(PaintP5ImageKind::EmptySource),
            PaintP5Field::SourceHeight,
        );
    });
}

#[test]
fn source_ranges_scan_x_near_x_far_y_near_then_y_far_with_widened_sums() {
    with_p4_image!(raw_image(IMAGE_KEY, 4, 3, [0, 0, 0, 0]), proof => {
        let invalid_destination = destination(
            SpatialScalarV2::MAX_RAW + 1,
            SpatialScalarV2::MAX_RAW + 1,
            SpatialScalarV2::MAX_RAW + 1,
            SpatialScalarV2::MAX_RAW + 1,
        );
        for (source, field) in [
            (source(4, 3, u32::MAX, u32::MAX), PaintP5Field::SourceX),
            (source(1, 3, u32::MAX, u32::MAX), PaintP5Field::SourceWidth),
            (source(1, 3, 3, u32::MAX), PaintP5Field::SourceY),
            (source(1, 1, 3, u32::MAX), PaintP5Field::SourceHeight),
            (source(1, 0, 4, 1), PaintP5Field::SourceWidth),
            (source(0, 1, 1, 3), PaintP5Field::SourceHeight),
        ] {
            expect_p5_error(
                prepare_image_paint_p5(
                    PAINT_INDEX,
                    &proof,
                    source,
                    invalid_destination,
                    29,
                ),
                PaintP5ErrorKind::InvalidImage(PaintP5ImageKind::SourceOutOfBounds),
                field,
            );
        }
    });
}

#[test]
fn source_near_and_far_equalities_are_distinct_and_far_equality_is_valid() {
    with_p4_image!(raw_image(IMAGE_KEY, 4, 3, [0, 0, 0, 0]), proof => {
        expect_p5_error(
            prepare_image_paint_p5(
                PAINT_INDEX,
                &proof,
                source(4, 0, 1, 1),
                valid_destination(),
                41,
            ),
            PaintP5ErrorKind::InvalidImage(PaintP5ImageKind::SourceOutOfBounds),
            PaintP5Field::SourceX,
        );
        assert!(
            prepare_image_paint_p5(
                PAINT_INDEX,
                &proof,
                source(1, 1, 3, 2),
                valid_destination(),
                41,
            )
            .is_ok()
        );
        assert!(
            prepare_image_paint_p5(
                PAINT_INDEX,
                &proof,
                source(0, 0, 4, 3),
                valid_destination(),
                41,
            )
            .is_ok()
        );
    });
}
