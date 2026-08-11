use fenestra_ui_layout::prototype::{LayoutOutputErrorKindV1, LayoutOutputFieldV1};

use super::placement_execution_support::{
    ScriptedLayoutEngine, VIEWPORT, expect_arithmetic, expect_layout, expect_valid, fixture, free,
    layout, limits, logical, output, placement, root, start_free,
};
use crate::error::SpatialErrorLocationV2;
use crate::model::{
    SpatialAnchorComponentV2, SpatialAnchorTargetV2, SpatialScalarV2, SpatialViewportV2,
};
use crate::numeric_error::SpatialArithmeticOperationV2;
use crate::resolve_error::SpatialLayoutErrorKindV2;

#[test]
fn island_member_arithmetic_reports_translation_then_far_edge_in_axis_order() {
    use SpatialAnchorComponentV2::Start;
    use SpatialArithmeticOperationV2::{
        BaseFarX, BaseFarY, IslandTranslationX, IslandTranslationY,
    };

    let maximum = SpatialScalarV2::MAX_RAW;
    let cases = [
        (maximum, 0, (1, 0, 0, 0), IslandTranslationX),
        (0, maximum, (0, 1, 0, 0), IslandTranslationY),
        (maximum, 0, (0, 0, 1, 0), BaseFarX),
        (0, maximum, (0, 0, 0, 1), BaseFarY),
    ];

    for (host_x, host_y, member, operation) in cases {
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
                host_x,
                host_y,
            ),
            layout(2, 1, 0, 0),
        ]);
        let engine = ScriptedLayoutEngine::new(vec![Ok(output(&[
            (0, 0, 0, 0, 0),
            (1, member.0, member.1, member.2, member.3),
        ]))]);

        expect_arithmetic(
            execute_dependency_graph!(&fixture, VIEWPORT, limits(), &engine),
            operation,
            2,
        );
        assert_eq!(engine.call_count(), 1);
    }
}

#[test]
fn one_island_member_finishes_translation_axes_before_far_edge_axes() {
    use SpatialAnchorComponentV2::Start;
    use SpatialArithmeticOperationV2::{BaseFarX, IslandTranslationX, IslandTranslationY};

    let maximum = SpatialScalarV2::MAX_RAW;
    let cases = [
        ((1, 1, 0, 0), IslandTranslationX),
        ((0, 1, 1, 0), IslandTranslationY),
        ((0, 0, 1, 1), BaseFarX),
    ];
    for (member, operation) in cases {
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
                maximum,
                maximum,
            ),
            layout(2, 1, 0, 0),
        ]);
        let engine = ScriptedLayoutEngine::new(vec![Ok(output(&[
            (0, 0, 0, 0, 0),
            (1, member.0, member.1, member.2, member.3),
        ]))]);

        expect_arithmetic(
            execute_dependency_graph!(&fixture, VIEWPORT, limits(), &engine),
            operation,
            2,
        );
    }
}

#[test]
fn an_earlier_member_completes_all_arithmetic_before_a_later_member() {
    use SpatialAnchorComponentV2::Start;

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
            maximum,
            maximum,
        ),
        layout(2, 1, 0, 1),
        layout(3, 1, 0, 0),
    ]);
    let engine = ScriptedLayoutEngine::new(vec![Ok(output(&[
        (0, 0, 0, 0, 0),
        (1, 0, 0, 0, 1),
        (2, 1, 0, 0, 0),
    ]))]);

    expect_arithmetic(
        execute_dependency_graph!(&fixture, VIEWPORT, limits(), &engine),
        SpatialArithmeticOperationV2::BaseFarY,
        2,
    );
}

#[test]
fn island_parent_deltas_reach_both_scalar_boundaries_without_an_extra_failure() {
    use SpatialAnchorComponentV2::Start;

    let minimum = SpatialScalarV2::MIN_RAW;
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
            minimum,
        ),
        layout(2, 1, 0, 0),
        layout(3, 2, 0, 0),
    ]);
    let engine = ScriptedLayoutEngine::new(vec![Ok(output(&[
        (0, 0, 0, 0, 0),
        (1, i32::MAX, i32::MAX, 0, 0),
        (2, 0, 0, 0, 0),
    ]))]);
    let proof = expect_valid(execute_dependency_graph!(
        &fixture,
        SpatialViewportV2::new(0, 0),
        limits(),
        &engine
    ));

    assert_eq!(
        proof.placement_facts(),
        vec![
            placement(0, 0, 0, 0, 0, 0, 0, 0, 0),
            placement(
                1, minimum, minimum, 0, 0, minimum, minimum, minimum, minimum
            ),
            placement(
                2,
                0,
                0,
                0,
                0,
                0,
                0,
                SpatialScalarV2::MAX_RAW,
                SpatialScalarV2::MAX_RAW
            ),
            placement(
                3, minimum, minimum, 0, 0, minimum, minimum, minimum, minimum
            ),
        ]
    );
}

