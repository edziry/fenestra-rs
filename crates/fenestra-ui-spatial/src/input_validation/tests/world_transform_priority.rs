use super::world_transform_support::{
    MAXIMUM, SCALE, ScriptedLayoutEngine, VIEWPORT, expect_arithmetic,
    expect_predecessor_arithmetic, expect_singular, expect_valid, fixture, free, free_anchored,
    identity, layout, limits, node_target, root, successful_output, transform,
};
use crate::model::{SpatialAnchorComponentV2, SpatialAnchorTargetV2, SpatialViewportV2};
use crate::numeric_error::SpatialArithmeticOperationV2;
use crate::vocabulary::{SpatialAffineComponentV2, SpatialTransformStageV2};

#[test]
fn spatial_preorder_not_dependency_order_selects_the_first_transform_failure() {
    let about_x = transform([0, SCALE, -SCALE, 0, 1, 0], MAXIMUM, 0);
    let about_y = transform([0, SCALE, -SCALE, 0, 0, 1], 0, MAXIMUM);
    let fixture = fixture(vec![
        root(),
        free(1, 0, node_target(4), 0, 0, 0, 0, about_x),
        free(2, 0, SpatialAnchorTargetV2::Viewport, 0, 0, 0, 0, about_y),
        free(3, 2, SpatialAnchorTargetV2::Parent, 0, 0, 0, 0, identity()),
        free(
            4,
            0,
            SpatialAnchorTargetV2::Viewport,
            0,
            0,
            0,
            0,
            identity(),
        ),
    ]);
    let graph = expect_valid(prepare_dependency_graph!(&fixture, VIEWPORT, limits()));
    assert_eq!(graph.dependency_order_facts(), vec![2, 3, 4, 1]);

    let engine = ScriptedLayoutEngine::new(Vec::new());
    expect_arithmetic(
        prepare_world_transforms!(&fixture, VIEWPORT, limits(), &engine),
        1,
        SpatialTransformStageV2::About,
        SpatialAffineComponentV2::Tx,
    );
}

#[test]
fn one_node_finishes_world_before_the_next_nodes_about_stage() {
    let parent = transform([MAXIMUM, 0, 0, SCALE, 0, 0], 0, 0);
    let child = transform([SCALE + 1, 0, 0, SCALE, 0, 0], 0, 0);
    let later_about = transform([0, SCALE, -SCALE, 0, 0, 1], 0, MAXIMUM);
    let fixture = fixture(vec![
        root(),
        free(1, 0, SpatialAnchorTargetV2::Viewport, 0, 0, 0, 0, parent),
        free(2, 1, SpatialAnchorTargetV2::Parent, 0, 0, 0, 0, child),
        free(
            3,
            0,
            SpatialAnchorTargetV2::Viewport,
            0,
            0,
            0,
            0,
            later_about,
        ),
    ]);
    let engine = ScriptedLayoutEngine::new(Vec::new());
    expect_arithmetic(
        prepare_world_transforms!(&fixture, VIEWPORT, limits(), &engine),
        2,
        SpatialTransformStageV2::World,
        SpatialAffineComponentV2::A,
    );
}

#[test]
fn an_earlier_nodes_about_failure_precedes_a_later_nodes_world_failure() {
    let about = transform([0, SCALE, -SCALE, 0, 1, 0], MAXIMUM, 0);
    let world_parent = transform([MAXIMUM, 0, 0, SCALE, 0, 0], 0, 0);
    let world_child = transform([SCALE + 1, 0, 0, SCALE, 0, 0], 0, 0);
    let fixture = fixture(vec![
        root(),
        free(1, 0, SpatialAnchorTargetV2::Viewport, 0, 0, 0, 0, about),
        free(
            2,
            0,
            SpatialAnchorTargetV2::Viewport,
            0,
            0,
            0,
            0,
            world_parent,
        ),
        free(3, 2, SpatialAnchorTargetV2::Parent, 0, 0, 0, 0, world_child),
    ]);
    let engine = ScriptedLayoutEngine::new(Vec::new());
    expect_arithmetic(
        prepare_world_transforms!(&fixture, VIEWPORT, limits(), &engine),
        1,
        SpatialTransformStageV2::About,
        SpatialAffineComponentV2::Tx,
    );
}

