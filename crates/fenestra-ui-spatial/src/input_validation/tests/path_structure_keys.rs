use super::path_structure_support::{expect_non_dense, fixture, path, permissive_limits, validate};
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::SpatialPathFieldV2;

#[test]
fn a_bad_first_key_is_not_skipped_as_if_it_were_a_sentinel() {
    let fixture = fixture(vec![path(u32::MAX, 1, u32::MAX)], Vec::new());

    expect_non_dense(
        validate(&fixture, permissive_limits()),
        SpatialErrorLocationV2::Path {
            index: 0,
            field: SpatialPathFieldV2::Key,
        },
    );
}

#[test]
fn the_complete_dense_key_pass_precedes_every_range_error() {
    let fixture = fixture(vec![path(0, 1, u32::MAX), path(2, 0, 0)], Vec::new());

    expect_non_dense(
        validate(&fixture, permissive_limits()),
        SpatialErrorLocationV2::Path {
            index: 1,
            field: SpatialPathFieldV2::Key,
        },
    );
}

#[test]
fn duplicate_gap_and_extreme_keys_fail_at_the_first_trusted_ordinal() {
    for key in [0, 2, u32::MAX] {
        let fixture = fixture(vec![path(0, 0, 0), path(key, 0, 0)], Vec::new());

        expect_non_dense(
            validate(&fixture, permissive_limits()),
            SpatialErrorLocationV2::Path {
                index: 1,
                field: SpatialPathFieldV2::Key,
            },
        );
    }
}
