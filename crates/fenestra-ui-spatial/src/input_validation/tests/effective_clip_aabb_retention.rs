use std::ptr;

use super::effective_clip_aabb_support::{
    SCALE, ScriptedLayoutEngine, VIEWPORT, aabb, empty, expect_valid, fact, fill, fixture_with,
    hit, identity, limits, nonzero_clip, owner_node, rect_values, retained_phase_ten_fixture, root,
    semantic_fill,
};
use super::validated_hit_support::fill as hit_fill;
use super::validated_paint_support::{destination, fill as paint_fill, image_paint, source};
use super::validated_semantic_support::semantic;
use super::world_transform_support::{logical, output};
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::SpatialFillRuleV2;
use crate::shape::SpatialShapeKindV2;

#[test]
fn terminal_clips_do_not_replace_unclipped_paint_hit_or_semantic_world_aabbs() {
    let fixture = fixture_with(
        vec![root(), owner_node(1, identity(), 20, 20)],
        vec![
            rect_values(0, 1, 0, 0, SCALE, SCALE),
            rect_values(1, 1, 10 * SCALE, 10 * SCALE, 2 * SCALE, 2 * SCALE),
            rect_values(2, 1, 20 * SCALE, 20 * SCALE, 2 * SCALE, 2 * SCALE),
            rect_values(3, 1, 30 * SCALE, 30 * SCALE, 3 * SCALE, 3 * SCALE),
            rect_values(4, 1, -5 * SCALE, 15 * SCALE, 4 * SCALE, 5 * SCALE),
        ],
        vec![nonzero_clip(0, 1, None, 0), nonzero_clip(1, 1, Some(0), 1)],
        vec![fill(1, 0, 2, Some(1))],
        vec![hit(1, 0, 3, Some(1))],
        vec![semantic_fill(1, 0, 4, Some(1))],
    );
    let engine = ScriptedLayoutEngine::new(Vec::new());
    let proof = expect_valid(prepare_effective_clip_aabbs!(
        &fixture,
        VIEWPORT,
        limits(),
        &engine
    ));

    assert_eq!(
        proof.clip_world_aabb_facts(),
        vec![
            fact(0, aabb(0, 0, SCALE, SCALE)),
            fact(1, aabb(10 * SCALE, 10 * SCALE, 12 * SCALE, 12 * SCALE)),
        ]
    );
    assert_eq!(
        proof.effective_clip_world_aabb_facts(),
        vec![fact(0, aabb(0, 0, SCALE, SCALE)), fact(1, empty())]
    );
    assert_eq!(
        proof.paint_world_aabb_facts(),
        vec![fact(
            0,
            aabb(20 * SCALE, 20 * SCALE, 22 * SCALE, 22 * SCALE)
        )]
    );
    assert_eq!(
        proof.hit_world_aabb_facts(),
        vec![fact(
            0,
            aabb(30 * SCALE, 30 * SCALE, 33 * SCALE, 33 * SCALE)
        )]
    );
    assert_eq!(
        proof.semantic_world_aabb_facts(),
        vec![fact(0, aabb(-5 * SCALE, 15 * SCALE, -SCALE, 20 * SCALE))]
    );
    assert_eq!(
        proof.geometry_world_aabb_facts(),
        vec![
            fact(0, aabb(0, 0, 20 * SCALE, 20 * SCALE)),
            fact(1, aabb(0, 0, 20 * SCALE, 20 * SCALE)),
        ]
    );
}

#[test]
fn effective_clips_retain_the_complete_predecessor_and_exact_raw_borrows() {
    let paints = vec![
        paint_fill(1, 0, 0, 0, Some(0), SpatialFillRuleV2::EvenOdd),
        image_paint(
            1,
            1,
            0,
            source(0, 0, 1, 1),
            destination(logical(10), logical(20), logical(3), logical(4)),
            None,
        ),
    ];
    let fixture = retained_phase_ten_fixture(
        paints,
        vec![hit_fill(
            2,
            0,
            1,
            None,
            SpatialFillRuleV2::NonZero,
            SpatialInputPolicyV2::Accept,
        )],
        vec![semantic(3, 0, 2, SpatialFillRuleV2::EvenOdd, None)],
    );
    let requested_limits = limits();
    let raw_input = fixture.input_with_viewport(VIEWPORT);
    let raw_images = raw_input.resources().images();
    let raw_semantics = raw_input.items().semantic_items();
    let engine = ScriptedLayoutEngine::new(vec![Ok(output(&[(0, 0, 0, 10, 10), (1, 1, 2, 3, 4)]))]);
    let proof = expect_valid(prepare_effective_clip_aabbs!(
        &fixture,
        VIEWPORT,
        requested_limits,
        &engine
    ));

    let primitive = fact(0, aabb(logical(1), logical(12), logical(4), logical(16)));
    assert_eq!(proof.clip_world_aabb_facts(), vec![primitive]);
    assert_eq!(proof.effective_clip_world_aabb_facts(), vec![primitive]);
    assert_eq!(proof.geometry_world_aabb_facts().len(), 4);
    assert_eq!(proof.world_transform_facts().len(), 4);
    assert_eq!(proof.placement_facts().len(), 4);
    assert_eq!(proof.dependency_order_facts(), vec![1, 2, 3]);
    assert_eq!(proof.prepared_island_facts(), vec![(0, vec![1, 2])]);
    assert_eq!(proof.path_range_facts(), vec![(0, 0, 2)]);
    assert_eq!(proof.validated_path_facts(), vec![(0, 2, 1)]);
    assert_eq!(proof.polygon_range_facts(), vec![(1, 0, 3)]);
    assert_eq!(
        proof.validated_shape_facts()[1].1,
        SpatialShapeKindV2::Polygon
    );
    assert_eq!(proof.gradient_range_facts(), vec![(1, 0, 2)]);
    assert_eq!(proof.prepared_brush_facts().len(), 2);
    assert_eq!(proof.validated_image_facts().len(), 1);
    assert_eq!(proof.validated_clip_facts().len(), 1);
    assert_eq!(proof.validated_paint_facts().len(), 2);
    assert_eq!(proof.validated_hit_facts().len(), 1);
    assert_eq!(proof.validated_semantic_facts().len(), 1);
    assert_eq!(proof.flattened_path_facts().len(), 1);
    assert_eq!(proof.shape_local_bounds_facts().len(), 3);
    assert_eq!(proof.paint_local_bounds_facts().len(), 2);
    assert_eq!(proof.hit_local_bounds_facts().len(), 1);
    let bytes = proof
        .finalized_image_paint_bytes(1)
        .expect("effective clips retain the exact final P5 token");
    assert!(ptr::eq(bytes, raw_images[0].bytes()));
    assert_eq!(proof.input().items().semantic_items(), raw_semantics);
    assert_eq!(proof.limits(), requested_limits);
    assert_eq!(engine.call_count(), 1);
}
