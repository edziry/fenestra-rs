use super::*;

#[test]
fn opacity_endpoints_and_intermediate_scale_exact_rgba_bytes_once() {
    let image = image_from_pixels(47, 2, 1, &[[64, 32, 7, 128], [0, 0, 0, 0]]);
    with_p4_image!(image, image_proof => {
        let source = source(0, 0, 2, 1);
        let destination = destination(0, 0, 2 * SpatialScalarV2::SCALE, SpatialScalarV2::SCALE);
        let make_paint = |opacity| {
            let preclip = expect_p5_success(prepare_image_paint_p5(
                PAINT_INDEX,
                &image_proof,
                source,
                destination,
                opacity,
            ));
            expect_p5_success(finish_image_paint_bounds_after_item_phase_p5(preclip))
        };
        let transparent_opacity = make_paint(0);
        let intermediate = make_paint(137);
        let opaque = make_paint(255);

        assert_eq!(
            sample_bytes(sample_image_p6(&transparent_opacity, point(0, 0))),
            Some([0, 0, 0, 0])
        );
        assert_eq!(
            sample_bytes(sample_image_p6(&intermediate, point(0, 0))),
            Some([34, 17, 4, 69])
        );
        assert_eq!(
            sample_bytes(sample_image_p6(&opaque, point(0, 0))),
            Some([64, 32, 7, 128])
        );
        assert_eq!(
            sample_bytes(sample_image_p6(
                &intermediate,
                point(SpatialScalarV2::SCALE, 0),
            )),
            Some([0, 0, 0, 0])
        );
    });
}

#[test]
fn same_size_paints_sample_the_bytes_bound_to_each_distinct_p5_proof() {
    let image_a = raw_image(53, 1, 1, [1, 2, 3, 4]);
    let image_b = raw_image(59, 1, 1, [4, 3, 2, 4]);
    let mut accepted_a = 0;
    let mut accepted_b = 0;
    let proof_a = prepare_image_p4(&image_a, &mut accepted_a, usize::MAX, usize::MAX)
        .expect("image A must satisfy P4");
    let proof_b = prepare_image_p4(&image_b, &mut accepted_b, usize::MAX, usize::MAX)
        .expect("image B must satisfy P4");
    let preclip_a = expect_p5_success(prepare_image_paint_p5(
        PAINT_INDEX,
        &proof_a,
        source(0, 0, 1, 1),
        destination(0, 0, 1, 1),
        255,
    ));
    let paint_a = expect_p5_success(finish_image_paint_bounds_after_item_phase_p5(preclip_a));
    let preclip_b = expect_p5_success(prepare_image_paint_p5(
        PAINT_INDEX,
        &proof_b,
        source(0, 0, 1, 1),
        destination(0, 0, 1, 1),
        255,
    ));
    let paint_b = expect_p5_success(finish_image_paint_bounds_after_item_phase_p5(preclip_b));

    assert_eq!(
        sample_bytes(sample_image_p6(&paint_a, point(0, 0))),
        Some([1, 2, 3, 4])
    );
    assert_eq!(
        sample_bytes(sample_image_p6(&paint_b, point(0, 0))),
        Some([4, 3, 2, 4])
    );
}
