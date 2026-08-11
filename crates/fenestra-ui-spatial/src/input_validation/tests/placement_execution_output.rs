use fenestra_ui_layout::prototype::{
    LayoutEngineErrorKindV1, LayoutEngineErrorV1, LayoutErrorLocationV1, LayoutExtentV1,
    LayoutOutputErrorKindV1, LayoutOutputFieldV1,
};

use super::placement_execution_support::{
    ScriptedLayoutEngine, VIEWPORT, expect_layout, fixture, layout, limits, output, root,
};
use crate::error::SpatialErrorLocationV2;
use crate::resolve_error::SpatialLayoutErrorKindV2;

#[test]
fn real_output_failures_preserve_every_kind_and_trusted_record_location() {
    use LayoutOutputErrorKindV1::{FarEdgeArithmetic, KeyMismatch, Negative, RecordCountMismatch};

    let cases = [
        (
            output(&[]),
            RecordCountMismatch,
            SpatialErrorLocationV2::Island { index: 0 },
        ),
        (
            output(&[(0, 0, 0, 20, 20), (99, 0, 0, 1, 1)]),
            KeyMismatch,
            SpatialErrorLocationV2::Node { index: 1 },
        ),
        (
            output(&[(0, 0, 0, 20, 20), (1, -1, 0, 1, 1)]),
            Negative(LayoutOutputFieldV1::X),
            SpatialErrorLocationV2::Node { index: 1 },
        ),
        (
            output(&[(0, 0, 0, 20, 20), (1, 0, -1, 1, 1)]),
            Negative(LayoutOutputFieldV1::Y),
            SpatialErrorLocationV2::Node { index: 1 },
        ),
        (
            output(&[(0, 0, 0, 20, 20), (1, 0, 0, -1, 1)]),
            Negative(LayoutOutputFieldV1::Width),
            SpatialErrorLocationV2::Node { index: 1 },
        ),
        (
            output(&[(0, 0, 0, 20, 20), (1, 0, 0, 1, -1)]),
            Negative(LayoutOutputFieldV1::Height),
            SpatialErrorLocationV2::Node { index: 1 },
        ),
        (
            output(&[(0, 0, 0, 20, 20), (1, i32::MAX, 0, 1, 0)]),
            FarEdgeArithmetic(LayoutExtentV1::Width),
            SpatialErrorLocationV2::Node { index: 1 },
        ),
        (
            output(&[(0, 0, 0, 20, 20), (1, 0, i32::MAX, 0, 1)]),
            FarEdgeArithmetic(LayoutExtentV1::Height),
            SpatialErrorLocationV2::Node { index: 1 },
        ),
        (
            output(&[(0, -1, 0, 20, 20), (1, 0, 0, 1, 1)]),
            Negative(LayoutOutputFieldV1::X),
            SpatialErrorLocationV2::Node { index: 0 },
        ),
    ];

    for (candidate, kind, location) in cases {
        let fixture = fixture(vec![root(), layout(1, 0, 1, 1)]);
        let engine = ScriptedLayoutEngine::new(vec![Ok(candidate)]);
        expect_layout(
            execute_dependency_graph!(&fixture, VIEWPORT, limits(), &engine),
            SpatialLayoutErrorKindV2::Output(kind),
            location,
        );
        assert_eq!(engine.call_count(), 1);
    }
}

#[test]
fn output_validation_completes_keys_then_scalars_then_far_edges_globally() {
    let fixture = fixture(vec![root(), layout(1, 0, 1, 1)]);
    let key_engine =
        ScriptedLayoutEngine::new(vec![Ok(output(&[(0, -1, 0, 20, 20), (99, 0, 0, 1, 1)]))]);
    expect_layout(
        execute_dependency_graph!(&fixture, VIEWPORT, limits(), &key_engine),
        SpatialLayoutErrorKindV2::Output(LayoutOutputErrorKindV1::KeyMismatch),
        SpatialErrorLocationV2::Node { index: 1 },
    );

    let scalar_engine = ScriptedLayoutEngine::new(vec![Ok(output(&[
        (0, i32::MAX, 0, 20, 20),
        (1, 0, 0, 1, -1),
    ]))]);
    expect_layout(
        execute_dependency_graph!(&fixture, VIEWPORT, limits(), &scalar_engine),
        SpatialLayoutErrorKindV2::Output(LayoutOutputErrorKindV1::Negative(
            LayoutOutputFieldV1::Height,
        )),
        SpatialErrorLocationV2::Node { index: 1 },
    );
}

#[test]
fn a_real_engine_failure_uses_the_same_trusted_member_mapping() {
    let fixture = fixture(vec![root(), layout(1, 0, 1, 1)]);
    let engine = ScriptedLayoutEngine::new(vec![Err(LayoutEngineErrorV1::new(
        LayoutEngineErrorKindV1::RejectedInput,
        LayoutErrorLocationV1::InputNode { index: 1 },
    ))]);

    expect_layout(
        execute_dependency_graph!(&fixture, VIEWPORT, limits(), &engine),
        SpatialLayoutErrorKindV2::Engine(LayoutEngineErrorKindV1::RejectedInput),
        SpatialErrorLocationV2::Node { index: 1 },
    );
    assert_eq!(engine.call_count(), 1);
}
