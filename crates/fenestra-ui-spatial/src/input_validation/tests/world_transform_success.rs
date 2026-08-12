use super::validated_shape_support::rect_values;
use super::world_transform_support::{
    MAXIMUM, SCALE, ScriptedLayoutEngine, VIEWPORT, expect_valid, fixture, free, identity, layout,
    limits, root, successful_output, transform, world,
};
use crate::model::SpatialAnchorTargetV2;

#[test]
fn layout_and_free_placements_compose_origin_local_and_parent_transforms_exactly() {
    let quarter_about_origin = transform([0, SCALE, -SCALE, 0, 0, 0], 2 * SCALE, 3 * SCALE);
    let scaled_about_origin = transform([2 * SCALE, 0, 0, 3 * SCALE, 0, 0], SCALE, SCALE);
    let fixture = fixture(vec![
        root(),
        layout(1, 0, 0, 0, quarter_about_origin),
        free(
            2,
            1,
            SpatialAnchorTargetV2::Parent,
            4 * SCALE,
            5 * SCALE,
            0,
            0,
            scaled_about_origin,
        ),
    ]);
    let engine = ScriptedLayoutEngine::new(vec![successful_output(&[
        (0, 0, 0, 20, 20),
        (1, 10, 20, 0, 0),
    ])]);
    let proof = expect_valid(prepare_world_transforms!(
        &fixture,
        VIEWPORT,
        limits(),
        &engine
    ));

    assert_eq!(
        proof.world_transform_facts(),
        vec![
            world(0, [SCALE, 0, 0, SCALE, 0, 0]),
            world(1, [0, SCALE, -SCALE, 0, 15 * SCALE, 21 * SCALE]),
            world(2, [0, 2 * SCALE, -3 * SCALE, 0, 12 * SCALE, 24 * SCALE]),
        ]
    );
    assert_eq!(engine.call_count(), 1);
}

#[test]
fn root_is_identity_and_negative_world_determinants_remain_valid() {
    let root_fixture = fixture(vec![root()]);
    let engine = ScriptedLayoutEngine::new(Vec::new());
    let proof = expect_valid(prepare_world_transforms!(
        &root_fixture,
        VIEWPORT,
        limits(),
        &engine
    ));
    assert_eq!(
        proof.world_transform_facts(),
        vec![world(0, [SCALE, 0, 0, SCALE, 0, 0])]
    );
    assert_eq!(engine.call_count(), 0);

    let reflection = transform([-SCALE, 0, 0, SCALE, 0, 0], 0, 0);
    let reflection_fixture = fixture(vec![
        root(),
        free(
            1,
            0,
            SpatialAnchorTargetV2::Viewport,
            0,
            0,
            0,
            0,
            reflection,
        ),
    ]);
    let engine = ScriptedLayoutEngine::new(Vec::new());
    let proof = expect_valid(prepare_world_transforms!(
        &reflection_fixture,
        VIEWPORT,
        limits(),
        &engine
    ));
    assert_eq!(
        proof.world_transform_facts(),
        vec![
            world(0, [SCALE, 0, 0, SCALE, 0, 0]),
            world(1, [-SCALE, 0, 0, SCALE, 0, 0]),
        ]
    );
}

#[test]
fn placed_is_completed_before_world_composition_with_fixed_rounding() {
    let tiny_parent = transform([1, 0, 0, 1, 0, 0], 0, 0);
    let half_translated_child = transform([SCALE, 0, 0, SCALE, SCALE / 2, 0], 0, 0);
    let fixture = fixture(vec![
        root(),
        free(
            1,
            0,
            SpatialAnchorTargetV2::Viewport,
            0,
            0,
            0,
            0,
            tiny_parent,
        ),
        free(
            2,
            1,
            SpatialAnchorTargetV2::Parent,
            SCALE / 2,
            0,
            0,
            0,
            half_translated_child,
        ),
    ]);
    let engine = ScriptedLayoutEngine::new(Vec::new());
    let proof = expect_valid(prepare_world_transforms!(
        &fixture,
        VIEWPORT,
        limits(),
        &engine
    ));

    assert_eq!(
        proof.world_transform_facts(),
        vec![
            world(0, [SCALE, 0, 0, SCALE, 0, 0]),
            world(1, [1, 0, 0, 1, 0, 0]),
            world(2, [1, 0, 0, 1, 1, 0]),
        ]
    );
}

#[test]
fn world_transform_stage_stops_before_closed_aabb_projection() {
    let translated_to_the_edge = transform([SCALE, 0, 0, SCALE, MAXIMUM, 0], 0, 0);
    let fixture = fixture(vec![
        root(),
        free(
            1,
            0,
            SpatialAnchorTargetV2::Viewport,
            0,
            0,
            0,
            0,
            translated_to_the_edge,
        ),
    ])
    .with_shapes(vec![rect_values(0, 1, 0, 0, 1, 1)], Vec::new());
    let engine = ScriptedLayoutEngine::new(Vec::new());
    let proof = expect_valid(prepare_world_transforms!(
        &fixture,
        VIEWPORT,
        limits(),
        &engine
    ));

    assert_eq!(
        proof.world_transform_facts(),
        vec![
            world(0, [SCALE, 0, 0, SCALE, 0, 0]),
            world(1, [SCALE, 0, 0, SCALE, MAXIMUM, 0]),
        ]
    );
    assert_eq!(proof.shape_local_bounds_facts().len(), 1);
}

#[test]
fn identity_transforms_preserve_exact_global_placement_translation() {
    let fixture = fixture(vec![
        root(),
        free(
            1,
            0,
            SpatialAnchorTargetV2::Viewport,
            7 * SCALE,
            -9 * SCALE,
            0,
            0,
            identity(),
        ),
    ]);
    let engine = ScriptedLayoutEngine::new(Vec::new());
    let proof = expect_valid(prepare_world_transforms!(
        &fixture,
        VIEWPORT,
        limits(),
        &engine
    ));
    assert_eq!(
        proof.world_transform_facts(),
        vec![
            world(0, [SCALE, 0, 0, SCALE, 0, 0]),
            world(1, [SCALE, 0, 0, SCALE, 7 * SCALE, -9 * SCALE]),
        ]
    );
}
