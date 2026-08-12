use super::path_structure_support::{
    closes, expect_invalid_range, fixture, path, permissive_limits, validate,
};
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::SpatialPathFieldV2;

#[test]
fn verb_start_precedes_length_on_the_same_record() {
    let fixture = fixture(vec![path(0, 1, u32::MAX)], Vec::new());

    expect_invalid_range(
        validate(&fixture, permissive_limits()),
        SpatialErrorLocationV2::Path {
            index: 0,
            field: SpatialPathFieldV2::VerbStart,
        },
    );
}

#[test]
fn an_out_of_bounds_end_is_attributed_to_verb_length() {
    let fixture = fixture(vec![path(0, 0, 2)], closes(1));

    expect_invalid_range(
        validate(&fixture, permissive_limits()),
        SpatialErrorLocationV2::Path {
            index: 0,
            field: SpatialPathFieldV2::VerbLength,
        },
    );
}

#[test]
fn ranges_are_gap_free_and_nonoverlapping_in_path_order() {
    for second_start in [0, 2] {
        let fixture = fixture(vec![path(0, 0, 1), path(1, second_start, 0)], closes(1));

        expect_invalid_range(
            validate(&fixture, permissive_limits()),
            SpatialErrorLocationV2::Path {
                index: 1,
                field: SpatialPathFieldV2::VerbStart,
            },
        );
    }
}

#[test]
fn an_earlier_bad_end_precedes_a_later_bad_start() {
    let fixture = fixture(vec![path(0, 0, 2), path(1, u32::MAX, 0)], closes(1));

    expect_invalid_range(
        validate(&fixture, permissive_limits()),
        SpatialErrorLocationV2::Path {
            index: 0,
            field: SpatialPathFieldV2::VerbLength,
        },
    );
}

#[test]
fn range_end_addition_is_widened_before_bounds_comparison() {
    let fixture = fixture(vec![path(0, 0, 1), path(1, 1, u32::MAX)], closes(1));

    expect_invalid_range(
        validate(&fixture, permissive_limits()),
        SpatialErrorLocationV2::Path {
            index: 1,
            field: SpatialPathFieldV2::VerbLength,
        },
    );
}

#[test]
fn trailing_and_ownerless_verbs_fail_at_input() {
    let trailing = fixture(vec![path(0, 0, 1)], closes(2));
    expect_invalid_range(
        validate(&trailing, permissive_limits()),
        SpatialErrorLocationV2::Input,
    );

    let ownerless = fixture(Vec::new(), closes(1));
    expect_invalid_range(
        validate(&ownerless, permissive_limits()),
        SpatialErrorLocationV2::Input,
    );
}
