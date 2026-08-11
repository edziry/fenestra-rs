use super::*;

const WIDE_EDGE: u32 = 131_074;

#[test]
fn horizontal_mapping_widens_before_multiplication_and_adds_source_origin() {
    let mut bytes = vec![0; WIDE_EDGE as usize * 4];
    let last_pixel = bytes.len() - 4;
    bytes[last_pixel..].copy_from_slice(&[64, 32, 7, 128]);
    let image = SpatialImageV2::new(
        SpatialImageKeyV2::new(61),
        WIDE_EDGE,
        1,
        WIDE_EDGE * 4,
        bytes.into_boxed_slice(),
    );
    let maximum = SpatialScalarV2::MAX_RAW;
    assert!(i128::from(maximum - 1) * i128::from(WIDE_EDGE - 1) > i128::from(u64::MAX));
    with_final_image_paint!(
        image,
        source(1, 0, WIDE_EDGE - 1, 1),
        destination(SpatialScalarV2::MIN_RAW, 0, maximum, 1),
        137,
        paint => {
            assert_eq!(
                sample_bytes(sample_image_p6(&paint, point(-1, 0))),
                Some([34, 17, 4, 69])
            );
            assert_eq!(sample_bytes(sample_image_p6(&paint, point(0, 0))), None);
        }
    );
}

#[test]
fn vertical_mapping_widens_before_multiplication_and_uses_row_major_stride() {
    let mut bytes = vec![0; WIDE_EDGE as usize * 4];
    let last_pixel = bytes.len() - 4;
    bytes[last_pixel..].copy_from_slice(&[64, 32, 7, 128]);
    let image = SpatialImageV2::new(
        SpatialImageKeyV2::new(67),
        1,
        WIDE_EDGE,
        4,
        bytes.into_boxed_slice(),
    );
    let maximum = SpatialScalarV2::MAX_RAW;
    assert!(i128::from(maximum - 1) * i128::from(WIDE_EDGE - 1) > i128::from(u64::MAX));
    with_final_image_paint!(
        image,
        source(0, 1, 1, WIDE_EDGE - 1),
        destination(0, SpatialScalarV2::MIN_RAW, 1, maximum),
        137,
        paint => {
            assert_eq!(
                sample_bytes(sample_image_p6(&paint, point(0, -1))),
                Some([34, 17, 4, 69])
            );
            assert_eq!(sample_bytes(sample_image_p6(&paint, point(0, 0))), None);
        }
    );
}
