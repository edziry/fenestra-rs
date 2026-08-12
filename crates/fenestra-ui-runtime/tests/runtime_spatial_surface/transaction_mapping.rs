use std::ptr;

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{NodeId, TransactionErrorKind, UiRuntime};
use fenestra_ui_spatial::prototype::SpatialNodeKeyV2;

use crate::spatial_support::engine::{EnginePlan, EngineSpy};
use crate::spatial_support::program::{MappingPlan, ProgramSpy, SourcePlan};
use crate::spatial_support::{VIEWPORT, limits, styled_program};
use crate::support::headless::{WIDTH, construction, runtime_capacity};
use crate::{RuntimeSpatialErrorV2, RuntimeSpatialViewV2};

#[test]
fn mapping_length_mismatch_precedes_entry_and_raw_faults_during_rebuild() {
    let foreign = foreign_root();
    assert_mapping_failure(
        SourcePlan::MalformedCanonicalOnCall(2),
        MappingPlan::ForeignOnlyOnCall(2, foreign),
        RuntimeSpatialErrorV2::MappingLengthMismatch,
    );
}

#[test]
fn first_missing_logical_mapping_precedes_later_duplicate_and_raw_faults_during_rebuild() {
    let foreign = foreign_root();
    assert_mapping_failure(
        SourcePlan::MalformedThreeOnCall(2),
        MappingPlan::MissingSecondOnCall(2, foreign),
        RuntimeSpatialErrorV2::MissingLogicalNode {
            key: SpatialNodeKeyV2::new(2),
        },
    );
}

#[test]
fn duplicate_logical_mapping_reports_the_second_key_before_raw_faults_during_rebuild() {
    assert_mapping_failure(
        SourcePlan::MalformedThreeOnCall(2),
        MappingPlan::DuplicateSecondOnCall(2),
        RuntimeSpatialErrorV2::DuplicateLogicalNode {
            key: SpatialNodeKeyV2::new(2),
        },
    );
}

fn assert_mapping_failure(
    source: SourcePlan,
    mapping: MappingPlan,
    expected: RuntimeSpatialErrorV2,
) {
    let (program, program_state) = ProgramSpy::new(source, mapping);
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Reference);
    let mut runtime = UiRuntime::new_spatial_with_layout_engine(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        limits(),
        runtime_capacity(),
        Box::new(engine),
    )
    .expect("spatial runtime should initialize with a valid mapping");
    let before = runtime.committed();
    let before_spatial = before.spatial().expect("spatial state should exist");
    let initial = program_state.only_facts();
    let expected_mapping = canonical_mapping(&initial);
    assert_exact_mapping(&before_spatial, &expected_mapping, initial.nodes.root);
    let program_before = program_state.calls();
    let engine_before = engine_state.calls();

    let error = runtime
        .commit(change_root(&runtime, 101))
        .expect_err("scripted mapping fault should reject the rebuild");

    assert_eq!(error.kind(), TransactionErrorKind::Spatial(expected));
    assert_eq!(error.operation_index(), None);
    assert_eq!(program_state.calls() - program_before, 1);
    assert_eq!(engine_state.calls() - engine_before, 0);
    assert_eq!(
        program_state.facts()[1].root_width,
        Some(PropertyValue::ScalarI32(101))
    );
    let after_failure = runtime.committed();
    let after_failure_spatial = after_failure
        .spatial()
        .expect("spatial state should remain after rollback");
    assert!(before.shares_state_with(&after_failure));
    assert!(ptr::eq(
        before_spatial.snapshot(),
        after_failure_spatial.snapshot()
    ));
    assert_eq!(
        after_failure.property(after_failure.root(), WIDTH),
        Some(&PropertyValue::ScalarI32(100))
    );
    assert_exact_mapping(
        &after_failure_spatial,
        &expected_mapping,
        initial.nodes.root,
    );

    drop(
        runtime
            .commit(change_root(&runtime, 101))
            .expect("a valid retry should publish"),
    );
    let after_retry = runtime.committed();
    let after_retry_spatial = after_retry
        .spatial()
        .expect("spatial state should remain after retry");
    assert_eq!(program_state.calls() - program_before, 2);
    assert_eq!(engine_state.calls() - engine_before, 1);
    assert_eq!(
        after_retry.generation().get(),
        before.generation().get() + 1
    );
    assert!(!before.shares_state_with(&after_retry));
    assert!(!ptr::eq(
        before_spatial.snapshot(),
        after_retry_spatial.snapshot()
    ));
    assert_eq!(
        after_retry.property(after_retry.root(), WIDTH),
        Some(&PropertyValue::ScalarI32(101))
    );
    assert_exact_mapping(&after_retry_spatial, &expected_mapping, initial.nodes.root);
}

fn change_root(runtime: &UiRuntime, value: i32) -> fenestra_ui_runtime::prototype::UiTransaction {
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(
            runtime.committed().root(),
            WIDTH,
            PropertyValue::ScalarI32(value),
        )
        .expect("property change should stage");
    transaction
}

fn canonical_mapping(facts: &crate::spatial_support::facts::BuildFacts) -> [NodeId; 4] {
    [
        facts.nodes.second_item,
        facts.nodes.container,
        facts.nodes.first_item,
        facts.nodes.control,
    ]
}

fn assert_exact_mapping(
    spatial: &RuntimeSpatialViewV2<'_>,
    expected: &[NodeId; 4],
    unmapped_root: NodeId,
) {
    for (index, &logical_node) in expected.iter().enumerate() {
        let key = SpatialNodeKeyV2::new(u32::try_from(index + 1).expect("fixture key should fit"));
        assert_eq!(spatial.logical_node(key), Some(logical_node));
        assert_eq!(spatial.spatial_key(logical_node), Some(key));
    }
    assert_eq!(spatial.logical_node(SpatialNodeKeyV2::new(0)), None);
    assert_eq!(spatial.logical_node(SpatialNodeKeyV2::new(5)), None);
    assert_eq!(spatial.spatial_key(unmapped_root), None);
}

fn foreign_root() -> NodeId {
    UiRuntime::new(construction(), runtime_capacity())
        .expect("foreign runtime should initialize")
        .committed()
        .root()
}
