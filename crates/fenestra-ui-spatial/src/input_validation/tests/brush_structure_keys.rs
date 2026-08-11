use super::brush_structure_support::{
    expect_non_dense, fixture, gradient, limits, solid, validate,
};
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::SpatialBrushFieldV2;

#[test]
fn a_bad_first_key_is_not_skipped_as_if_brushes_had_a_sentinel() {
    let fixture = fixture(vec![gradient(u32::MAX, 1, u32::MAX)], Vec::new());

    expect_non_dense(
        validate(&fixture, limits()),
        SpatialErrorLocationV2::Brush {
            index: 0,
            field: SpatialBrushFieldV2::Key,
        },
    );
}

#[test]
fn the_complete_dense_key_pass_precedes_gradient_range_failures() {
    for second_key in [0, 2, u32::MAX] {
        let fixture = fixture(
            vec![gradient(0, 1, u32::MAX), solid(second_key)],
            Vec::new(),
        );

        expect_non_dense(
            validate(&fixture, limits()),
            SpatialErrorLocationV2::Brush {
                index: 1,
                field: SpatialBrushFieldV2::Key,
            },
        );
    }
}
