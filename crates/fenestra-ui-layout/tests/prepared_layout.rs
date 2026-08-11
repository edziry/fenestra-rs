use std::fmt::Debug;
use std::fs;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::path::PathBuf;

use fenestra_ui_layout::prototype::{
    LayoutEngineErrorKindV1, LayoutEngineErrorV1, LayoutEngineV1, LayoutErrorKindV1,
    LayoutErrorLocationV1, LayoutExtentV1, LayoutInputErrorKindV1, LayoutInputV1, LayoutLimitsV1,
    LayoutNodeKeyV1, LayoutOutputErrorKindV1, LayoutOutputFieldV1, LayoutOutputV1, LayoutRecordV1,
    LayoutRectV1, LayoutViewportV1, PreparedLayoutInputV1, compute_layout_v1,
    compute_prepared_layout_v1, prepare_layout_v1,
};

#[path = "prepared_layout/support.rs"]
mod support;

use support::{
    EngineResponse, ScriptedEngine, generous_limits, negative_gap_nodes, output_for,
    thirty_three_nodes, two_nodes,
};

macro_rules! assert_not_clone {
    ($type:ty) => {
        let _ = <$type as AmbiguousIfClone<_>>::marker;
    };
}

macro_rules! assert_not_debug {
    ($type:ty) => {
        let _ = <$type as AmbiguousIfDebug<_>>::marker;
    };
}

#[test]
fn prepared_value_is_owned_opaque_and_runtime_safe() {
    assert_auto_traits::<PreparedLayoutInputV1>();
    assert_not_clone!(PreparedLayoutInputV1);
    assert_not_debug!(PreparedLayoutInputV1);

    let expected_nodes = two_nodes();
    let prepared = {
        let mut authored = expected_nodes.clone();
        let prepared = prepare_ok(
            LayoutInputV1::new(LayoutViewportV1::new(90, 70), &authored),
            generous_limits(),
        );
        authored[0] = support::node(0, None, 1);
        drop(authored);
        prepared
    };

    let engine = ScriptedEngine::new(
        LayoutViewportV1::new(90, 70),
        expected_nodes.clone(),
        EngineResponse::Echo,
    );
    let engine_object: &dyn LayoutEngineV1 = &engine;
    let output = compute_prepared_layout_v1(engine_object, prepared)
        .expect("owned prepared input should remain computable");

    assert_eq!(engine.calls(), 1);
    assert_eq!(output, output_for(&expected_nodes));
}

#[test]
fn prepare_rejects_early_and_last_input_validation_failures() {
    let cases = [
        (
            support::invalid_preorder_nodes(),
            LayoutInputErrorKindV1::InvalidPreorder,
            LayoutErrorLocationV1::InputNode { index: 4 },
        ),
        (
            negative_gap_nodes(),
            LayoutInputErrorKindV1::NegativeGap,
            LayoutErrorLocationV1::InputNode { index: 0 },
        ),
    ];

    for (nodes, expected_kind, expected_location) in cases {
        let input = LayoutInputV1::new(LayoutViewportV1::new(80, 60), &nodes);
        let prepared_error = prepare_error(input, generous_limits());
        assert_eq!(
            prepared_error.kind(),
            LayoutErrorKindV1::Input(expected_kind)
        );
        assert_eq!(prepared_error.location(), expected_location);

        let engine = ScriptedEngine::new(input.viewport(), nodes.clone(), EngineResponse::Echo);
        let one_shot = compute_layout_v1(&engine, input, generous_limits())
            .expect_err("invalid input should fail before engine invocation");
        assert_eq!(engine.calls(), 0);
        assert_eq!(one_shot, prepared_error);
    }
}

#[test]
fn prepared_compute_accepts_caller_derived_limits_above_the_registered_profile() {
    let nodes = thirty_three_nodes();
    let viewport = LayoutViewportV1::new(200, 160);
    let prepared = prepare_ok(
        LayoutInputV1::new(viewport, &nodes),
        LayoutLimitsV1::new(33, 3, 16),
    );
    let engine = ScriptedEngine::new(viewport, nodes.clone(), EngineResponse::Echo);

    let output = compute_prepared_layout_v1(&engine, prepared)
        .expect("caller-derived limits should admit the complete input");

    assert_eq!(engine.calls(), 1);
    assert_eq!(output.records().len(), 33);
    assert_eq!(output, output_for(&nodes));
}

