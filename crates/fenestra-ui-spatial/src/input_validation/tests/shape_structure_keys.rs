use super::shape_structure_support::{expect_non_dense, fixture, limits, polygon, rect, validate};
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::SpatialShapeFieldV2;

#[test]
fn a_bad_first_key_is_not_skipped_as_if_shapes_had_a_sentinel() {
    let fixture = fixture(vec![polygon(u32::MAX, 0, 1, u32::MAX)], Vec::new());

    expect_non_dense(
        validate(&fixture, limits()),
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::Key,
        },
    );
}

#[test]
fn the_complete_dense_key_pass_precedes_owner_and_variant_failures() {
    for second_key in [0, 2, u32::MAX] {
        let fixture = fixture(
            vec![polygon(0, 0, 1, u32::MAX), rect(second_key, 0)],
            Vec::new(),
        );

        expect_non_dense(
            validate(&fixture, limits()),
            SpatialErrorLocationV2::Shape {
                index: 1,
                field: SpatialShapeFieldV2::Key,
            },
        );
    }
}
