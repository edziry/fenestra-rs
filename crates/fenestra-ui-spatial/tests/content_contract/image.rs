use crate::*;

#[test]
fn image_owns_the_exact_box_and_round_trips_unvalidated_metadata() {
    let image = {
        let bytes = vec![255, 1, 2, 0, 9].into_boxed_slice();
        SpatialImageV2::new(SpatialImageKeyV2::new(u32::MAX), 0, u32::MAX, 1, bytes)
    };

    assert_eq!(image.key().get(), u32::MAX);
    assert_eq!(image.width(), 0);
    assert_eq!(image.height(), u32::MAX);
    assert_eq!(image.stride(), 1);
    assert_eq!(image.bytes(), &[255, 1, 2, 0, 9]);
}

#[test]
fn cloned_image_retains_owned_bytes_after_the_source_drops() {
    let image = SpatialImageV2::new(
        SpatialImageKeyV2::new(7),
        1,
        1,
        4,
        vec![8, 9, 10, 11].into_boxed_slice(),
    );
    let clone = image.clone();

    assert_eq!(clone, image);
    drop(image);
    assert_eq!(clone.key().get(), 7);
    assert_eq!(clone.width(), 1);
    assert_eq!(clone.height(), 1);
    assert_eq!(clone.stride(), 4);
    assert_eq!(clone.bytes(), &[8, 9, 10, 11]);
}

#[test]
fn image_accepts_an_empty_owned_byte_sequence() {
    let image = SpatialImageV2::new(
        SpatialImageKeyV2::new(0),
        u32::MAX,
        u32::MAX,
        0,
        Box::default(),
    );
    assert!(image.bytes().is_empty());
}
