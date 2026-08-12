use super::*;

#[test]
fn upscaling_uses_floor_at_half_texel_boundaries_on_both_axes() {
    let scale = SpatialScalarV2::SCALE;
    let destination = destination(7, -9, 4 * scale, 4 * scale);
    with_final_image_paint!(
        id_image(37, 2, 2),
        source(0, 0, 2, 2),
        destination,
        255,
        paint => {
            for (query, expected) in [
                (point(7, -9), [1, 0, 0, 255]),
                (point(7 + scale, -9), [1, 0, 0, 255]),
                (point(7 + 2 * scale, -9), [2, 0, 0, 255]),
                (point(7, -9 + scale), [1, 0, 0, 255]),
                (point(7, -9 + 2 * scale), [3, 0, 0, 255]),
                (point(7 + 4 * scale - 1, -9 + 4 * scale - 1), [4, 0, 0, 255]),
            ] {
                assert_eq!(sample_bytes(sample_image_p6(&paint, query)), Some(expected));
            }
        }
    );
}

#[test]
fn downscaling_skips_source_texels_by_the_registered_floor_ratio() {
    let scale = SpatialScalarV2::SCALE;
    let destination = destination(-11, 13, 2 * scale, 2 * scale);
    with_final_image_paint!(
        id_image(41, 4, 4),
        source(0, 0, 4, 4),
        destination,
        255,
        paint => {
            for (query, expected) in [
                (point(-11 + scale / 2, 13), [2, 0, 0, 255]),
                (point(-11 + scale, 13), [3, 0, 0, 255]),
                (point(-11 + 2 * scale - 1, 13), [4, 0, 0, 255]),
                (point(-11, 13 + scale / 2), [5, 0, 0, 255]),
                (point(-11, 13 + scale), [9, 0, 0, 255]),
                (point(-11, 13 + 2 * scale - 1), [13, 0, 0, 255]),
                (point(-11 + 2 * scale - 1, 13 + 2 * scale - 1), [16, 0, 0, 255]),
            ] {
                assert_eq!(sample_bytes(sample_image_p6(&paint, query)), Some(expected));
            }
        }
    );
}

#[test]
fn fractional_fixed_destination_extent_is_not_rounded_before_mapping() {
    let scale = SpatialScalarV2::SCALE;
    let near_x = 19;
    let near_y = -27;
    with_final_image_paint!(
        id_image(43, 3, 1),
        source(0, 0, 3, 1),
        destination(near_x, near_y, 5 * scale / 2, scale),
        255,
        paint => {
            let query = point(near_x + 3 * scale / 4, near_y);
            assert_eq!(
                sample_bytes(sample_image_p6(&paint, query)),
                Some([1, 0, 0, 255])
            );
        }
    );
}
