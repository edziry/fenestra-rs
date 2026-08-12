use super::raster_support::*;
use crate::coverage::SpatialFillRuleV2;

#[test]
fn k4_dispatches_rect_circle_polygon_and_path_and_rejects_aabb_false_positives() {
    use super::super::flattened_path_support::{line_to, move_to, path};
    use super::super::validated_shape_support::{circle_values, path_shape, polygon, rect_values};

    let snapshot = snapshot(owned_fixture(
        viewport(4, 2),
        root_and_owners(1, 4, 2),
        vec![
            rect_values(0, 1, 0, 0, S, S),
            circle_values(1, 1, S + S / 2, S / 2, S / 2),
            polygon(2, 1, 0, 3),
            path_shape(3, 1, 0),
        ],
        vec![point(2 * S, 0), point(3 * S, 0), point(2 * S, S)],
        vec![path(0, 0, 3)],
        vec![move_to(3 * S, 0), line_to(4 * S, 0), line_to(3 * S, S)],
        vec![solid(0, color(255, 255, 255, 255))],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![
            fill(1, 0, 0, 0, 255, None, SpatialFillRuleV2::EvenOdd),
            fill(1, 1, 1, 0, 255, None, SpatialFillRuleV2::NonZero),
            fill(1, 2, 2, 0, 255, None, SpatialFillRuleV2::EvenOdd),
            fill(1, 3, 3, 0, 255, None, SpatialFillRuleV2::NonZero),
        ],
    ));
    let raster = snapshot
        .rasterize_reference(limits(8))
        .expect("all K4 shapes");
    assert_raster(
        &raster,
        4,
        2,
        &[
            255, 255, 255, 255, 191, 191, 191, 191, 159, 159, 159, 159, 159, 159, 159, 159, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
    );
}

#[test]
fn k4_preserves_nonzero_and_even_odd_repeated_winding() {
    use super::super::flattened_path_support::{line_to, move_to, path};
    use super::super::validated_shape_support::path_shape;
    use crate::path::SpatialPathVerbV2;

    let winding = vec![
        move_to(0, 0),
        line_to(S, 0),
        line_to(S, S),
        line_to(0, S),
        SpatialPathVerbV2::Close,
    ];
    let mut verbs = winding.clone();
    verbs.extend(winding);
    let snapshot = snapshot(owned_fixture(
        viewport(1, 1),
        root_and_owners(1, 1, 1),
        vec![path_shape(0, 1, 0)],
        Vec::new(),
        vec![path(0, 0, 10)],
        verbs,
        vec![
            solid(0, color(255, 0, 0, 255)),
            solid(1, color(0, 255, 0, 255)),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![
            fill(1, 0, 0, 0, 255, None, SpatialFillRuleV2::NonZero),
            fill(1, 1, 0, 1, 255, None, SpatialFillRuleV2::EvenOdd),
        ],
    ));
    let raster = snapshot
        .rasterize_reference(limits(1))
        .expect("winding raster");
    assert_raster(&raster, 1, 1, &[255, 0, 0, 255]);
}

#[test]
fn k5_dispatches_all_shapes_and_rejects_interiors_and_expanded_bounds() {
    use super::super::flattened_path_support::{line_to, move_to, path};
    use super::super::validated_shape_support::{circle_values, path_shape, polygon, rect_values};

    let snapshot = snapshot(owned_fixture(
        viewport(7, 1),
        root_and_owners(1, 7, 1),
        vec![
            rect_values(0, 1, 0, S / 2, S, 0),
            circle_values(1, 1, 2 * S + S / 2, S / 2, 0),
            polygon(2, 1, 0, 3),
            path_shape(3, 1, 0),
        ],
        vec![
            point(4 * S, S / 2),
            point(4 * S + S / 2, S / 2),
            point(5 * S, S / 2),
        ],
        vec![path(0, 0, 2)],
        vec![move_to(6 * S, S / 2), line_to(7 * S, S / 2)],
        vec![
            solid(0, color(255, 0, 0, 255)),
            solid(1, color(0, 255, 0, 255)),
            solid(2, color(0, 0, 255, 255)),
            solid(3, color(255, 255, 255, 255)),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![
            stroke(1, 0, 0, S, 0, 255, None),
            stroke(1, 1, 1, S / 2, 1, 255, None),
            stroke(1, 2, 2, S, 2, 255, None),
            stroke(1, 3, 3, S, 3, 255, None),
        ],
    ));
    let raster = snapshot
        .rasterize_reference(limits(7))
        .expect("all K5 shapes");
    assert_raster(
        &raster,
        7,
        1,
        &[
            255, 0, 0, 255, 96, 0, 0, 96, 0, 64, 0, 64, 0, 0, 96, 96, 0, 0, 255, 255, 96, 96, 191,
            191, 255, 255, 255, 255,
        ],
    );
}

#[test]
fn k5_keeps_zero_extent_and_zero_length_geometry_as_round_disks() {
    use super::super::flattened_path_support::{line_to, move_to, path};
    use super::super::validated_shape_support::{path_shape, rect_values};

    let snapshot = snapshot(owned_fixture(
        viewport(2, 1),
        root_and_owners(1, 2, 1),
        vec![rect_values(0, 1, S / 2, S / 2, 0, 0), path_shape(1, 1, 0)],
        Vec::new(),
        vec![path(0, 0, 2)],
        vec![move_to(S + S / 2, S / 2), line_to(S + S / 2, S / 2)],
        vec![solid(0, color(255, 255, 255, 255))],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![
            stroke(1, 0, 0, S, 0, 255, None),
            stroke(1, 1, 1, S, 0, 255, None),
        ],
    ));
    let raster = snapshot
        .rasterize_reference(limits(2))
        .expect("degenerate stroke disks");
    assert_raster(&raster, 2, 1, &[191, 191, 191, 191, 191, 191, 191, 191]);
}
