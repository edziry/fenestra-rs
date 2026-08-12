use super::validated_image_support::{
    expect_non_dense, fixture, image, image_location, limits, validate,
};
use crate::geometry_field::SpatialImageFieldV2;

#[test]
fn a_bad_first_key_is_not_skipped_as_if_images_had_a_sentinel() {
    let fixture = fixture(vec![image(u32::MAX, 0, 0, u32::MAX, Vec::new())]);

    expect_non_dense(
        validate(&fixture, limits(0, 0)),
        image_location(0, SpatialImageFieldV2::Key),
    );
}

#[test]
fn the_complete_image_key_pass_precedes_any_p4_failure() {
    for second_key in [0, 2, u32::MAX] {
        let fixture = fixture(vec![
            image(0, 0, 0, u32::MAX, Vec::new()),
            image(second_key, 0, 0, u32::MAX, Vec::new()),
        ]);

        expect_non_dense(
            validate(&fixture, limits(0, 0)),
            image_location(1, SpatialImageFieldV2::Key),
        );
    }
}
