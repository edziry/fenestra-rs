use fenestra_ui_layout::prototype::{
    LayoutEngineErrorKindV1, LayoutEngineErrorV1, LayoutErrorLocationV1, LayoutOutputErrorKindV1,
};

use super::dependency_support::expect_dependency;
use super::placement_execution_support::{
    ScriptedLayoutEngine, VIEWPORT, expect_arithmetic, expect_layout, fixture, free, layout,
    limits, node_target, output, root, start_free,
};
use crate::error::{SpatialDependencyErrorKindV2, SpatialErrorLocationV2};
use crate::model::{
    SpatialAnchorComponentV2, SpatialAnchorTargetV2, SpatialScalarV2, SpatialViewportV2,
};
use crate::numeric_error::SpatialArithmeticOperationV2;
use crate::resolve_error::SpatialLayoutErrorKindV2;

#[test]
fn stable_dependency_order_not_dense_island_order_controls_engine_calls() {
    use SpatialAnchorComponentV2::Start;

    let fixture = fixture(vec![
        root(),
        free(
            1,
            0,
            11,
            12,
            Start,
            Start,
            node_target(4),
            Start,
            Start,
            0,
            0,
        ),
        layout(2, 1, 2, 3),
        layout(3, 0, 4, 5),
        start_free(4, 0, SpatialAnchorTargetV2::Viewport),
    ]);
    let engine = ScriptedLayoutEngine::new(vec![
        Ok(output(&[(0, 0, 0, 20, 20), (1, 1, 2, 4, 5)])),
        Ok(output(&[(0, 0, 0, 11, 12), (1, 3, 4, 2, 3)])),
    ]);
    let proof = super::placement_execution_support::expect_valid(execute_dependency_graph!(
        &fixture,
        VIEWPORT,
        limits(),
        &engine
    ));

    assert_eq!(proof.dependency_order_facts(), vec![3, 4, 1, 2]);
    assert_eq!(
        engine.calls(),
        vec![
            (20, 20, vec![(0, None, 20, 20), (1, Some(0), 4, 5)]),
            (11, 12, vec![(0, None, 11, 12), (1, Some(0), 2, 3)]),
        ]
    );
}

#[test]
fn a_free_failure_keeps_earlier_calls_and_prevents_every_later_island_call() {
    use SpatialAnchorComponentV2::{End, Start};

    let viewport = SpatialViewportV2::new(i32::MAX, 20);
    let fixture = fixture(vec![
        root(),
        layout(1, 0, 1, 1),
        free(
            2,
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
        layout(3, 2, 1, 1),
    ]);
    let engine = ScriptedLayoutEngine::new(vec![Ok(output(&[
        (0, 0, 0, i32::MAX, 20),
        (1, 0, 0, 1, 1),
    ]))]);

    expect_arithmetic(
        execute_dependency_graph!(&fixture, viewport, limits(), &engine),
        SpatialArithmeticOperationV2::TargetOffsetX,
        2,
    );
    assert_eq!(engine.call_count(), 1);
}

#[test]
fn a_free_host_failure_prevents_its_island_from_being_called() {
    use SpatialAnchorComponentV2::{End, Start};

    let viewport = SpatialViewportV2::new(i32::MAX, 0);
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
            End,
            Start,
            1,
            0,
        ),
        layout(2, 1, 1, 1),
    ]);
    let engine = ScriptedLayoutEngine::new(Vec::new());

    expect_arithmetic(
        execute_dependency_graph!(&fixture, viewport, limits(), &engine),
        SpatialArithmeticOperationV2::TargetOffsetX,
        1,
    );
    assert_eq!(engine.call_count(), 0);
}

#[test]
fn a_failing_island_call_is_counted_and_stops_the_remaining_schedule() {
    let fixture = fixture(vec![
        root(),
        layout(1, 0, 1, 1),
        start_free(2, 0, SpatialAnchorTargetV2::Viewport),
        layout(3, 2, 1, 1),
        start_free(4, 0, SpatialAnchorTargetV2::Viewport),
        layout(5, 4, 1, 1),
    ]);
    let engine = ScriptedLayoutEngine::new(vec![
        Ok(output(&[(0, 0, 0, 20, 20), (1, 0, 0, 1, 1)])),
        Err(LayoutEngineErrorV1::new(
            LayoutEngineErrorKindV1::RejectedInput,
            LayoutErrorLocationV1::Input,
        )),
    ]);

    expect_layout(
        execute_dependency_graph!(&fixture, VIEWPORT, limits(), &engine),
        SpatialLayoutErrorKindV2::Engine(LayoutEngineErrorKindV1::RejectedInput),
        SpatialErrorLocationV2::Island { index: 1 },
    );
    assert_eq!(engine.call_count(), 2);
}

