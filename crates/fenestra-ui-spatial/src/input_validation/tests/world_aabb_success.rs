use super::validated_clip_support::{clip, root_clip};
use super::validated_shape_support::rect_values;
use super::world_aabb_support::{
    MAXIMUM, SCALE, ScriptedLayoutEngine, VIEWPORT, aabb, empty, expect_valid, fact, fill,
    fixture_with, hit, identity, limits, owner_node, root, semantic_fill, transform, valid_shape,
};
use crate::coverage::SpatialFillRuleV2;

#[test]
fn every_output_table_projects_its_exact_local_bound_through_the_owner_world() {
    let local = transform([0, SCALE, -SCALE, 0, 10 * SCALE, 20 * SCALE], 0, 0);
    let fixture = fixture_with(
        vec![root(), owner_node(1, local, 4, 5)],
        vec![
            rect_values(0, 1, SCALE, 2 * SCALE, 2 * SCALE, 3 * SCALE),
            rect_values(1, 1, -2 * SCALE, -SCALE, 4 * SCALE, 6 * SCALE),
            rect_values(2, 1, 0, 0, SCALE, 2 * SCALE),
            rect_values(3, 1, 3 * SCALE, -4 * SCALE, SCALE, 2 * SCALE),
        ],
        vec![root_clip(0, 1, 0)],
        vec![fill(1, 0, 1, Some(0))],
        vec![hit(1, 0, 2, Some(0))],
        vec![semantic_fill(1, 0, 3, Some(0))],
    );
    let engine = ScriptedLayoutEngine::new(Vec::new());
    let proof = expect_valid(prepare_world_aabbs!(&fixture, VIEWPORT, limits(), &engine));

    assert_eq!(
        proof.geometry_world_aabb_facts(),
        vec![
            fact(0, aabb(0, 0, 20 * SCALE, 20 * SCALE)),
            fact(1, aabb(5 * SCALE, 20 * SCALE, 10 * SCALE, 24 * SCALE)),
        ]
    );
    assert_eq!(
        proof.clip_world_aabb_facts(),
        vec![fact(0, aabb(5 * SCALE, 21 * SCALE, 8 * SCALE, 23 * SCALE))]
    );
    assert_eq!(
        proof.paint_world_aabb_facts(),
        vec![fact(0, aabb(5 * SCALE, 18 * SCALE, 11 * SCALE, 22 * SCALE))]
    );
    assert_eq!(
        proof.hit_world_aabb_facts(),
        vec![fact(0, aabb(8 * SCALE, 20 * SCALE, 10 * SCALE, 21 * SCALE))]
    );
    assert_eq!(
        proof.semantic_world_aabb_facts(),
        vec![fact(
            0,
            aabb(12 * SCALE, 23 * SCALE, 14 * SCALE, 24 * SCALE)
        )]
    );
}

#[test]
fn empty_fill_and_clip_bounds_short_circuit_a_poisonous_nonempty_base_projection() {
    let fixture = fixture_with(
        vec![
            root(),
            owner_node(1, transform([MAXIMUM, 0, 0, SCALE, 0, 0], 0, 0), 1, 1),
        ],
        vec![rect_values(0, 1, 2 * SCALE, 0, 0, SCALE)],
        vec![root_clip(0, 1, 0)],
        vec![fill(1, 0, 0, Some(0))],
        vec![hit(1, 0, 0, Some(0))],
        vec![semantic_fill(1, 0, 0, Some(0))],
    );
    let engine = ScriptedLayoutEngine::new(Vec::new());
    let proof = expect_valid(prepare_world_aabbs!(&fixture, VIEWPORT, limits(), &engine));

    assert_eq!(proof.clip_world_aabb_facts(), vec![fact(0, empty())]);
    assert_eq!(proof.paint_world_aabb_facts(), vec![fact(0, empty())]);
    assert_eq!(proof.hit_world_aabb_facts(), vec![fact(0, empty())]);
    assert_eq!(proof.semantic_world_aabb_facts(), vec![fact(0, empty())]);
}

