use super::effective_clip_aabb_support::{
    SCALE, ScriptedLayoutEngine, VIEWPORT, aabb, clips_only_fixture, empty, even_odd_clip,
    expect_valid, fact, fixture_with, free, identity, limits, nonzero_clip, rect_values, root,
};
use super::world_aabb_support::{expect_aabb_error, geometry_fault_fixture};
use crate::model::SpatialAnchorTargetV2;
use crate::numeric_error::SpatialArithmeticOperationV2;

#[test]
fn empty_clip_table_produces_a_distinct_empty_effective_stage() {
    let fixture = clips_only_fixture(Vec::new(), Vec::new());
    let engine = ScriptedLayoutEngine::new(Vec::new());
    let proof = expect_valid(prepare_effective_clip_aabbs!(
        &fixture,
        VIEWPORT,
        limits(),
        &engine
    ));

    assert!(proof.clip_world_aabb_facts().is_empty());
    assert!(proof.effective_clip_world_aabb_facts().is_empty());
    assert_eq!(proof.geometry_world_aabb_facts().len(), 2);
}

#[test]
fn complete_world_aabb_projection_precedes_the_infallible_clip_stage() {
    let operation = SpatialArithmeticOperationV2::AabbMaxX;
    let fixture = geometry_fault_fixture(operation);
    let engine = ScriptedLayoutEngine::new(Vec::new());

    expect_aabb_error(
        prepare_effective_clip_aabbs!(&fixture, VIEWPORT, limits(), &engine),
        1,
        operation,
    );
}

#[test]
fn deep_chains_intersect_each_primitive_with_the_complete_effective_parent() {
    let fixture = clips_only_fixture(
        vec![
            rect_values(0, 1, 0, 0, 10 * SCALE, 10 * SCALE),
            rect_values(1, 1, 2 * SCALE, -5 * SCALE, 10 * SCALE, 13 * SCALE),
            rect_values(2, 1, -SCALE, 3 * SCALE, 21 * SCALE, 17 * SCALE),
        ],
        vec![
            nonzero_clip(0, 1, None, 0),
            even_odd_clip(1, 1, Some(0), 1),
            nonzero_clip(2, 1, Some(1), 2),
        ],
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
            fact(0, aabb(0, 0, 10 * SCALE, 10 * SCALE)),
            fact(1, aabb(2 * SCALE, -5 * SCALE, 12 * SCALE, 8 * SCALE)),
            fact(2, aabb(-SCALE, 3 * SCALE, 20 * SCALE, 20 * SCALE)),
        ]
    );
    assert_eq!(
        proof.effective_clip_world_aabb_facts(),
        vec![
            fact(0, aabb(0, 0, 10 * SCALE, 10 * SCALE)),
            fact(1, aabb(2 * SCALE, 0, 10 * SCALE, 8 * SCALE)),
            fact(2, aabb(2 * SCALE, 3 * SCALE, 10 * SCALE, 8 * SCALE)),
        ]
    );
}

#[test]
fn branches_use_the_named_parent_and_parentless_records_reset_the_chain() {
    let fixture = clips_only_fixture(
        vec![
            rect_values(0, 1, 0, 0, 10 * SCALE, 10 * SCALE),
            rect_values(1, 1, SCALE, SCALE, 3 * SCALE, 3 * SCALE),
            rect_values(2, 1, 20 * SCALE, 20 * SCALE, SCALE, SCALE),
            rect_values(3, 1, 6 * SCALE, 6 * SCALE, 3 * SCALE, 3 * SCALE),
            rect_values(4, 1, 100 * SCALE, 100 * SCALE, 10 * SCALE, 10 * SCALE),
        ],
        vec![
            nonzero_clip(0, 1, None, 0),
            nonzero_clip(1, 1, Some(0), 1),
            nonzero_clip(2, 1, Some(0), 2),
            nonzero_clip(3, 1, Some(0), 3),
            even_odd_clip(4, 1, None, 4),
        ],
    );
    let engine = ScriptedLayoutEngine::new(Vec::new());
    let proof = expect_valid(prepare_effective_clip_aabbs!(
        &fixture,
        VIEWPORT,
        limits(),
        &engine
    ));

    assert_eq!(
        proof.effective_clip_world_aabb_facts(),
        vec![
            fact(0, aabb(0, 0, 10 * SCALE, 10 * SCALE)),
            fact(1, aabb(SCALE, SCALE, 4 * SCALE, 4 * SCALE)),
            fact(2, empty()),
            fact(3, aabb(6 * SCALE, 6 * SCALE, 9 * SCALE, 9 * SCALE)),
            fact(4, aabb(100 * SCALE, 100 * SCALE, 110 * SCALE, 110 * SCALE)),
        ]
    );
}