#[test]
fn prepared_compute_calls_once_and_forwards_engine_errors() {
    let nodes = two_nodes();
    let viewport = LayoutViewportV1::new(80, 60);
    let prepared = prepare_ok(LayoutInputV1::new(viewport, &nodes), generous_limits());
    let engine_error = LayoutEngineErrorV1::new(
        LayoutEngineErrorKindV1::RejectedInput,
        LayoutErrorLocationV1::OutputRecord { index: 1 },
    );
    let engine = ScriptedEngine::new(viewport, nodes, EngineResponse::Error(engine_error));

    let error = compute_prepared_layout_v1(&engine, prepared)
        .expect_err("engine failure should cross the prepared boundary");

    assert_eq!(engine.calls(), 1);
    assert_eq!(
        error.kind(),
        LayoutErrorKindV1::Engine(LayoutEngineErrorKindV1::RejectedInput)
    );
    assert_eq!(error.location(), engine_error.location());
}

#[test]
fn prepared_compute_reuses_complete_output_validation() {
    let nodes = two_nodes();
    let viewport = LayoutViewportV1::new(80, 60);
    let valid = output_for(&nodes);
    let cases = [
        (
            LayoutOutputV1::new(vec![valid.records()[0]]),
            LayoutOutputErrorKindV1::RecordCountMismatch,
            LayoutErrorLocationV1::Output,
        ),
        (
            LayoutOutputV1::new(vec![valid.records()[0], valid.records()[0]]),
            LayoutOutputErrorKindV1::KeyMismatch,
            LayoutErrorLocationV1::OutputRecord { index: 1 },
        ),
        (
            LayoutOutputV1::new(vec![
                LayoutRecordV1::new(LayoutNodeKeyV1::new(0), LayoutRectV1::new(-1, 0, 0, 0)),
                valid.records()[1],
            ]),
            LayoutOutputErrorKindV1::Negative(LayoutOutputFieldV1::X),
            LayoutErrorLocationV1::OutputRecord { index: 0 },
        ),
        (
            LayoutOutputV1::new(vec![
                valid.records()[0],
                LayoutRecordV1::new(
                    LayoutNodeKeyV1::new(1),
                    LayoutRectV1::new(i32::MAX, 0, 1, 0),
                ),
            ]),
            LayoutOutputErrorKindV1::FarEdgeArithmetic(LayoutExtentV1::Width),
            LayoutErrorLocationV1::OutputRecord { index: 1 },
        ),
    ];

    for (output, expected_kind, expected_location) in cases {
        let prepared = prepare_ok(LayoutInputV1::new(viewport, &nodes), generous_limits());
        let engine = ScriptedEngine::new(viewport, nodes.clone(), EngineResponse::Output(output));
        let error = compute_prepared_layout_v1(&engine, prepared)
            .expect_err("malformed output should not cross the boundary");

        assert_eq!(engine.calls(), 1);
        assert_eq!(error.kind(), LayoutErrorKindV1::Output(expected_kind));
        assert_eq!(error.location(), expected_location);
    }
}

#[test]
fn one_shot_api_remains_observationally_compatible() {
    let nodes = two_nodes();
    let viewport = LayoutViewportV1::new(80, 60);
    let input = LayoutInputV1::new(viewport, &nodes);
    let one_shot_engine = ScriptedEngine::new(viewport, nodes.clone(), EngineResponse::Echo);
    let prepared_engine = ScriptedEngine::new(viewport, nodes.clone(), EngineResponse::Echo);

    let one_shot = compute_layout_v1(&one_shot_engine, input, generous_limits())
        .expect("one-shot layout should succeed");
    let prepared = prepare_ok(input, generous_limits());
    let split = compute_prepared_layout_v1(&prepared_engine, prepared)
        .expect("prepared layout should succeed");
    assert_eq!(one_shot, split);
    assert_eq!(one_shot_engine.calls(), 1);
    assert_eq!(prepared_engine.calls(), 1);

    let engine_error = LayoutEngineErrorV1::new(
        LayoutEngineErrorKindV1::InvariantViolation,
        LayoutErrorLocationV1::Output,
    );
    let one_shot_engine =
        ScriptedEngine::new(viewport, nodes.clone(), EngineResponse::Error(engine_error));
    let prepared_engine =
        ScriptedEngine::new(viewport, nodes.clone(), EngineResponse::Error(engine_error));
    let one_shot_error = compute_layout_v1(&one_shot_engine, input, generous_limits())
        .expect_err("one-shot engine error should propagate");
    let prepared = prepare_ok(input, generous_limits());
    let split_error = compute_prepared_layout_v1(&prepared_engine, prepared)
        .expect_err("prepared engine error should propagate");
    assert_eq!(one_shot_error, split_error);
    assert_eq!(one_shot_engine.calls(), 1);
    assert_eq!(prepared_engine.calls(), 1);
}