#[test]
fn transformed_bounds_round_minima_down_and_maxima_up_at_half_ticks() {
    let cases = [
        ([SCALE / 2, 0, 0, SCALE / 2, 0, 0], aabb(0, 0, 1, 1)),
        ([-SCALE / 2, 0, 0, SCALE / 2, 0, 0], aabb(-1, 0, 0, 1)),
    ];
    for (matrix, expected) in cases {
        let fixture = fixture_with(
            vec![root(), owner_node(1, transform(matrix, 0, 0), 1, 1)],
            vec![rect_values(0, 1, 1, 1, 1, 1)],
            vec![root_clip(0, 1, 0)],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        let engine = ScriptedLayoutEngine::new(Vec::new());
        let proof = expect_valid(prepare_world_aabbs!(&fixture, VIEWPORT, limits(), &engine));
        assert_eq!(proof.clip_world_aabb_facts(), vec![fact(0, expected)]);
    }
}

#[test]
fn geometry_uses_computed_layout_extents_as_a_local_box_exactly_once() {
    let fixture = fixture_with(
        vec![
            root(),
            super::world_transform_support::layout(1, 0, 3, 4, identity()),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let engine = ScriptedLayoutEngine::new(vec![Ok(super::world_transform_support::output(&[
        (0, 0, 0, 20, 20),
        (1, 7, 8, 5, 6),
    ]))]);
    let proof = expect_valid(prepare_world_aabbs!(&fixture, VIEWPORT, limits(), &engine));

    assert_eq!(
        proof.geometry_world_aabb_facts(),
        vec![
            fact(0, aabb(0, 0, 20 * SCALE, 20 * SCALE)),
            fact(1, aabb(7 * SCALE, 8 * SCALE, 12 * SCALE, 14 * SCALE)),
        ]
    );
    assert_eq!(engine.call_count(), 1);
}

#[test]
fn zero_geometry_extents_remain_closed_points_and_lines() {
    let fixture = fixture_with(
        vec![
            root(),
            super::world_transform_support::free(
                1,
                0,
                crate::model::SpatialAnchorTargetV2::Viewport,
                3 * SCALE,
                4 * SCALE,
                0,
                0,
                identity(),
            ),
            super::world_transform_support::free(
                2,
                0,
                crate::model::SpatialAnchorTargetV2::Viewport,
                5 * SCALE,
                6 * SCALE,
                0,
                5,
                identity(),
            ),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let engine = ScriptedLayoutEngine::new(Vec::new());
    let proof = expect_valid(prepare_world_aabbs!(&fixture, VIEWPORT, limits(), &engine));

    assert_eq!(
        proof.geometry_world_aabb_facts(),
        vec![
            fact(0, aabb(0, 0, 20 * SCALE, 20 * SCALE)),
            fact(1, aabb(3 * SCALE, 4 * SCALE, 3 * SCALE, 4 * SCALE)),
            fact(2, aabb(5 * SCALE, 6 * SCALE, 5 * SCALE, 11 * SCALE)),
        ]
    );
}

#[test]
fn unreferenced_shape_resources_are_not_world_projected() {
    let fixture = fixture_with(
        vec![
            root(),
            owner_node(1, transform([MAXIMUM, 0, 0, SCALE, 0, 0], 0, 0), 1, 1),
        ],
        vec![super::world_aabb_support::fault_shape(
            0,
            1,
            crate::numeric_error::SpatialArithmeticOperationV2::AabbMaxX,
        )],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let engine = ScriptedLayoutEngine::new(Vec::new());
    let proof = expect_valid(prepare_world_aabbs!(&fixture, VIEWPORT, limits(), &engine));

    assert!(proof.clip_world_aabb_facts().is_empty());
    assert!(proof.paint_world_aabb_facts().is_empty());
    assert!(proof.hit_world_aabb_facts().is_empty());
    assert!(proof.semantic_world_aabb_facts().is_empty());
}

#[test]
fn nested_records_use_the_complete_parent_world_not_only_their_local_transform() {
    let fixture = fixture_with(
        vec![
            root(),
            owner_node(
                1,
                transform([SCALE, 0, 0, SCALE, 5 * SCALE, 7 * SCALE], 0, 0),
                10,
                10,
            ),
            super::world_transform_support::free(
                2,
                1,
                crate::model::SpatialAnchorTargetV2::Parent,
                0,
                0,
                1,
                1,
                identity(),
            ),
        ],
        vec![valid_shape(0, 2)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![semantic_fill(2, 0, 0, None)],
    );
    let engine = ScriptedLayoutEngine::new(Vec::new());
    let proof = expect_valid(prepare_world_aabbs!(&fixture, VIEWPORT, limits(), &engine));

    assert_eq!(
        proof.semantic_world_aabb_facts(),
        vec![fact(0, aabb(5 * SCALE, 7 * SCALE, 6 * SCALE, 8 * SCALE))]
    );
}

#[test]
fn clip_intersections_and_terminal_clipping_remain_deferred() {
    let fixture = fixture_with(
        vec![root(), owner_node(1, identity(), 20, 20)],
        vec![
            rect_values(0, 1, 0, 0, SCALE, SCALE),
            rect_values(1, 1, 10 * SCALE, 10 * SCALE, 2 * SCALE, 2 * SCALE),
            rect_values(2, 1, 20 * SCALE, 20 * SCALE, 2 * SCALE, 2 * SCALE),
        ],
        vec![
            root_clip(0, 1, 0),
            clip(1, 1, Some(0), 1, SpatialFillRuleV2::EvenOdd),
        ],
        vec![fill(1, 0, 2, Some(1))],
        vec![hit(1, 0, 2, Some(1))],
        vec![semantic_fill(1, 0, 2, Some(1))],
    );
    let engine = ScriptedLayoutEngine::new(Vec::new());
    let proof = expect_valid(prepare_world_aabbs!(&fixture, VIEWPORT, limits(), &engine));

    assert_eq!(
        proof.clip_world_aabb_facts(),
        vec![
            fact(0, aabb(0, 0, SCALE, SCALE)),
            fact(1, aabb(10 * SCALE, 10 * SCALE, 12 * SCALE, 12 * SCALE)),
        ]
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
            aabb(20 * SCALE, 20 * SCALE, 22 * SCALE, 22 * SCALE)
        )]
    );
    assert_eq!(
        proof.semantic_world_aabb_facts(),
        vec![fact(
            0,
            aabb(20 * SCALE, 20 * SCALE, 22 * SCALE, 22 * SCALE)
        )]
    );
}