#[test]
fn intersections_use_each_owners_primitive_world_aabb_not_local_shape_bounds() {
    let fixture = fixture_with(
        vec![
            root(),
            free(
                1,
                0,
                SpatialAnchorTargetV2::Viewport,
                10 * SCALE,
                20 * SCALE,
                20,
                20,
                identity(),
            ),
            free(
                2,
                1,
                SpatialAnchorTargetV2::Parent,
                2 * SCALE,
                3 * SCALE,
                20,
                20,
                identity(),
            ),
        ],
        vec![
            rect_values(0, 1, 0, 0, 4 * SCALE, 4 * SCALE),
            rect_values(1, 2, 0, 0, 4 * SCALE, 4 * SCALE),
        ],
        vec![nonzero_clip(0, 1, None, 0), nonzero_clip(1, 2, Some(0), 1)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
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
            fact(0, aabb(10 * SCALE, 20 * SCALE, 14 * SCALE, 24 * SCALE)),
            fact(1, aabb(12 * SCALE, 23 * SCALE, 16 * SCALE, 27 * SCALE)),
        ]
    );
    assert_eq!(
        proof.effective_clip_world_aabb_facts(),
        vec![
            fact(0, aabb(10 * SCALE, 20 * SCALE, 14 * SCALE, 24 * SCALE)),
            fact(1, aabb(12 * SCALE, 23 * SCALE, 14 * SCALE, 24 * SCALE)),
        ]
    );
}

#[test]
fn equality_disjointness_and_empty_ancestors_use_closed_canonical_intersection() {
    let fixture = clips_only_fixture(
        vec![
            rect_values(0, 1, 0, 0, 10 * SCALE, 10 * SCALE),
            rect_values(1, 1, 10 * SCALE, 2 * SCALE, 5 * SCALE, 6 * SCALE),
            rect_values(2, 1, 10 * SCALE, 10 * SCALE, 5 * SCALE, 5 * SCALE),
            rect_values(3, 1, 11 * SCALE, 0, SCALE, SCALE),
            rect_values(4, 1, -5 * SCALE, -5 * SCALE, 20 * SCALE, 20 * SCALE),
            rect_values(5, 1, 2 * SCALE, 2 * SCALE, 0, 4 * SCALE),
        ],
        vec![
            nonzero_clip(0, 1, None, 0),
            nonzero_clip(1, 1, Some(0), 1),
            nonzero_clip(2, 1, Some(0), 2),
            nonzero_clip(3, 1, Some(0), 3),
            nonzero_clip(4, 1, Some(3), 4),
            nonzero_clip(5, 1, None, 5),
        ],
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
            fact(0, aabb(0, 0, 10 * SCALE, 10 * SCALE)),
            fact(1, aabb(10 * SCALE, 2 * SCALE, 15 * SCALE, 8 * SCALE)),
            fact(2, aabb(10 * SCALE, 10 * SCALE, 15 * SCALE, 15 * SCALE)),
            fact(3, aabb(11 * SCALE, 0, 12 * SCALE, SCALE)),
            fact(4, aabb(-5 * SCALE, -5 * SCALE, 15 * SCALE, 15 * SCALE)),
            fact(5, empty()),
        ]
    );
    assert_eq!(
        proof.effective_clip_world_aabb_facts(),
        vec![
            fact(0, aabb(0, 0, 10 * SCALE, 10 * SCALE)),
            fact(1, aabb(10 * SCALE, 2 * SCALE, 10 * SCALE, 8 * SCALE)),
            fact(2, aabb(10 * SCALE, 10 * SCALE, 10 * SCALE, 10 * SCALE)),
            fact(3, empty()),
            fact(4, empty()),
            fact(5, empty()),
        ]
    );
}
