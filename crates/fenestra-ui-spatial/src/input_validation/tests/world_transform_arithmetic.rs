use super::world_transform_support::{
    MAXIMUM, SCALE, ScriptedLayoutEngine, VIEWPORT, expect_arithmetic, fixture, free, identity,
    limits, root, transform,
};
use crate::model::SpatialAnchorTargetV2;
use crate::vocabulary::{SpatialAffineComponentV2, SpatialTransformStageV2};

#[test]
fn each_nested_about_composition_reports_its_reachable_translation_axis() {
    let inner_cases = [
        (
            transform([2 * SCALE, 0, 0, SCALE, 0, 0], MAXIMUM, 0),
            SpatialAffineComponentV2::Tx,
        ),
        (
            transform([SCALE, 0, 0, 2 * SCALE, 0, 0], 0, MAXIMUM),
            SpatialAffineComponentV2::Ty,
        ),
    ];
    for (local, component) in inner_cases {
        let fixture = fixture(vec![
            root(),
            free(1, 0, SpatialAnchorTargetV2::Viewport, 0, 0, 0, 0, local),
        ]);
        let engine = ScriptedLayoutEngine::new(Vec::new());
        expect_arithmetic(
            prepare_world_transforms!(&fixture, VIEWPORT, limits(), &engine),
            1,
            SpatialTransformStageV2::About,
            component,
        );
    }

    let outer_cases = [
        (
            transform([0, SCALE, -SCALE, 0, 1, 0], MAXIMUM, 0),
            SpatialAffineComponentV2::Tx,
        ),
        (
            transform([0, SCALE, -SCALE, 0, 0, 1], 0, MAXIMUM),
            SpatialAffineComponentV2::Ty,
        ),
    ];
    for (local, component) in outer_cases {
        let fixture = fixture(vec![
            root(),
            free(1, 0, SpatialAnchorTargetV2::Viewport, 0, 0, 0, 0, local),
        ]);
        let engine = ScriptedLayoutEngine::new(Vec::new());
        expect_arithmetic(
            prepare_world_transforms!(&fixture, VIEWPORT, limits(), &engine),
            1,
            SpatialTransformStageV2::About,
            component,
        );
    }
}

#[test]
fn the_inner_about_composition_finishes_before_the_outer_component_order() {
    let inner_ty_before_outer_tx = transform([0, SCALE, -SCALE, 0, 0, -1], MAXIMUM, MAXIMUM);
    let inner_fixture = fixture(vec![
        root(),
        free(
            1,
            0,
            SpatialAnchorTargetV2::Viewport,
            0,
            0,
            0,
            0,
            inner_ty_before_outer_tx,
        ),
    ]);
    let engine = ScriptedLayoutEngine::new(Vec::new());
    expect_arithmetic(
        prepare_world_transforms!(&inner_fixture, VIEWPORT, limits(), &engine),
        1,
        SpatialTransformStageV2::About,
        SpatialAffineComponentV2::Ty,
    );

    let half_over = MAXIMUM / 2 + 1;
    let both_outer_axes = transform([-SCALE, 0, 0, -SCALE, 0, 0], half_over, half_over);
    let outer_fixture = fixture(vec![
        root(),
        free(
            1,
            0,
            SpatialAnchorTargetV2::Viewport,
            0,
            0,
            0,
            0,
            both_outer_axes,
        ),
    ]);
    let engine = ScriptedLayoutEngine::new(Vec::new());
    expect_arithmetic(
        prepare_world_transforms!(&outer_fixture, VIEWPORT, limits(), &engine),
        1,
        SpatialTransformStageV2::About,
        SpatialAffineComponentV2::Tx,
    );
}