#[test]
fn an_output_validation_failure_is_counted_and_stops_the_remaining_schedule() {
    let fixture = fixture(vec![
        root(),
        layout(1, 0, 1, 1),
        start_free(2, 0, SpatialAnchorTargetV2::Viewport),
        layout(3, 2, 1, 1),
        start_free(4, 0, SpatialAnchorTargetV2::Viewport),
        layout(5, 4, 1, 1),
    ]);
    let engine = ScriptedLayoutEngine::new(vec![
        Ok(output(&[(0, 0, 0, 20, 20), (1, 0, 0, 1, 1)])),
        Ok(output(&[(0, 0, 0, 10, 10), (99, 0, 0, 1, 1)])),
    ]);

    expect_layout(
        execute_dependency_graph!(&fixture, VIEWPORT, limits(), &engine),
        SpatialLayoutErrorKindV2::Output(LayoutOutputErrorKindV1::KeyMismatch),
        SpatialErrorLocationV2::Node { index: 3 },
    );
    assert_eq!(engine.call_count(), 2);
}

#[test]
fn a_synthetic_root_failure_is_counted_and_stops_the_remaining_schedule() {
    let fixture = three_island_fixture(0);
    let engine = ScriptedLayoutEngine::new(vec![
        Ok(output(&[(0, 0, 0, 20, 20), (1, 0, 0, 1, 1)])),
        Ok(output(&[(0, 1, 0, 0, 0), (1, 0, 0, 1, 1)])),
    ]);

    expect_layout(
        execute_dependency_graph!(&fixture, VIEWPORT, limits(), &engine),
        SpatialLayoutErrorKindV2::SyntheticRootMismatch(
            fenestra_ui_layout::prototype::LayoutOutputFieldV1::X,
        ),
        SpatialErrorLocationV2::Island { index: 1 },
    );
    assert_eq!(engine.call_count(), 2);
}

#[test]
fn an_island_member_arithmetic_failure_is_counted_and_stops_the_remaining_schedule() {
    let fixture = three_island_fixture(SpatialScalarV2::MAX_RAW);
    let engine = ScriptedLayoutEngine::new(vec![
        Ok(output(&[(0, 0, 0, 20, 20), (1, 0, 0, 1, 1)])),
        Ok(output(&[(0, 0, 0, 0, 0), (1, 0, 0, 1, 0)])),
    ]);

    expect_arithmetic(
        execute_dependency_graph!(&fixture, VIEWPORT, limits(), &engine),
        SpatialArithmeticOperationV2::BaseFarX,
        3,
    );
    assert_eq!(engine.call_count(), 2);
}

#[test]
fn a_complete_dry_graph_failure_makes_zero_layout_calls() {
    let fixture = fixture(vec![
        root(),
        start_free(1, 0, node_target(2)),
        start_free(2, 0, node_target(1)),
    ]);
    let engine = ScriptedLayoutEngine::new(Vec::new());

    expect_dependency(
        execute_dependency_graph!(&fixture, VIEWPORT, limits(), &engine),
        SpatialDependencyErrorKindV2::Cycle,
        SpatialErrorLocationV2::Dependency { ordinal: 1 },
    );
    assert_eq!(engine.call_count(), 0);
}

fn three_island_fixture(second_host_x: i64) -> super::fixture::RawInputFixture {
    use SpatialAnchorComponentV2::Start;

    fixture(vec![
        root(),
        layout(1, 0, 1, 1),
        free(
            2,
            0,
            0,
            0,
            Start,
            Start,
            SpatialAnchorTargetV2::Viewport,
            Start,
            Start,
            second_host_x,
            0,
        ),
        layout(3, 2, 1, 1),
        start_free(4, 0, SpatialAnchorTargetV2::Viewport),
        layout(5, 4, 1, 1),
    ])
}
