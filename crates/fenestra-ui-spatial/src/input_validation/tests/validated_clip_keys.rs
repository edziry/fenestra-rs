use super::validated_clip_support::{
    clip_location, expect_non_dense, limits, root_clip, standard_fixture, validate,
};
use crate::item_field::SpatialClipFieldV2;

#[test]
fn a_bad_first_key_is_not_skipped_as_if_clips_had_a_sentinel() {
    for key in [1, u32::MAX] {
        let fixture = standard_fixture(vec![root_clip(key, 0, u32::MAX)]);

        expect_non_dense(
            validate(&fixture, limits(0)),
            clip_location(0, SpatialClipFieldV2::Key),
        );
    }
}

#[test]
fn the_complete_clip_key_pass_precedes_every_record_failure() {
    for second_key in [0, 2, u32::MAX] {
        let fixture = standard_fixture(vec![
            root_clip(0, 0, u32::MAX),
            root_clip(second_key, u32::MAX, u32::MAX),
        ]);

        expect_non_dense(
            validate(&fixture, limits(0)),
            clip_location(1, SpatialClipFieldV2::Key),
        );
    }
}
