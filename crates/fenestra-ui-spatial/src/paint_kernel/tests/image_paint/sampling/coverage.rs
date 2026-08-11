use super::*;

#[test]
fn cropped_destination_is_half_open_and_maps_all_four_included_corners() {
    let source = source(1, 1, 3, 2);
    let destination = destination(-17, 23, 31, 47);
    with_final_image_paint!(id_image(31, 4, 3), source, destination, 255, paint => {
        for (query, expected) in [
            (point(-17, 23), Some([6, 0, 0, 255])),
            (point(13, 23), Some([8, 0, 0, 255])),
            (point(-17, 69), Some([10, 0, 0, 255])),
            (point(13, 69), Some([12, 0, 0, 255])),
            (point(-18, 23), None),
            (point(-17, 22), None),
            (point(14, 23), None),
            (point(-17, 70), None),
            (point(14, 70), None),
        ] {
            assert_eq!(sample_bytes(sample_image_p6(&paint, query)), expected);
        }
    });
}

#[test]
fn closed_local_bounds_do_not_make_the_destination_far_edges_sampleable() {
    let destination = destination(10, 20, 30, 40);
    with_final_image_paint!(
        raw_image(IMAGE_KEY, 1, 1, [1, 2, 3, 4]),
        source(0, 0, 1, 1),
        destination,
        255,
        paint => {
            assert_eq!(
                sample_bytes(sample_image_p6(&paint, point(40, 60))),
                None
            );
            assert_eq!(paint.local_bounds().max_x(), scalar(40));
            assert_eq!(paint.local_bounds().max_y(), scalar(60));
        }
    );
}
