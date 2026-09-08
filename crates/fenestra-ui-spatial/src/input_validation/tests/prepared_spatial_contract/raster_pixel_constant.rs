use std::sync::Arc;

use super::super::validated_clip_support::clip;
use super::super::validated_shape_support::rect_values;
use super::super::world_transform_support::{free, identity, root, transform};
use super::raster_support::*;
use crate::coverage::SpatialFillRuleV2;
use crate::model::SpatialAnchorTargetV2;
use crate::owned_input::SpatialOwnedInputV2;
use crate::prototype::{ReferenceRasterV2, SpatialResolvedSnapshotV2};

mod solid_images;

pub(super) const RULE: SpatialFillRuleV2 = SpatialFillRuleV2::NonZero;

pub(super) fn assert_sampling(
    snapshot: &SpatialResolvedSnapshotV2,
    constant: bool,
) -> ReferenceRasterV2 {
    assert_eq!(snapshot.pixel_samples_are_constant_for_test(), constant);
    let optimized = snapshot.rasterize_reference(limits(100_000)).unwrap();
    let original = snapshot
        .rasterize_reference_full_sampling_for_test(limits(100_000))
        .unwrap();
    assert_eq!(optimized.bytes(), original.bytes());
    optimized
}

pub(super) fn mixed_source(
    x: i64,
    y: i64,
    alpha: u8,
    width: i32,
    height: i32,
) -> Arc<SpatialOwnedInputV2> {
    let mut image_bytes = Vec::new();
    for index in 0..20 {
        let a = [0, 1, 64, 128, 255][index % 5];
        image_bytes.extend([a / 2, a / 3, a, a]);
    }
    owned_fixture(
        viewport(width, height),
        vec![
            root(),
            free(
                1,
                0,
                SpatialAnchorTargetV2::Viewport,
                0,
                0,
                8,
                6,
                transform([S, 0, 0, S, x, y], 0, 0),
            ),
            free(
                2,
                1,
                SpatialAnchorTargetV2::Viewport,
                0,
                0,
                8,
                6,
                identity(),
            ),
        ],
        vec![
            rect_values(0, 1, -2 * S, -S, 8 * S, 6 * S),
            rect_values(1, 2, -S, 0, 6 * S, 4 * S),
            rect_values(2, 1, -S, -S, 6 * S, 5 * S),
            rect_values(3, 2, 0, 0, 4 * S, 4 * S),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![
            solid(0, color(128, 64, 32, 255)),
            solid(1, color(0, 255, 128, alpha)),
            solid(2, color(0, 0, 0, 0)),
        ],
        Vec::new(),
        vec![image(0, 5, 4, image_bytes)],
        vec![clip(0, 1, None, 2, RULE), clip(1, 2, Some(0), 3, RULE)],
        vec![
            fill(1, 0, 0, 0, 213, None, RULE),
            fill(2, 0, 1, 1, 181, Some(0), RULE),
            image_paint(
                2,
                1,
                0,
                source(1, 1, 3, 2),
                destination(-S, S, 3 * S, 2 * S),
                173,
                Some(1),
            ),
            fill(2, 2, 1, 2, 255, None, RULE),
        ],
    )
}

#[test]
fn integer_rectangles_and_one_to_one_images_match_all_sixteen_samples() {
    for x in [-3 * S, 0, 2 * S] {
        for y in [-2 * S, 0, S] {
            for alpha in [0, 64, 128, 255] {
                let snapshot = snapshot(mixed_source(x, y, alpha, 8, 6));
                assert_sampling(&snapshot, true);
            }
        }
    }
}

#[test]
fn empty_frames_and_zero_extent_rectangles_have_constant_samples() {
    let empty = snapshot(empty_owned(viewport(3, 2)));
    assert_sampling(&empty, true);
    let degenerate = snapshot(owned_fixture(
        viewport(3, 2),
        root_and_owners(1, 3, 2),
        vec![rect_values(0, 1, S, S, 0, S)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![solid(0, color(255, 255, 255, 255))],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![fill(1, 0, 0, 0, 255, None, RULE)],
    ));
    assert_eq!(assert_sampling(&degenerate, true).bytes(), &[0; 24]);
}

#[test]
fn integer_sampling_keeps_viewport_budget_errors_exact() {
    let snapshot = snapshot(mixed_source(0, 0, 128, 8, 6));
    let optimized = snapshot.rasterize_reference(limits(47)).err().unwrap();
    let original = snapshot
        .rasterize_reference_full_sampling_for_test(limits(47))
        .err()
        .unwrap();
    assert_eq!(optimized, original);
}

#[test]
fn integer_translation_preserves_sampling_at_scalar_domain_edges() {
    use super::super::world_aabb_support::owner_node;
    use crate::model::SpatialScalarV2;

    for x in [SpatialScalarV2::MIN_RAW, SpatialScalarV2::MAX_RAW - S] {
        let snapshot = snapshot(owned_fixture(
            viewport(1, 1),
            vec![
                root(),
                owner_node(1, transform([S, 0, 0, S, -x, 0], 0, 0), 0, 0),
            ],
            vec![rect_values(0, 1, x, 0, S, S)],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![solid(0, color(128, 64, 32, 128))],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![fill(1, 0, 0, 0, 173, None, RULE)],
        ));
        assert_sampling(&snapshot, true);
    }
}

#[test]
fn full_frame_comparison_records_focused_raster_timing() {
    use std::hint::black_box;
    use std::time::Instant;

    let mut bytes = Vec::new();
    for index in 0..160 * 40 {
        let a = [0, 64, 128, 255][index % 4];
        bytes.extend([a / 3, a / 2, a, a]);
    }
    let snapshot = snapshot(owned_fixture(
        viewport(320, 180),
        root_and_owners(3, 320, 180),
        vec![
            rect_values(0, 1, 0, 0, 320 * S, 180 * S),
            rect_values(1, 2, 8 * S, 8 * S, 304 * S, 164 * S),
            rect_values(2, 3, 150 * S, 20 * S, 100 * S, 100 * S),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![
            solid(0, color(20, 40, 80, 255)),
            solid(1, color(0, 255, 128, 128)),
            solid(2, color(255, 255, 255, 32)),
        ],
        Vec::new(),
        vec![image(0, 160, 40, bytes)],
        Vec::new(),
        vec![
            fill(1, 0, 0, 0, 255, None, RULE),
            fill(2, 0, 1, 1, 213, None, RULE),
            image_paint(
                3,
                0,
                0,
                source(0, 0, 160, 40),
                destination(80 * S, 60 * S, 160 * S, 40 * S),
                255,
                None,
            ),
            fill(3, 1, 2, 2, 255, None, RULE),
        ],
    ));
    assert!(snapshot.pixel_samples_are_constant_for_test());
    let start = Instant::now();
    let full = black_box(&snapshot)
        .rasterize_reference_full_sampling_for_test(limits(57_600))
        .unwrap();
    let full_time = start.elapsed();
    let start = Instant::now();
    let optimized = black_box(&snapshot)
        .rasterize_reference(limits(57_600))
        .unwrap();
    let optimized_time = start.elapsed();
    assert_eq!(optimized.bytes(), full.bytes());
    eprintln!("320x180 pixels, four paints: full={full_time:?}, constant={optimized_time:?}");
}
