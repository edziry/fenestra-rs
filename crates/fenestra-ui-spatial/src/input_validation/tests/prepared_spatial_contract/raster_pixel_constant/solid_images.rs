use super::super::super::validated_clip_support::root_clip;
use super::super::super::validated_shape_support::rect_values;
use super::super::super::world_aabb_support::owner_node;
use super::super::super::world_transform_support::{root, transform};
use super::super::raster_support::*;
use super::{RULE, assert_sampling};
use crate::model::SpatialScalarV2;

#[test]
fn scaled_single_texel_images_keep_premultiplied_color_and_all_sixteen_samples() {
    for rgba in [[0, 0, 0, 0], [1, 0, 0, 1], [64, 32, 16, 128], [255; 4]] {
        for opacity in [0, 1, 128, 255] {
            for (width, height) in [(3, 2), (1, 7), (9, 1)] {
                let snapshot = snapshot(owned_fixture(
                    viewport(width, height),
                    root_and_owners(1, width, height),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    vec![image(0, 1, 1, rgba.to_vec())],
                    Vec::new(),
                    vec![image_paint(
                        1,
                        0,
                        0,
                        source(0, 0, 1, 1),
                        destination(0, 0, i64::from(width) * S, i64::from(height) * S),
                        opacity,
                        None,
                    )],
                ));
                let full = snapshot
                    .rasterize_reference_full_sampling_for_test(limits(100))
                    .unwrap();
                let expected = rgba
                    .map(|channel| ((u16::from(channel) * u16::from(opacity) + 127) / 255) as u8);
                assert!(full.bytes().chunks_exact(4).all(|pixel| pixel == expected));
                assert_sampling(&snapshot, true);
            }
        }
    }
}

#[test]
fn scaled_single_texels_preserve_integer_clipping_overlap_and_opacity() {
    let snapshot = snapshot(owned_fixture(
        viewport(8, 6),
        root_and_owners(1, 8, 6),
        vec![
            rect_values(0, 1, 0, 0, 8 * S, 6 * S),
            rect_values(1, 1, 2 * S, S, 4 * S, 4 * S),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![solid(0, color(20, 40, 80, 255))],
        Vec::new(),
        vec![
            image(0, 1, 1, vec![64, 32, 16, 128]),
            image(1, 1, 1, vec![0, 80, 0, 100]),
        ],
        vec![root_clip(0, 1, 1)],
        vec![
            fill(1, 0, 0, 0, 255, None, RULE),
            image_paint(
                1,
                1,
                0,
                source(0, 0, 1, 1),
                destination(-S, -S, 7 * S, 6 * S),
                173,
                Some(0),
            ),
            image_paint(
                1,
                2,
                1,
                source(0, 0, 1, 1),
                destination(3 * S, 2 * S, 3 * S, 3 * S),
                219,
                None,
            ),
        ],
    ));
    assert_sampling(&snapshot, true);
}

#[test]
fn scaled_single_texels_preserve_scalar_domain_rejection() {
    for x in [SpatialScalarV2::MIN_RAW, SpatialScalarV2::MAX_RAW - 2 * S] {
        let snapshot = snapshot(owned_fixture(
            viewport(2, 1),
            vec![
                root(),
                owner_node(1, transform([S, 0, 0, S, -x, 0], 0, 0), 0, 0),
            ],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![image(0, 1, 1, vec![64, 32, 16, 128])],
            Vec::new(),
            vec![image_paint(
                1,
                0,
                0,
                source(0, 0, 1, 1),
                destination(x, 0, 2 * S, S),
                173,
                None,
            )],
        ));
        assert_sampling(&snapshot, true);
    }
}

#[test]
fn single_texel_fractional_destinations_and_transforms_keep_full_sampling() {
    for (matrix, destination) in [
        ([S, 0, 0, S, 0, 0], destination(S / 4, 0, 2 * S, 2 * S)),
        ([S, 0, 0, S, 0, 0], destination(0, 0, 2 * S, 3 * S / 2)),
        ([S, 0, 0, S, S / 4, 0], destination(0, 0, 2 * S, 2 * S)),
        ([2 * S, 0, 0, S, 0, 0], destination(0, 0, 2 * S, 2 * S)),
        ([0, S, -S, 0, 2 * S, 0], destination(0, 0, 2 * S, 2 * S)),
        ([S, S / 4, 0, S, 0, 0], destination(0, 0, 2 * S, 2 * S)),
    ] {
        let snapshot = snapshot(owned_fixture(
            viewport(4, 3),
            vec![root(), owner_node(1, transform(matrix, 0, 0), 4, 3)],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![image(0, 1, 1, vec![128; 4])],
            Vec::new(),
            vec![image_paint(
                1,
                0,
                0,
                source(0, 0, 1, 1),
                destination,
                255,
                None,
            )],
        ));
        assert_sampling(&snapshot, false);
    }
}

#[test]
fn fractional_clips_and_crops_from_larger_images_are_not_newly_admitted() {
    for cropped in [false, true] {
        let snapshot = snapshot(owned_fixture(
            viewport(3, 2),
            root_and_owners(1, 3, 2),
            vec![rect_values(
                0,
                1,
                if cropped { 0 } else { S / 4 },
                0,
                2 * S,
                2 * S,
            )],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![if cropped {
                image(0, 2, 1, vec![128; 8])
            } else {
                image(0, 1, 1, vec![128; 4])
            }],
            vec![root_clip(0, 1, 0)],
            vec![image_paint(
                1,
                0,
                0,
                source(0, 0, 1, 1),
                destination(0, 0, 3 * S, 2 * S),
                255,
                if cropped { None } else { Some(0) },
            )],
        ));
        assert_sampling(&snapshot, false);
    }
}