#[test]
fn prepared_surface_and_delegation_remain_exact() {
    let source_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let engine = fs::read_to_string(source_dir.join("engine.rs")).expect("read engine source");
    let proof = item_block(&engine, "pub struct PreparedLayoutInputV1");
    let fields: Vec<_> = proof[1..proof.len() - 1]
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    assert_eq!(
        fields,
        ["viewport: LayoutViewportV1,", "nodes: Vec<LayoutNodeV1>,"]
    );
    assert!(!engine.contains("impl PreparedLayoutInputV1"));

    let prepare = item_block(&engine, "pub fn prepare_layout_v1");
    assert_eq!(prepare.matches("validate_input_v1").count(), 1);
    assert!(prepare.contains("input.nodes().to_vec()"));
    assert!(
        prepare.find("validate_input_v1").expect("validation")
            < prepare.find("input.nodes().to_vec()").expect("owned copy")
    );
    assert!(!prepare.contains(".compute("));
    assert!(!prepare.contains("validate_output_v1"));

    let compute_prepared = item_block(&engine, "pub fn compute_prepared_layout_v1");
    assert!(!compute_prepared.contains("validate_input_v1"));
    assert_eq!(compute_prepared.matches(".compute(").count(), 1);
    assert_eq!(compute_prepared.matches("validate_output_v1").count(), 1);

    let one_shot = item_block(&engine, "pub fn compute_layout_v1");
    assert_eq!(one_shot.matches("prepare_layout_v1").count(), 1);
    assert_eq!(one_shot.matches("compute_prepared_layout_v1").count(), 1);
    for forbidden in ["validate_input_v1", "validate_output_v1", ".compute("] {
        assert!(!one_shot.contains(forbidden), "duplicated step {forbidden}");
    }

    let library = fs::read_to_string(source_dir.join("lib.rs")).expect("read library source");
    let engine_exports = reexported_names(&library, "pub use crate::engine::{");
    assert_eq!(
        engine_exports,
        [
            "LayoutEngineV1",
            "PreparedLayoutInputV1",
            "ReferenceStackEngineV1",
            "ValidatedLayoutInputV1",
            "compute_layout_v1",
            "compute_prepared_layout_v1",
            "prepare_layout_v1",
        ]
    );
}

fn prepare_ok(input: LayoutInputV1<'_>, limits: LayoutLimitsV1) -> PreparedLayoutInputV1 {
    match prepare_layout_v1(input, limits) {
        Ok(prepared) => prepared,
        Err(error) => panic!("valid input failed preparation: {error}"),
    }
}

fn prepare_error(
    input: LayoutInputV1<'_>,
    limits: LayoutLimitsV1,
) -> fenestra_ui_layout::prototype::LayoutErrorV1 {
    match prepare_layout_v1(input, limits) {
        Ok(_) => panic!("invalid input crossed preparation"),
        Err(error) => error,
    }
}

fn assert_auto_traits<T>()
where
    T: Send + Sync + Unpin + UnwindSafe + RefUnwindSafe + 'static,
{
}

fn item_block<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing source item {marker}"));
    let item = &source[start..];
    let open = item.find('{').expect("source item body");
    let mut depth = 0_usize;
    for (offset, character) in item[open..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &item[open..open + offset + 1];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated source item {marker}")
}

fn reexported_names<'a>(source: &'a str, marker: &str) -> Vec<&'a str> {
    let start = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing reexport group {marker}"));
    let group = &source[start + marker.len()..];
    let end = group.find("};").expect("terminated reexport group");
    group[..end]
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .collect()
}

trait AmbiguousIfClone<A> {
    fn marker() {}
}

impl<T: ?Sized> AmbiguousIfClone<()> for T {}
impl<T: ?Sized + Clone> AmbiguousIfClone<u8> for T {}

trait AmbiguousIfDebug<A> {
    fn marker() {}
}

impl<T: ?Sized> AmbiguousIfDebug<()> for T {}
impl<T: ?Sized + Debug> AmbiguousIfDebug<u8> for T {}