#[test]
fn placed_translation_reports_x_then_y_after_about_succeeds() {
    let cases = [
        (
            MAXIMUM,
            0,
            transform([SCALE, 0, 0, SCALE, 1, 0], 0, 0),
            SpatialAffineComponentV2::Tx,
        ),
        (
            0,
            MAXIMUM,
            transform([SCALE, 0, 0, SCALE, 0, 1], 0, 0),
            SpatialAffineComponentV2::Ty,
        ),
        (
            MAXIMUM,
            MAXIMUM,
            transform([SCALE, 0, 0, SCALE, 1, 1], 0, 0),
            SpatialAffineComponentV2::Tx,
        ),
    ];
    for (offset_x, offset_y, local, component) in cases {
        let fixture = fixture(vec![
            root(),
            free(
                1,
                0,
                SpatialAnchorTargetV2::Viewport,
                offset_x,
                offset_y,
                0,
                0,
                local,
            ),
        ]);
        let engine = ScriptedLayoutEngine::new(Vec::new());
        expect_arithmetic(
            prepare_world_transforms!(&fixture, VIEWPORT, limits(), &engine),
            1,
            SpatialTransformStageV2::Placed,
            component,
        );
    }
}

#[test]
fn world_composition_preserves_every_reachable_component_location() {
    let cases = [
        (
            [MAXIMUM, 0, 0, SCALE, 0, 0],
            [SCALE + 1, 0, 0, SCALE, 0, 0],
            SpatialAffineComponentV2::A,
        ),
        (
            [SCALE, MAXIMUM, 0, SCALE, 0, 0],
            [SCALE + 1, 0, 0, SCALE, 0, 0],
            SpatialAffineComponentV2::B,
        ),
        (
            [MAXIMUM, 0, 0, SCALE, 0, 0],
            [SCALE, 0, SCALE + 1, SCALE, 0, 0],
            SpatialAffineComponentV2::C,
        ),
        (
            [SCALE, MAXIMUM, 0, SCALE, 0, 0],
            [SCALE, 0, SCALE + 1, SCALE, 0, 0],
            SpatialAffineComponentV2::D,
        ),
        (
            [MAXIMUM, 0, 0, SCALE, 0, 0],
            [SCALE, 0, 0, SCALE, SCALE + 1, 0],
            SpatialAffineComponentV2::Tx,
        ),
        (
            [SCALE, MAXIMUM, 0, SCALE, 0, 0],
            [SCALE, 0, 0, SCALE, SCALE + 1, 0],
            SpatialAffineComponentV2::Ty,
        ),
        (
            [MAXIMUM, MAXIMUM, 0, SCALE, 0, 0],
            [SCALE + 1, 0, 0, SCALE, 0, 0],
            SpatialAffineComponentV2::A,
        ),
        (
            [MAXIMUM, MAXIMUM, 0, SCALE, 0, 0],
            [SCALE, 0, SCALE + 1, SCALE, 0, 0],
            SpatialAffineComponentV2::C,
        ),
        (
            [MAXIMUM, MAXIMUM, 0, SCALE, 0, 0],
            [SCALE, 0, 0, SCALE, SCALE + 1, 0],
            SpatialAffineComponentV2::Tx,
        ),
    ];

    for (parent, child, component) in cases {
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
                transform(parent, 0, 0),
            ),
            free(
                2,
                1,
                SpatialAnchorTargetV2::Parent,
                0,
                0,
                0,
                0,
                transform(child, 0, 0),
            ),
        ]);
        let engine = ScriptedLayoutEngine::new(Vec::new());
        expect_arithmetic(
            prepare_world_transforms!(&fixture, VIEWPORT, limits(), &engine),
            2,
            SpatialTransformStageV2::World,
            component,
        );
    }
}

#[test]
fn identity_transform_reaches_all_three_stages_without_error() {
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
            identity(),
        ),
    ]);
    let engine = ScriptedLayoutEngine::new(Vec::new());
    let _ = super::world_transform_support::expect_valid(prepare_world_transforms!(
        &fixture,
        VIEWPORT,
        limits(),
        &engine
    ));
}
