use super::*;

#[test]
fn prepared_paint_borrows_the_image_without_borrowing_the_p4_proof() {
    let image = raw_image(5, 2, 2, [64, 32, 0, 128]);
    let preclip = {
        let mut accepted = 0;
        let proof = prepare_image_p4(&image, &mut accepted, usize::MAX, usize::MAX)
            .expect("image must satisfy P4");
        assert_eq!(accepted, 4);
        expect_p5_success(prepare_image_paint_p5(
            PAINT_INDEX,
            &proof,
            source(0, 0, 2, 2),
            valid_destination(),
            137,
        ))
    };

    let prepared = expect_p5_success(finish_image_paint_bounds_after_item_phase_p5(preclip));
    assert_eq!(prepared.image_bytes(), image.bytes());
    assert_eq!(prepared.opacity(), 137);
}

#[test]
fn prepared_paints_remain_bound_to_their_exact_reusable_p4_images() {
    let image_a = raw_image(7, 4, 3, [64, 32, 0, 128]);
    let image_b = raw_image(11, 2, 2, [1, 2, 3, 3]);
    let image_c = raw_image(17, 4, 3, [3, 2, 1, 3]);
    let mut accepted_a = 0;
    let mut accepted_b = 0;
    let mut accepted_c = 0;
    let proof_a = prepare_image_p4(&image_a, &mut accepted_a, usize::MAX, usize::MAX)
        .expect("image A must satisfy P4");
    let proof_b = prepare_image_p4(&image_b, &mut accepted_b, usize::MAX, usize::MAX)
        .expect("image B must satisfy P4");
    let proof_c = prepare_image_p4(&image_c, &mut accepted_c, usize::MAX, usize::MAX)
        .expect("image C must satisfy P4");
    assert_eq!((accepted_a, accepted_b, accepted_c), (12, 4, 12));

    let source_a = source(2, 1, 2, 2);
    let first_a = expect_p5_success(prepare_image_paint_p5(
        PAINT_INDEX,
        &proof_a,
        source_a,
        valid_destination(),
        0,
    ));
    let mismatched_index = PAINT_INDEX + 9;
    expect_p5_error_at(
        prepare_image_paint_p5(mismatched_index, &proof_b, source_a, valid_destination(), 0),
        mismatched_index,
        PaintP5ErrorKind::InvalidImage(PaintP5ImageKind::SourceOutOfBounds),
        PaintP5Field::SourceX,
    );

    let second_a = expect_p5_success(prepare_image_paint_p5(
        PAINT_INDEX + 1,
        &proof_a,
        source(0, 0, 1, 1),
        destination(10, 20, 30, 40),
        137,
    ));
    let prepared_b = expect_p5_success(prepare_image_paint_p5(
        PAINT_INDEX + 2,
        &proof_b,
        source(0, 0, 2, 2),
        destination(-10, -20, 50, 60),
        255,
    ));
    let prepared_c = expect_p5_success(prepare_image_paint_p5(
        PAINT_INDEX + 3,
        &proof_c,
        source_a,
        destination(5, 6, 7, 8),
        211,
    ));

    let first_a = expect_p5_success(finish_image_paint_bounds_after_item_phase_p5(first_a));
    let second_a = expect_p5_success(finish_image_paint_bounds_after_item_phase_p5(second_a));
    let final_b = expect_p5_success(finish_image_paint_bounds_after_item_phase_p5(prepared_b));
    let final_c = expect_p5_success(finish_image_paint_bounds_after_item_phase_p5(prepared_c));

    assert_eq!(first_a.source(), source_a);
    assert_eq!(first_a.opacity(), 0);
    assert_eq!(second_a.opacity(), 137);
    assert_eq!(final_b.opacity(), 255);
    assert_eq!(final_c.opacity(), 211);
    assert_eq!((first_a.image_width(), first_a.image_height()), (4, 3));
    assert_eq!((second_a.image_width(), second_a.image_height()), (4, 3));
    assert_eq!((final_b.image_width(), final_b.image_height()), (2, 2));
    assert_eq!((final_c.image_width(), final_c.image_height()), (4, 3));
    assert_eq!(first_a.image_bytes(), image_a.bytes());
    assert_eq!(second_a.image_bytes(), image_a.bytes());
    assert_eq!(final_b.image_bytes(), image_b.bytes());
    assert_eq!(final_c.image_bytes(), image_c.bytes());
}
