use super::placement_execution_support::{
    ScriptedLayoutEngine, expect_arithmetic, fixture, free, limits, root,
};
use crate::model::{
    SpatialAnchorComponentV2, SpatialAnchorTargetV2, SpatialScalarV2, SpatialViewportV2,
};
use crate::numeric_error::SpatialArithmeticOperationV2;

#[test]
fn free_execution_is_stage_major_then_axis_major_within_each_stage() {
    use SpatialAnchorComponentV2::{End, Start};
    use SpatialArithmeticOperationV2::{
        BaseFarY, ParentDeltaX, SelfSubtractionX, SelfSubtractionY, TargetOffsetX, TargetOffsetY,
    };

    let maximum = SpatialScalarV2::MAX_RAW;
    let minimum = SpatialScalarV2::MIN_RAW;
    let cases = [
        (
            SpatialViewportV2::new(i32::MAX, i32::MAX),
            vec![
                root(),
                free(
                    1,
                    0,
                    0,
                    0,
                    Start,
                    Start,
                    SpatialAnchorTargetV2::Viewport,
                    End,
                    End,
                    1,
                    1,
                ),
            ],
            TargetOffsetX,
            1,
        ),
        (
            SpatialViewportV2::new(0, 0),
            vec![
                root(),
                free(
                    1,
                    0,
                    1,
                    1,
                    End,
                    End,
                    SpatialAnchorTargetV2::Viewport,
                    Start,
                    Start,
                    minimum,
                    minimum,
                ),
            ],
            SelfSubtractionX,
            1,
        ),
        (
            SpatialViewportV2::new(0, 0),
            vec![
                root(),
                free(
                    1,
                    0,
                    1,
                    1,
                    Start,
                    Start,
                    SpatialAnchorTargetV2::Viewport,
                    Start,
                    Start,
                    maximum,
                    maximum,
                ),
            ],
            crate::numeric_error::SpatialArithmeticOperationV2::BaseFarX,
            1,
        ),
        (
            SpatialViewportV2::new(0, 0),
            vec![
                root(),
                free(
                    1,
                    0,
                    0,
                    0,
                    Start,
                    Start,
                    SpatialAnchorTargetV2::Viewport,
                    Start,
                    Start,
                    minimum,
                    minimum,
                ),
                free(
                    2,
                    1,
                    0,
                    0,
                    Start,
                    Start,
                    SpatialAnchorTargetV2::Viewport,
                    Start,
                    Start,
                    maximum,
                    maximum,
                ),
            ],
            ParentDeltaX,
            2,
        ),
        (
            SpatialViewportV2::new(0, i32::MAX),
            vec![
                root(),
                free(
                    1,
                    0,
                    1,
                    0,
                    End,
                    Start,
                    SpatialAnchorTargetV2::Viewport,
                    Start,
                    End,
                    minimum,
                    1,
                ),
            ],
            TargetOffsetY,
            1,
        ),
        (
            SpatialViewportV2::new(0, 0),
            vec![
                root(),
                free(
                    1,
                    0,
                    1,
                    1,
                    Start,
                    End,
                    SpatialAnchorTargetV2::Viewport,
                    Start,
                    Start,
                    maximum,
                    minimum,
                ),
            ],
            SelfSubtractionY,
            1,
        ),
        (
            SpatialViewportV2::new(0, 0),
            vec![
                root(),
                free(
                    1,
                    0,
                    0,
                    0,
                    Start,
                    Start,
                    SpatialAnchorTargetV2::Viewport,
                    Start,
                    Start,
                    minimum,
                    minimum,
                ),
                free(
                    2,
                    1,
                    0,
                    1,
                    Start,
                    Start,
                    SpatialAnchorTargetV2::Viewport,
                    Start,
                    Start,
                    maximum,
                    maximum,
                ),
            ],
            BaseFarY,
            2,
        ),
    ];

    for (viewport, nodes, operation, node) in cases {
        let fixture = fixture(nodes);
        let engine = ScriptedLayoutEngine::new(Vec::new());
        expect_arithmetic(
            execute_dependency_graph!(&fixture, viewport, limits(), &engine),
            operation,
            node,
        );
    }
}

#[test]
fn each_free_unit_finishes_before_the_next_units_earlier_operation_kind() {
    use SpatialAnchorComponentV2::{End, Start};

    let minimum = SpatialScalarV2::MIN_RAW;
    let maximum = SpatialScalarV2::MAX_RAW;
    let fixture = fixture(vec![
        root(),
        free(
            1,
            0,
            0,
            0,
            Start,
            Start,
            SpatialAnchorTargetV2::Viewport,
            Start,
            Start,
            minimum,
            0,
        ),
        free(
            2,
            1,
            0,
            0,
            Start,
            Start,
            SpatialAnchorTargetV2::Viewport,
            Start,
            Start,
            maximum,
            0,
        ),
        free(
            3,
            0,
            0,
            0,
            Start,
            Start,
            SpatialAnchorTargetV2::Viewport,
            End,
            Start,
            1,
            0,
        ),
    ]);
    let engine = ScriptedLayoutEngine::new(Vec::new());

    expect_arithmetic(
        execute_dependency_graph!(
            &fixture,
            SpatialViewportV2::new(i32::MAX, 0),
            limits(),
            &engine
        ),
        SpatialArithmeticOperationV2::ParentDeltaX,
        2,
    );
}
