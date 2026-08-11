use super::placement_execution_support::{
    ScriptedLayoutEngine, expect_arithmetic, fixture, free, limits, root,
};
use crate::model::{
    SpatialAnchorComponentV2, SpatialAnchorTargetV2, SpatialScalarV2, SpatialViewportV2,
};
use crate::numeric_error::SpatialArithmeticOperationV2;

#[test]
fn each_free_placement_arithmetic_stage_reports_its_exact_axis() {
    use SpatialAnchorComponentV2::{End, Start};
    use SpatialArithmeticOperationV2::{
        BaseFarX, BaseFarY, ParentDeltaX, ParentDeltaY, SelfSubtractionX, SelfSubtractionY,
        TargetOffsetX, TargetOffsetY,
    };

    let maximum = SpatialScalarV2::MAX_RAW;
    let minimum = SpatialScalarV2::MIN_RAW;
    let cases = [
        (
            SpatialViewportV2::new(i32::MAX, 0),
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
                    Start,
                    1,
                    0,
                ),
            ],
            TargetOffsetX,
            1,
        ),
        (
            SpatialViewportV2::new(0, i32::MAX),
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
                    End,
                    0,
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
                    0,
                    End,
                    Start,
                    SpatialAnchorTargetV2::Viewport,
                    Start,
                    Start,
                    minimum,
                    0,
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
                    0,
                    1,
                    Start,
                    End,
                    SpatialAnchorTargetV2::Viewport,
                    Start,
                    Start,
                    0,
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
                    1,
                    0,
                    Start,
                    Start,
                    SpatialAnchorTargetV2::Viewport,
                    Start,
                    Start,
                    maximum,
                    0,
                ),
            ],
            BaseFarX,
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
                    1,
                    Start,
                    Start,
                    SpatialAnchorTargetV2::Viewport,
                    Start,
                    Start,
                    0,
                    maximum,
                ),
            ],
            BaseFarY,
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
            ],
            ParentDeltaX,
            2,
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
                    0,
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
                    0,
                    maximum,
                ),
            ],
            ParentDeltaY,
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
        assert_eq!(engine.call_count(), 0);
    }
}
