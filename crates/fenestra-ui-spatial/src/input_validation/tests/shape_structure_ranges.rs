use super::check_polygon_range;
use super::shape_structure_support::{
    circle, expect_invalid_range, fixture, fixture_with_paths, limits, path_shape, point, polygon,
    rect, validate,
};
use super::validated_path_support::{line_to, move_to, path};
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::SpatialShapeFieldV2;

#[test]
fn polygon_start_precedes_length_on_the_same_record() {
    let fixture = fixture(vec![polygon(0, 1, 1, u32::MAX)], Vec::new());

    expect_invalid_range(
        validate(&fixture, limits()),
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::PolygonPointStart,
        },
    );
}

#[test]
fn an_out_of_bounds_end_is_attributed_to_polygon_length() {
    let fixture = fixture(vec![polygon(0, 1, 0, 2)], vec![point(0, 0)]);

    expect_invalid_range(
        validate(&fixture, limits()),
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::PolygonPointLength,
        },
    );
}

#[test]
fn polygon_ranges_ignore_other_variants_but_remain_gap_free() {
    for second_start in [1, 3] {
        let fixture = fixture_with_paths(
            vec![
                polygon(0, 1, 0, 2),
                rect(1, 1),
                path_shape(2, 2, 0),
                circle(3, 2),
                polygon(4, 2, second_start, 0),
            ],
            vec![point(0, 0), point(1, 1)],
            vec![path(0, 0, 2)],
            vec![move_to(0, 0), line_to(1, 1)],
        );

        expect_invalid_range(
            validate(&fixture, limits()),
            SpatialErrorLocationV2::Shape {
                index: 4,
                field: SpatialShapeFieldV2::PolygonPointStart,
            },
        );
    }
}

#[test]
fn range_end_addition_is_widened_before_bounds_comparison() {
    let fixture = fixture(
        vec![polygon(0, 1, 0, 1), polygon(1, 1, 1, u32::MAX)],
        vec![point(0, 0)],
    );

    expect_invalid_range(
        validate(&fixture, limits()),
        SpatialErrorLocationV2::Shape {
            index: 1,
            field: SpatialShapeFieldV2::PolygonPointLength,
        },
    );
}

#[test]
fn the_range_helper_preserves_cursors_and_counts_above_u32() {
    let above_u32 = u32::MAX as u128 + 1;

    assert_eq!(
        check_polygon_range(7, u32::MAX as u128, u32::MAX, 1, above_u32),
        Ok(above_u32)
    );
    expect_invalid_range(
        check_polygon_range(7, u32::MAX as u128, u32::MAX, 1, u32::MAX as u128),
        SpatialErrorLocationV2::Shape {
            index: 7,
            field: SpatialShapeFieldV2::PolygonPointLength,
        },
    );
    expect_invalid_range(
        check_polygon_range(9, above_u32, u32::MAX, 0, above_u32),
        SpatialErrorLocationV2::Shape {
            index: 9,
            field: SpatialShapeFieldV2::PolygonPointStart,
        },
    );
}

#[test]
fn trailing_and_ownerless_polygon_points_fail_at_input() {
    let trailing = fixture(vec![polygon(0, 1, 0, 1)], vec![point(0, 0), point(1, 1)]);
    expect_invalid_range(validate(&trailing, limits()), SpatialErrorLocationV2::Input);

    let ownerless = fixture(Vec::new(), vec![point(0, 0)]);
    expect_invalid_range(
        validate(&ownerless, limits()),
        SpatialErrorLocationV2::Input,
    );
}
