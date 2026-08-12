use super::*;

#[test]
fn destination_far_edges_are_deferred_and_bounds_complete_x_before_y() {
    let maximum = SpatialScalarV2::MAX_RAW;
    with_p4_image!(raw_image(IMAGE_KEY, 4, 3, [0, 0, 0, 0]), proof => {
        let both = expect_p5_success(prepare_image_paint_p5(
            PAINT_INDEX,
            &proof,
            valid_source(),
            destination(maximum, maximum, 1, 1),
            149,
        ));
        expect_p5_error(
            finish_image_paint_bounds_after_item_phase_p5(both),
            PaintP5ErrorKind::LocalBoundsOutOfDomain(SpatialAxisV2::X),
            PaintP5Field::DestinationWidth,
        );

        let y_index = PAINT_INDEX + 5;
        let y_only = expect_p5_success(prepare_image_paint_p5(
            y_index,
            &proof,
            valid_source(),
            destination(maximum - 1, maximum, 1, 1),
            151,
        ));
        expect_p5_error_at(
            finish_image_paint_bounds_after_item_phase_p5(y_only),
            y_index,
            PaintP5ErrorKind::LocalBoundsOutOfDomain(SpatialAxisV2::Y),
            PaintP5Field::DestinationHeight,
        );
    });
}

#[test]
fn canonical_far_edge_equalities_produce_closed_bounds() {
    let maximum = SpatialScalarV2::MAX_RAW;
    with_p4_image!(raw_image(IMAGE_KEY, 4, 3, [0, 0, 0, 0]), proof => {
        let preclip = expect_p5_success(prepare_image_paint_p5(
            PAINT_INDEX,
            &proof,
            valid_source(),
            destination(maximum - SpatialScalarV2::SCALE, maximum - 2, SpatialScalarV2::SCALE, 2),
            163,
        ));
        let final_proof = expect_p5_success(finish_image_paint_bounds_after_item_phase_p5(preclip));
        let expected = SpatialAabbV2::from_edges(
            scalar(maximum - SpatialScalarV2::SCALE),
            scalar(maximum - 2),
            scalar(maximum),
            scalar(maximum),
        )
        .expect("fixture edges are canonical and ordered");
        assert_eq!(final_proof.local_bounds(), expected);
    });
}

#[test]
fn maximum_positive_extents_from_the_minimum_near_edges_end_at_zero() {
    let minimum = SpatialScalarV2::MIN_RAW;
    let maximum = SpatialScalarV2::MAX_RAW;
    with_p4_image!(raw_image(IMAGE_KEY, 4, 3, [0, 0, 0, 0]), proof => {
        let preclip = expect_p5_success(prepare_image_paint_p5(
            PAINT_INDEX,
            &proof,
            valid_source(),
            destination(minimum, minimum, maximum, maximum),
            181,
        ));
        let final_proof = expect_p5_success(finish_image_paint_bounds_after_item_phase_p5(preclip));
        let expected = SpatialAabbV2::from_edges(
            scalar(minimum),
            scalar(minimum),
            scalar(0),
            scalar(0),
        )
        .expect("fixture edges are canonical and ordered");
        assert_eq!(final_proof.local_bounds(), expected);
    });
}

#[test]
fn successful_bounds_retain_distinct_near_far_edges_and_prepared_values() {
    let source = source(1, 1, 3, 2);
    let destination = destination(-17, 23, 31, 47);
    with_p4_image!(raw_image(IMAGE_KEY, 4, 3, [64, 32, 0, 128]), proof => {
        let preclip = expect_p5_success(prepare_image_paint_p5(
            PAINT_INDEX,
            &proof,
            source,
            destination,
            137,
        ));
        let final_proof = expect_p5_success(finish_image_paint_bounds_after_item_phase_p5(preclip));
        let expected = SpatialAabbV2::from_edges(
            scalar(-17),
            scalar(23),
            scalar(14),
            scalar(70),
        )
        .expect("fixture edges are canonical and ordered");
        assert_eq!(final_proof.local_bounds(), expected);
        assert_eq!(final_proof.source(), source);
        assert_eq!(final_proof.destination(), destination);
        assert_eq!(final_proof.opacity(), 137);
        assert_eq!(final_proof.image_width(), 4);
        assert_eq!(final_proof.image_height(), 3);
        assert_eq!(final_proof.image_stride(), 16);
        assert_eq!(final_proof.image_bytes(), &[64, 32, 0, 128].repeat(12));
    });
}