#[test]
fn synthetic_root_mismatches_use_exact_field_priority_and_island_location() {
    use LayoutOutputFieldV1::{Height, Width, X, Y};
    use SpatialAnchorComponentV2::Start;

    let cases = [
        ((1, 0, 10, 20), X),
        ((0, 1, 10, 20), Y),
        ((0, 0, 9, 20), Width),
        ((0, 0, 10, 19), Height),
        ((1, 1, 9, 19), X),
        ((0, 1, 9, 19), Y),
        ((0, 0, 9, 19), Width),
    ];
    for ((x, y, width, height), field) in cases {
        let fixture = fixture(vec![
            root(),
            free(
                1,
                0,
                10,
                20,
                Start,
                Start,
                SpatialAnchorTargetV2::Viewport,
                Start,
                Start,
                0,
                0,
            ),
            layout(2, 1, 1, 1),
        ]);
        let engine = ScriptedLayoutEngine::new(vec![Ok(output(&[
            (0, x, y, width, height),
            (1, 0, 0, 1, 1),
        ]))]);
        expect_layout(
            execute_dependency_graph!(&fixture, VIEWPORT, limits(), &engine),
            SpatialLayoutErrorKindV2::SyntheticRootMismatch(field),
            SpatialErrorLocationV2::Island { index: 0 },
        );
        assert_eq!(engine.call_count(), 1);
    }
}

#[test]
fn neutral_output_validation_precedes_root_comparison_and_member_translation_follows_it() {
    use SpatialAnchorComponentV2::Start;

    let invalid_fixture = fixture(vec![root(), layout(1, 0, 1, 1)]);
    let negative_engine =
        ScriptedLayoutEngine::new(vec![Ok(output(&[(0, 1, 0, 20, 20), (1, 0, 0, 1, -1)]))]);
    expect_layout(
        execute_dependency_graph!(&invalid_fixture, VIEWPORT, limits(), &negative_engine),
        SpatialLayoutErrorKindV2::Output(LayoutOutputErrorKindV1::Negative(
            LayoutOutputFieldV1::Height,
        )),
        SpatialErrorLocationV2::Node { index: 1 },
    );

    let maximum = SpatialScalarV2::MAX_RAW;
    let poisoned = fixture(vec![
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
            maximum,
            0,
        ),
        layout(2, 1, 0, 0),
    ]);
    let mismatch_engine =
        ScriptedLayoutEngine::new(vec![Ok(output(&[(0, 1, 0, 0, 0), (1, 1, 0, 0, 0)]))]);
    expect_layout(
        execute_dependency_graph!(&poisoned, VIEWPORT, limits(), &mismatch_engine),
        SpatialLayoutErrorKindV2::SyntheticRootMismatch(LayoutOutputFieldV1::X),
        SpatialErrorLocationV2::Island { index: 0 },
    );
}

#[test]
fn a_valid_layout_output_translates_to_exact_fixed_point_facts() {
    let fixture = fixture(vec![root(), layout(1, 0, 3, 4)]);
    let engine = ScriptedLayoutEngine::new(vec![Ok(output(&[(0, 0, 0, 20, 20), (1, 7, 8, 3, 4)]))]);
    let proof = expect_valid(execute_dependency_graph!(
        &fixture,
        VIEWPORT,
        limits(),
        &engine
    ));

    assert_eq!(
        proof.placement_facts(),
        vec![
            placement(0, 0, 0, 20, 20, logical(20), logical(20), 0, 0),
            placement(
                1,
                logical(7),
                logical(8),
                3,
                4,
                logical(10),
                logical(12),
                logical(7),
                logical(8),
            ),
        ]
    );
}

#[test]
fn execution_uses_the_retained_noncontiguous_member_remap() {
    let fixture = fixture(vec![
        root(),
        layout(1, 0, 4, 5),
        start_free(2, 1, SpatialAnchorTargetV2::Parent),
        layout(3, 1, 1, 2),
    ]);
    let engine = ScriptedLayoutEngine::new(vec![Ok(output(&[
        (0, 0, 0, 20, 20),
        (1, 2, 3, 4, 5),
        (2, 7, 8, 1, 2),
    ]))]);
    let proof = expect_valid(execute_dependency_graph!(
        &fixture,
        VIEWPORT,
        limits(),
        &engine
    ));

    assert_eq!(proof.prepared_island_facts(), vec![(0, vec![0, 1, 3])]);
    assert_eq!(
        proof.placement_facts(),
        vec![
            placement(0, 0, 0, 20, 20, logical(20), logical(20), 0, 0),
            placement(
                1,
                logical(2),
                logical(3),
                4,
                5,
                logical(6),
                logical(8),
                logical(2),
                logical(3)
            ),
            placement(
                2,
                logical(2),
                logical(3),
                10,
                10,
                logical(12),
                logical(13),
                0,
                0
            ),
            placement(
                3,
                logical(7),
                logical(8),
                1,
                2,
                logical(8),
                logical(10),
                logical(5),
                logical(5)
            ),
        ]
    );
}