#[test]
fn about_composition_precedes_placed_translation_on_the_same_node() {
    let about_and_placed = transform([0, SCALE, -SCALE, 0, 1, 0], MAXIMUM, 0);
    let fixture = fixture(vec![
        root(),
        free(
            1,
            0,
            SpatialAnchorTargetV2::Viewport,
            MAXIMUM,
            0,
            0,
            0,
            about_and_placed,
        ),
    ]);
    let engine = ScriptedLayoutEngine::new(Vec::new());
    expect_arithmetic(
        prepare_world_transforms!(&fixture, VIEWPORT, limits(), &engine),
        1,
        SpatialTransformStageV2::About,
        SpatialAffineComponentV2::Tx,
    );
}

#[test]
fn placed_arithmetic_precedes_a_world_component_failure_on_the_same_node() {
    let parent = transform([MAXIMUM, 0, 0, SCALE, 0, 0], 0, 0);
    let placed_and_world = transform([SCALE + 1, 0, 0, SCALE, 1, 0], 0, 0);
    let fixture = fixture(vec![
        root(),
        free(1, 0, SpatialAnchorTargetV2::Viewport, 0, 0, 0, 0, parent),
        free(
            2,
            1,
            SpatialAnchorTargetV2::Parent,
            MAXIMUM,
            0,
            0,
            0,
            placed_and_world,
        ),
    ]);
    let engine = ScriptedLayoutEngine::new(Vec::new());
    expect_arithmetic(
        prepare_world_transforms!(&fixture, VIEWPORT, limits(), &engine),
        2,
        SpatialTransformStageV2::Placed,
        SpatialAffineComponentV2::Tx,
    );
}

#[test]
fn rounded_world_singularity_is_reported_at_the_child_before_later_nodes() {
    let tiny = transform([1, 0, 0, 1, 0, 0], 0, 0);
    let later_about = transform([0, SCALE, -SCALE, 0, 1, 0], MAXIMUM, 0);
    let fixture = fixture(vec![
        root(),
        free(1, 0, SpatialAnchorTargetV2::Viewport, 0, 0, 0, 0, tiny),
        free(2, 1, SpatialAnchorTargetV2::Parent, 0, 0, 0, 0, tiny),
        free(
            3,
            0,
            SpatialAnchorTargetV2::Viewport,
            0,
            0,
            0,
            0,
            later_about,
        ),
    ]);
    let engine = ScriptedLayoutEngine::new(Vec::new());
    expect_singular(
        prepare_world_transforms!(&fixture, VIEWPORT, limits(), &engine),
        2,
        SpatialTransformStageV2::World,
    );
}

#[test]
fn placement_execution_failure_precedes_world_composition() {
    let about = transform([0, SCALE, -SCALE, 0, 1, 0], MAXIMUM, 0);
    let fixture = fixture(vec![
        root(),
        free_anchored(
            1,
            0,
            SpatialAnchorTargetV2::Viewport,
            SpatialAnchorComponentV2::End,
            SpatialAnchorComponentV2::Start,
            1,
            0,
            0,
            0,
            about,
        ),
    ]);
    let engine = ScriptedLayoutEngine::new(Vec::new());
    expect_predecessor_arithmetic(
        prepare_world_transforms!(
            &fixture,
            SpatialViewportV2::new(i32::MAX, 0),
            limits(),
            &engine
        ),
        1,
        SpatialArithmeticOperationV2::TargetOffsetX,
    );
}

#[test]
fn every_scheduled_island_call_finishes_before_world_composition_begins() {
    let about = transform([0, SCALE, -SCALE, 0, 1, 0], MAXIMUM, 0);
    let fixture = fixture(vec![
        root(),
        layout(1, 0, 0, 0, about),
        free(
            2,
            0,
            SpatialAnchorTargetV2::Viewport,
            0,
            0,
            0,
            0,
            identity(),
        ),
        layout(3, 2, 0, 0, identity()),
    ]);
    let engine = ScriptedLayoutEngine::new(vec![
        successful_output(&[(0, 0, 0, 20, 20), (1, 0, 0, 0, 0)]),
        successful_output(&[(0, 0, 0, 0, 0), (1, 0, 0, 0, 0)]),
    ]);
    expect_arithmetic(
        prepare_world_transforms!(&fixture, VIEWPORT, limits(), &engine),
        1,
        SpatialTransformStageV2::About,
        SpatialAffineComponentV2::Tx,
    );
    assert_eq!(engine.call_count(), 2);
}
