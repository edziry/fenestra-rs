use super::super::flattened_path_support::{line_to, move_to, path};
use super::super::validated_clip_support::root_clip;
use super::super::validated_shape_support::{circle_values, path_shape, polygon, rect_values};
use super::super::world_aabb_support::owner_node;
use super::super::world_transform_support::{root, transform};
use super::raster_pixel_constant::{RULE, assert_sampling};
use super::raster_support::*;
use crate::path::SpatialPathVerbV2;

#[test]
fn accepted_fractional_projection_keeps_full_sampling_when_authored_input_is_integer() {
    use super::validator_support::{GeometryRow, PaintRow, validate};

    let source = owned_fixture(
        viewport(2, 1),
        root_and_owners(1, 2, 1),
        vec![rect_values(0, 1, 0, 0, S, S)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![solid(0, color(255, 255, 255, 255))],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![fill(1, 0, 0, 0, 255, None, RULE)],
    );
    let (prepared, mut rows) = candidate_case(source);
    let mut geometry = GeometryRow::read(rows.geometry[1]);
    geometry.world = [S, 0, 0, S, S / 4, 0];
    geometry.aabb = (false, [S / 4, 0, 9 * S / 4, S]);
    rows.geometry[1] = geometry.build();
    let mut paint = PaintRow::read(rows.paints[0]);
    paint.world = geometry.world;
    paint.aabb = (false, [S / 4, 0, 5 * S / 4, S]);
    rows.paints[0] = paint.build();
    let snapshot = validate(prepared, &rows).unwrap();
    assert_eq!(
        assert_sampling(&snapshot, false).bytes(),
        &[191, 191, 191, 191, 64, 64, 64, 64]
    );
}

#[test]
fn fractional_rectangles_circles_and_paths_keep_full_sampling() {
    for shape in [
        rect_values(0, 1, S / 4, 0, S, S),
        rect_values(0, 1, 0, 0, S / 4, S),
        circle_values(0, 1, S, S, S),
        path_shape(0, 1, 0),
        polygon(0, 1, 0, 3),
    ] {
        let polygon_points = if matches!(
            shape.geometry(),
            crate::shape::SpatialShapeGeometryV2::Polygon { .. }
        ) {
            vec![point(0, 0), point(2 * S, 0), point(0, 2 * S)]
        } else {
            Vec::new()
        };
        let snapshot = snapshot(owned_fixture(
            viewport(2, 2),
            root_and_owners(1, 2, 2),
            vec![shape],
            polygon_points,
            vec![path(0, 0, 4)],
            vec![
                move_to(0, 0),
                line_to(2 * S, 0),
                line_to(0, 2 * S),
                SpatialPathVerbV2::Close,
            ],
            vec![solid(0, color(255, 255, 255, 255))],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![fill(1, 0, 0, 0, 255, None, RULE)],
        ));
        assert_sampling(&snapshot, false);
    }
}

#[test]
fn strokes_and_gradients_keep_full_sampling_even_with_integer_bounds() {
    for paint in [
        fill(1, 0, 0, 0, 255, None, RULE),
        stroke(1, 0, 0, S, 1, 255, None),
    ] {
        let snapshot = snapshot(owned_fixture(
            viewport(2, 2),
            root_and_owners(1, 2, 2),
            vec![rect_values(0, 1, 0, 0, 2 * S, 2 * S)],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![
                gradient(0, 0, 2, point(0, 0), point(2 * S, 0)),
                solid(1, color(255, 255, 255, 255)),
            ],
            vec![
                stop(0, color(255, 0, 0, 255)),
                stop(u16::MAX, color(0, 0, 255, 255)),
            ],
            Vec::new(),
            Vec::new(),
            vec![paint],
        ));
        assert_sampling(&snapshot, false);
    }
}

#[test]
fn scaled_rotated_and_fractionally_translated_rectangles_keep_full_sampling() {
    for matrix in [
        [S, 0, 0, S, S / 4, 0],
        [2 * S, 0, 0, S, 0, 0],
        [0, S, -S, 0, 2 * S, 0],
        [S, S / 4, 0, S, 0, 0],
    ] {
        let snapshot = snapshot(owned_fixture(
            viewport(2, 2),
            vec![root(), owner_node(1, transform(matrix, 0, 0), 2, 2)],
            vec![rect_values(0, 1, 0, 0, S, S)],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![solid(0, color(255, 255, 255, 255))],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![fill(1, 0, 0, 0, 255, None, RULE)],
        ));
        assert_sampling(&snapshot, false);
    }
}

#[test]
fn scaled_or_fractionally_placed_images_keep_full_sampling() {
    for destination in [destination(0, 0, 3 * S, S), destination(S / 4, 0, 2 * S, S)] {
        let snapshot = snapshot(owned_fixture(
            viewport(3, 1),
            root_and_owners(1, 3, 1),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![image(0, 2, 1, vec![255, 0, 0, 255, 0, 255, 0, 255])],
            Vec::new(),
            vec![image_paint(
                1,
                0,
                0,
                source(0, 0, 2, 1),
                destination,
                255,
                None,
            )],
        ));
        assert_sampling(&snapshot, false);
    }
}

#[test]
fn fractional_and_curved_clips_keep_full_sampling_for_integer_paints() {
    for clip_shape in [
        rect_values(1, 1, 0, 0, S / 4, S),
        circle_values(1, 1, S, S, S),
    ] {
        let snapshot = snapshot(owned_fixture(
            viewport(2, 2),
            root_and_owners(1, 2, 2),
            vec![rect_values(0, 1, 0, 0, 2 * S, 2 * S), clip_shape],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![solid(0, color(255, 255, 255, 255))],
            Vec::new(),
            Vec::new(),
            vec![root_clip(0, 1, 1)],
            vec![fill(1, 0, 0, 0, 255, Some(0), RULE)],
        ));
        assert_sampling(&snapshot, false);
    }
}
