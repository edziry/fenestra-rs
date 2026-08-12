use super::raster_support::*;
use crate::coverage::SpatialFillRuleV2;

#[test]
fn solid_color_is_premultiplied_then_opacity_is_applied_once() {
    use super::super::validated_shape_support::rect_values;

    let snapshot = snapshot(owned_fixture(
        viewport(1, 1),
        root_and_owners(1, 1, 1),
        vec![rect_values(0, 1, 0, 0, S, S)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![solid(0, color(200, 100, 50, 128))],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![fill(1, 0, 0, 0, 128, None, SpatialFillRuleV2::NonZero)],
    ));
    let raster = snapshot
        .rasterize_reference(limits(1))
        .expect("solid reference pixel");
    assert_raster(&raster, 1, 1, &[50, 25, 13, 64]);
}

#[test]
fn gradient_uses_exact_quantization_duplicate_stop_selection_and_rounding() {
    use super::super::validated_shape_support::rect_values;

    let snapshot = snapshot(owned_fixture(
        viewport(1, 1),
        root_and_owners(1, 1, 1),
        vec![rect_values(0, 1, 0, 0, S, S)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![gradient(0, 0, 4, point(0, 0), point(S, 0))],
        vec![
            stop(0, color(0, 0, 0, 255)),
            stop(24_576, color(255, 0, 0, 255)),
            stop(24_576, color(0, 255, 0, 255)),
            stop(u16::MAX, color(0, 0, 255, 255)),
        ],
        Vec::new(),
        Vec::new(),
        vec![fill(1, 0, 0, 0, 255, None, SpatialFillRuleV2::NonZero)],
    ));
    let raster = snapshot
        .rasterize_reference(limits(1))
        .expect("quantized gradient");
    assert_raster(&raster, 1, 1, &[21, 115, 77, 255]);
}

#[test]
fn image_crop_nearest_floor_half_open_channel_order_and_opacity_are_exact() {
    let pixels = vec![
        1, 2, 3, 255, 10, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255,
    ];
    let snapshot = snapshot(owned_fixture(
        viewport(2, 1),
        root_and_owners(1, 2, 1),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![image(0, 4, 1, pixels)],
        Vec::new(),
        vec![image_paint(
            1,
            0,
            0,
            source(1, 0, 2, 1),
            destination(0, 0, 2 * S, S),
            128,
            None,
        )],
    ));
    let raster = snapshot
        .rasterize_reference(limits(2))
        .expect("cropped nearest image");
    assert_raster(&raster, 2, 1, &[5, 10, 15, 128, 20, 25, 30, 128]);
}

#[test]
fn transparent_and_uncovered_image_samples_preserve_transparency() {
    let snapshot = snapshot(owned_fixture(
        viewport(2, 1),
        root_and_owners(1, 2, 1),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![image(0, 1, 1, vec![0, 0, 0, 0])],
        Vec::new(),
        vec![image_paint(
            1,
            0,
            0,
            source(0, 0, 1, 1),
            destination(0, 0, S, S),
            255,
            None,
        )],
    ));
    let raster = snapshot
        .rasterize_reference(limits(2))
        .expect("transparent image");
    assert_raster(&raster, 2, 1, &[0; 8]);
}

#[test]
fn transformed_gradient_samples_in_paint_owner_local_space() {
    use super::super::validated_shape_support::rect_values;
    use super::validator_support::{GeometryRow, PaintRow, validate};

    let source = owned_fixture(
        viewport(1, 1),
        root_and_owners(1, 2, 1),
        vec![rect_values(0, 1, 0, 0, S, S)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![gradient(0, 0, 2, point(0, 0), point(S, 0))],
        vec![
            stop(0, color(255, 0, 0, 255)),
            stop(u16::MAX, color(0, 0, 255, 255)),
        ],
        Vec::new(),
        Vec::new(),
        vec![fill(1, 0, 0, 0, 255, None, SpatialFillRuleV2::NonZero)],
    );
    let (prepared, mut rows) = candidate_case(source);
    let world = [S / 2, 0, 0, S, S / 2, 0];
    let determinant = (S as i128 * S as i128) / 2;
    let mut geometry = GeometryRow::read(rows.geometry[1]);
    geometry.world = world;
    geometry.determinant = determinant;
    geometry.aabb = (false, [S / 2, 0, S + S / 2, S]);
    rows.geometry[1] = geometry.build();
    let mut paint = PaintRow::read(rows.paints[0]);
    paint.world = world;
    paint.determinant = determinant;
    paint.aabb = (false, [S / 2, 0, S, S]);
    rows.paints[0] = paint.build();

    let raster = validate(prepared, &rows)
        .expect("transformed gradient candidate")
        .rasterize_reference(limits(1))
        .expect("owner-local gradient raster");
    assert_raster(&raster, 1, 1, &[64, 0, 64, 128]);
}

#[test]
fn transformed_image_samples_in_paint_owner_local_space() {
    use super::validator_support::{GeometryRow, PaintRow, validate};

    let source = owned_fixture(
        viewport(1, 1),
        root_and_owners(1, 2, 1),
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
            destination(0, 0, S, S),
            255,
            None,
        )],
    );
    let (prepared, mut rows) = candidate_case(source);
    let world = [S / 2, 0, 0, S, S / 2, 0];
    let determinant = (S as i128 * S as i128) / 2;
    let mut geometry = GeometryRow::read(rows.geometry[1]);
    geometry.world = world;
    geometry.determinant = determinant;
    geometry.aabb = (false, [S / 2, 0, 3 * S / 2, S]);
    rows.geometry[1] = geometry.build();
    let mut paint = PaintRow::read(rows.paints[0]);
    paint.world = world;
    paint.determinant = determinant;
    paint.aabb = (false, [S / 2, 0, S, S]);
    rows.paints[0] = paint.build();

    let raster = validate(prepared, &rows)
        .expect("transformed image candidate")
        .rasterize_reference(limits(1))
        .expect("owner-local image raster");
    assert_raster(&raster, 1, 1, &[64, 64, 0, 128]);
}

#[test]
fn mixed_coverage_and_image_rows_follow_ascending_record_order() {
    use super::super::validated_shape_support::rect_values;

    for image_first in [false, true] {
        let solid_paint = fill(1, 0, 0, 0, 255, None, SpatialFillRuleV2::NonZero);
        let later_image = image_paint(
            1,
            1,
            0,
            source(0, 0, 1, 1),
            destination(0, 0, S, S),
            255,
            None,
        );
        let paints = if image_first {
            vec![
                image_paint(
                    1,
                    0,
                    0,
                    source(0, 0, 1, 1),
                    destination(0, 0, S, S),
                    255,
                    None,
                ),
                fill(1, 1, 0, 0, 255, None, SpatialFillRuleV2::NonZero),
            ]
        } else {
            vec![solid_paint, later_image]
        };
        let snapshot = snapshot(owned_fixture(
            viewport(1, 1),
            root_and_owners(1, 1, 1),
            vec![rect_values(0, 1, 0, 0, S, S)],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![solid(0, color(255, 0, 0, 128))],
            Vec::new(),
            vec![image(0, 1, 1, vec![0, 128, 0, 128])],
            Vec::new(),
            paints,
        ));
        let raster = snapshot
            .rasterize_reference(limits(1))
            .expect("mixed-kind painter order");
        let expected = if image_first {
            [128, 64, 0, 192]
        } else {
            [64, 128, 0, 192]
        };
        assert_raster(&raster, 1, 1, &expected);
    }
}

#[test]
fn image_destination_far_edge_is_half_open_at_a_registered_sample() {
    let snapshot = snapshot(owned_fixture(
        viewport(1, 1),
        root_and_owners(1, 1, 1),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![image(0, 1, 1, vec![255, 255, 255, 255])],
        Vec::new(),
        vec![image_paint(
            1,
            0,
            0,
            source(0, 0, 1, 1),
            destination(0, 0, S / 8, S),
            255,
            None,
        )],
    ));
    let raster = snapshot
        .rasterize_reference(limits(1))
        .expect("half-open destination");
    assert_raster(&raster, 1, 1, &[0, 0, 0, 0]);
}
