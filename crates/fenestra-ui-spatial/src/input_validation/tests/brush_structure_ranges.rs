use super::brush_structure_support::{
    expect_invalid_range, fixture, gradient, limits, solid, stop, validate,
};
use super::check_gradient_stop_range;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::SpatialBrushFieldV2;

#[test]
fn gradient_start_precedes_length_on_the_same_record() {
    let fixture = fixture(vec![gradient(0, 1, u32::MAX)], Vec::new());

    expect_invalid_range(
        validate(&fixture, limits()),
        SpatialErrorLocationV2::Brush {
            index: 0,
            field: SpatialBrushFieldV2::GradientStopStart,
        },
    );
}

#[test]
fn an_out_of_bounds_end_is_attributed_to_gradient_length() {
    let fixture = fixture(vec![gradient(0, 0, 2)], vec![stop(0)]);

    expect_invalid_range(
        validate(&fixture, limits()),
        SpatialErrorLocationV2::Brush {
            index: 0,
            field: SpatialBrushFieldV2::GradientStopLength,
        },
    );
}

#[test]
fn gradient_ranges_ignore_solids_but_remain_gap_free() {
    for second_start in [1, 3] {
        let fixture = fixture(
            vec![gradient(0, 0, 2), solid(1), gradient(2, second_start, 0)],
            vec![stop(0), stop(u16::MAX)],
        );

        expect_invalid_range(
            validate(&fixture, limits()),
            SpatialErrorLocationV2::Brush {
                index: 2,
                field: SpatialBrushFieldV2::GradientStopStart,
            },
        );
    }
}

#[test]
fn range_end_addition_is_widened_before_bounds_comparison() {
    let fixture = fixture(
        vec![gradient(0, 0, 1), gradient(1, 1, u32::MAX)],
        vec![stop(0)],
    );

    expect_invalid_range(
        validate(&fixture, limits()),
        SpatialErrorLocationV2::Brush {
            index: 1,
            field: SpatialBrushFieldV2::GradientStopLength,
        },
    );
}

#[test]
fn an_earlier_bad_end_beats_a_later_bad_start() {
    let fixture = fixture(
        vec![gradient(0, 0, 3), gradient(1, 1, u32::MAX)],
        vec![stop(0), stop(u16::MAX)],
    );

    expect_invalid_range(
        validate(&fixture, limits()),
        SpatialErrorLocationV2::Brush {
            index: 0,
            field: SpatialBrushFieldV2::GradientStopLength,
        },
    );
}

#[test]
fn a_later_bad_start_precedes_the_final_trailing_payload_check() {
    let fixture = fixture(
        vec![gradient(0, 0, 1), gradient(1, 2, 0)],
        vec![stop(0), stop(1), stop(u16::MAX)],
    );

    expect_invalid_range(
        validate(&fixture, limits()),
        SpatialErrorLocationV2::Brush {
            index: 1,
            field: SpatialBrushFieldV2::GradientStopStart,
        },
    );
}

#[test]
fn the_range_helper_preserves_cursors_and_counts_above_u32() {
    let above_u32 = u32::MAX as u128 + 1;

    assert_eq!(
        check_gradient_stop_range(7, u32::MAX as u128, u32::MAX, 1, above_u32),
        Ok(above_u32)
    );
    expect_invalid_range(
        check_gradient_stop_range(7, u32::MAX as u128, u32::MAX, 1, u32::MAX as u128),
        SpatialErrorLocationV2::Brush {
            index: 7,
            field: SpatialBrushFieldV2::GradientStopLength,
        },
    );
    expect_invalid_range(
        check_gradient_stop_range(9, above_u32, u32::MAX, 0, above_u32),
        SpatialErrorLocationV2::Brush {
            index: 9,
            field: SpatialBrushFieldV2::GradientStopStart,
        },
    );
}

#[test]
fn trailing_and_ownerless_gradient_stops_fail_at_input() {
    let trailing = fixture(vec![gradient(0, 0, 1)], vec![stop(0), stop(u16::MAX)]);
    expect_invalid_range(validate(&trailing, limits()), SpatialErrorLocationV2::Input);

    let solid_only = fixture(vec![solid(0)], vec![stop(0)]);
    expect_invalid_range(
        validate(&solid_only, limits()),
        SpatialErrorLocationV2::Input,
    );

    let ownerless = fixture(Vec::new(), vec![stop(0)]);
    expect_invalid_range(
        validate(&ownerless, limits()),
        SpatialErrorLocationV2::Input,
    );
}
