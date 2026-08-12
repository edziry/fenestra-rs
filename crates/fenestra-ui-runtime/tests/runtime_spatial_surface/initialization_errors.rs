use std::sync::Arc;

use fenestra_ui_layout::prototype::LayoutEngineErrorKindV1;
use fenestra_ui_runtime::prototype::{CapacityKind, RuntimeInitializationErrorKind, UiRuntime};
use fenestra_ui_spatial::prototype::{
    SpatialErrorLocationV2, SpatialLimitKindV2, SpatialResolveErrorKindV2, SpatialViewportV2,
};

use crate::RuntimeSpatialErrorV2;
use crate::spatial_support::engine::{EnginePlan, EngineSpy};
use crate::spatial_support::input::{canonical_source, malformed_source, three_node_source};
use crate::spatial_support::program::{MappingPlan, ProgramSpy, SourcePlan};
use crate::spatial_support::{VIEWPORT, limits, nodes_limit, styled_program};
use crate::support::headless::{construction, runtime_capacity};

#[test]
fn logical_capacity_failure_precedes_both_spatial_callbacks() {
    let (program, program_state) = ProgramSpy::new(SourcePlan::Canonical, MappingPlan::Canonical);
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Panic);
    let error = UiRuntime::new_spatial_with_layout_engine(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        limits(),
        runtime_capacity().with_live_nodes(4),
        Box::new(engine),
    )
    .err()
    .expect("logical capacity should fail initialization");

    assert_eq!(
        error.kind(),
        RuntimeInitializationErrorKind::CapacityExceeded(CapacityKind::LiveNodes)
    );
    assert_eq!(program_state.calls(), 0);
    assert_eq!(engine_state.calls(), 0);
}

#[test]
fn viewport_mismatch_precedes_mapping_length_entry_and_raw_input_faults() {
    let foreign = foreign_root();
    let source = malformed_source(SpatialViewportV2::new(91, 70));
    let (program, program_state) =
        ProgramSpy::new(SourcePlan::Exact(source), MappingPlan::ForeignOnly(foreign));
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Panic);
    let error = initialization_error(program, engine, limits());

    assert_eq!(
        error,
        RuntimeInitializationErrorKind::Spatial(RuntimeSpatialErrorV2::ViewportMismatch)
    );
    assert_eq!(program_state.calls(), 1);
    assert_eq!(engine_state.calls(), 0);
}

#[test]
fn mapping_length_mismatch_precedes_entry_and_raw_input_faults() {
    let foreign = foreign_root();
    let source = malformed_source(VIEWPORT);
    let (program, program_state) =
        ProgramSpy::new(SourcePlan::Exact(source), MappingPlan::ForeignOnly(foreign));
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Panic);
    let error = initialization_error(program, engine, limits());

    assert_eq!(
        error,
        RuntimeInitializationErrorKind::Spatial(RuntimeSpatialErrorV2::MappingLengthMismatch)
    );
    assert_eq!(program_state.calls(), 1);
    assert_eq!(engine_state.calls(), 0);
}

#[test]
fn first_missing_logical_mapping_precedes_later_duplicate_and_raw_faults() {
    let foreign = UiRuntime::new(construction(), runtime_capacity())
        .expect("foreign runtime should initialize")
        .committed()
        .root();
    let source = three_node_source(VIEWPORT, true);
    let (program, program_state) = ProgramSpy::new(
        SourcePlan::Exact(source),
        MappingPlan::MissingSecond(foreign),
    );
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Panic);
    let error = initialization_error(program, engine, limits());

    assert_eq!(
        error,
        RuntimeInitializationErrorKind::Spatial(RuntimeSpatialErrorV2::MissingLogicalNode {
            key: fenestra_ui_spatial::prototype::SpatialNodeKeyV2::new(2),
        })
    );
    assert_eq!(program_state.calls(), 1);
    assert_eq!(engine_state.calls(), 0);
}

#[test]
fn duplicate_logical_mapping_reports_the_second_spatial_key_before_raw_faults() {
    let source = three_node_source(VIEWPORT, true);
    let (program, program_state) =
        ProgramSpy::new(SourcePlan::Exact(source), MappingPlan::DuplicateSecond);
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Panic);
    let error = initialization_error(program, engine, limits());

    assert_eq!(
        error,
        RuntimeInitializationErrorKind::Spatial(RuntimeSpatialErrorV2::DuplicateLogicalNode {
            key: fenestra_ui_spatial::prototype::SpatialNodeKeyV2::new(2),
        })
    );
    assert_eq!(program_state.calls(), 1);
    assert_eq!(engine_state.calls(), 0);
}

#[test]
fn resolver_node_limit_is_preserved_and_drops_the_candidate_without_layout() {
    let source = canonical_source(VIEWPORT);
    let weak = Arc::downgrade(&source);
    let (program, program_state) =
        ProgramSpy::new(SourcePlan::Exact(source), MappingPlan::Canonical);
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Panic);
    let error = initialization_error(program, engine, nodes_limit(4));

    let RuntimeInitializationErrorKind::Spatial(RuntimeSpatialErrorV2::Resolve(resolve)) = error
    else {
        panic!("expected a wrapped spatial resolver error");
    };
    assert_eq!(
        resolve.kind(),
        SpatialResolveErrorKindV2::LimitExceeded(SpatialLimitKindV2::Nodes)
    );
    assert_eq!(resolve.location(), SpatialErrorLocationV2::Input);
    assert_eq!(resolve.observed(), Some(5));
    assert_eq!(resolve.maximum(), Some(4));
    assert_eq!(program_state.calls(), 1);
    assert_eq!(engine_state.calls(), 0);
    assert!(weak.upgrade().is_none());
}

#[test]
fn injected_layout_rejection_is_preserved_with_exact_spatial_location() {
    let source = canonical_source(VIEWPORT);
    let weak = Arc::downgrade(&source);
    let (program, program_state) =
        ProgramSpy::new(SourcePlan::Exact(source), MappingPlan::Canonical);
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Reject);
    let error = initialization_error(program, engine, limits());

    let RuntimeInitializationErrorKind::Spatial(RuntimeSpatialErrorV2::Resolve(resolve)) = error
    else {
        panic!("expected a wrapped spatial resolver error");
    };
    assert_eq!(
        resolve.kind(),
        SpatialResolveErrorKindV2::Layout(
            fenestra_ui_spatial::prototype::SpatialLayoutErrorKindV2::Engine(
                LayoutEngineErrorKindV1::RejectedInput,
            ),
        )
    );
    assert_eq!(
        resolve.location(),
        SpatialErrorLocationV2::Node { index: 2 }
    );
    assert_eq!(resolve.observed(), None);
    assert_eq!(resolve.maximum(), None);
    assert_eq!(program_state.calls(), 1);
    assert_eq!(engine_state.calls(), 1);
    assert!(weak.upgrade().is_none());
}

fn initialization_error(
    program: ProgramSpy,
    engine: EngineSpy,
    spatial_limits: fenestra_ui_spatial::prototype::SpatialLimitsV2,
) -> RuntimeInitializationErrorKind {
    UiRuntime::new_spatial_with_layout_engine(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        spatial_limits,
        runtime_capacity(),
        Box::new(engine),
    )
    .err()
    .expect("spatial initialization should fail")
    .kind()
}

fn foreign_root() -> fenestra_ui_runtime::prototype::NodeId {
    UiRuntime::new(construction(), runtime_capacity())
        .expect("foreign runtime should initialize")
        .committed()
        .root()
}
