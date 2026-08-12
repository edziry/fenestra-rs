use std::ptr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fenestra_ui_runtime::prototype::{NodeId, UiRuntime};
use fenestra_ui_spatial::prototype::{SpatialNodeKeyV2, SpatialScalarV2};

use crate::spatial_support::engine::{EnginePlan, EngineSpy};
use crate::spatial_support::facts::ForeignIds;
use crate::spatial_support::input::{SourceIdentity, canonical_source};
use crate::spatial_support::program::{MappingPlan, ProgramSpy, SourcePlan};
use crate::spatial_support::{VIEWPORT, limits, styled_program};
use crate::support::headless::{ITEMS, construction, exact_style, runtime_capacity};
use crate::support::headless_spec::{HeadlessSpecBuilder, surface};

#[test]
fn spatial_initialization_builds_from_the_complete_styled_logical_view() {
    let foreign = foreign_ids();
    let (program, state) =
        ProgramSpy::with_foreign(SourcePlan::Canonical, MappingPlan::Canonical, foreign);

    let runtime = UiRuntime::new_spatial(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        limits(),
        runtime_capacity(),
    )
    .expect("styled spatial runtime should initialize");

    assert_eq!(runtime.committed().generation().get(), 0);
    assert_eq!(state.calls(), 1);
    state.only_facts().assert_complete_styled_view();
}

#[test]
fn default_spatial_constructor_materializes_exact_reference_geometry() {
    let (program, state) = ProgramSpy::new(SourcePlan::Canonical, MappingPlan::Canonical);
    let runtime = UiRuntime::new_spatial(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        limits(),
        runtime_capacity(),
    )
    .expect("reference spatial runtime should initialize");
    let committed = runtime.committed();
    let spatial = committed.spatial().expect("spatial state should exist");
    let output = spatial.snapshot().output();

    assert_eq!(state.calls(), 1);
    assert!(committed.headless_projection().is_none());
    assert_eq!(spatial.snapshot().viewport(), VIEWPORT);
    assert_eq!(
        output
            .geometry()
            .iter()
            .copied()
            .map(|row| (
                row.key().get(),
                logical(row.base_x()),
                logical(row.base_y()),
                logical(row.base_width()),
                logical(row.base_height()),
            ))
            .collect::<Vec<_>>(),
        vec![
            (0, 0, 0, 90, 70),
            (1, 0, 0, 11, 7),
            (2, 0, 7, 12, 8),
            (3, 0, 15, 13, 9),
            (4, 0, 24, 14, 10),
        ]
    );
    assert!(output.clips().is_empty());
    assert!(output.paints().is_empty());
    assert!(output.hits().is_empty());
    assert!(output.semantics().is_empty());
}

#[test]
fn injected_spatial_engine_receives_one_exact_island_and_owns_its_output() {
    let (program, program_state) = ProgramSpy::new(SourcePlan::Canonical, MappingPlan::Canonical);
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Distinct);
    let runtime = UiRuntime::new_spatial_with_layout_engine(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        limits(),
        runtime_capacity(),
        Box::new(engine),
    )
    .expect("injected spatial runtime should initialize");
    let committed = runtime.committed();
    let geometry = committed
        .spatial()
        .expect("spatial state should exist")
        .snapshot()
        .output()
        .geometry();

    assert_eq!(program_state.calls(), 1);
    assert_eq!(engine_state.calls(), 1);
    assert_eq!(
        engine_state.only_input().nodes,
        vec![
            (0, None, 90, 70),
            (1, Some(0), 11, 7),
            (2, Some(0), 12, 8),
            (3, Some(0), 13, 9),
            (4, Some(0), 14, 10),
        ]
    );
    assert_eq!(engine_state.only_input().viewport, (90, 70));
    assert_eq!(logical(geometry[1].base_x()), 3);
    assert_eq!(logical(geometry[1].base_y()), 4);
    assert_eq!(logical(geometry[4].base_x()), 12);
    assert_eq!(logical(geometry[4].base_y()), 16);
}

#[test]
fn free_only_spatial_initialization_never_calls_the_layout_engine() {
    let (program, program_state) = ProgramSpy::new(SourcePlan::Free, MappingPlan::Free);
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Panic);
    let runtime = UiRuntime::new_spatial_with_layout_engine(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        limits(),
        runtime_capacity(),
        Box::new(engine),
    )
    .expect("free-only spatial runtime should initialize");

    assert_eq!(program_state.calls(), 1);
    assert_eq!(engine_state.calls(), 0);
    assert_eq!(
        runtime
            .committed()
            .spatial()
            .expect("spatial state should exist")
            .snapshot()
            .output()
            .geometry()
            .len(),
        2
    );
}

#[test]
fn accepted_mapping_round_trips_independently_of_both_parent_trees() {
    let source = canonical_source(VIEWPORT);
    let (program, state) = ProgramSpy::new(
        SourcePlan::Exact(Arc::clone(&source)),
        MappingPlan::Canonical,
    );
    let runtime = UiRuntime::new_spatial(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        limits(),
        runtime_capacity(),
    )
    .expect("permuted mapping should initialize");
    let facts = state.only_facts();
    let committed = runtime.committed();
    let spatial = committed.spatial().expect("spatial state should exist");
    let expected = [
        facts.nodes.second_item,
        facts.nodes.container,
        facts.nodes.first_item,
        facts.nodes.control,
    ];

    for (index, logical_node) in expected.into_iter().enumerate() {
        let key = SpatialNodeKeyV2::new(u32::try_from(index + 1).expect("fixture key fits"));
        assert_eq!(spatial.logical_node(key), Some(logical_node));
        assert_eq!(spatial.spatial_key(logical_node), Some(key));
    }
    assert_eq!(spatial.logical_node(SpatialNodeKeyV2::new(0)), None);
    assert_eq!(spatial.logical_node(SpatialNodeKeyV2::new(99)), None);
    assert_eq!(spatial.spatial_key(facts.nodes.root), None);
    assert_eq!(spatial.spatial_key(foreign_root()), None);
    assert_eq!(
        committed.parent(facts.nodes.second_item),
        Some(facts.nodes.container)
    );
    assert_eq!(
        source.as_input().topology().nodes()[1].parent(),
        Some(SpatialNodeKeyV2::new(0))
    );
}

#[test]
fn repeated_spatial_observation_is_borrow_only_and_callback_free() {
    let (program, state) = ProgramSpy::new(SourcePlan::Canonical, MappingPlan::Canonical);
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Reference);
    let runtime = UiRuntime::new_spatial_with_layout_engine(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        limits(),
        runtime_capacity(),
        Box::new(engine),
    )
    .expect("spatial runtime should initialize");
    let committed = runtime.committed();
    let first = committed.spatial().expect("spatial state should exist");
    let second = committed
        .spatial()
        .expect("spatial state should still exist");

    assert!(ptr::eq(first.snapshot(), second.snapshot()));
    assert_eq!(
        first.snapshot().output().geometry().as_ptr(),
        second.snapshot().output().geometry().as_ptr()
    );
    let _ = first.logical_node(SpatialNodeKeyV2::new(1));
    let _ = first.spatial_key(state.only_facts().nodes.control);
    let _ = committed.root();
    let _ = committed.children(committed.root());
    let _ = committed.property(committed.root(), crate::support::headless::WIDTH);
    assert_eq!(state.calls(), 1);
    assert_eq!(engine_state.calls(), 1);
}

#[test]
fn committed_snapshot_retains_the_exact_owned_source_after_runtime_drop() {
    let source = canonical_source(VIEWPORT);
    let identity = SourceIdentity::capture(&source);
    let weak = Arc::downgrade(&source);
    let program_drops = Arc::new(AtomicUsize::new(0));
    let engine_drops = Arc::new(AtomicUsize::new(0));
    let (program, _) = ProgramSpy::with_drop_probe(
        SourcePlan::Exact(source),
        MappingPlan::Canonical,
        Arc::clone(&program_drops),
    );
    let (engine, _) = EngineSpy::with_drops(EnginePlan::Reference, Arc::clone(&engine_drops));
    let runtime = UiRuntime::new_spatial_with_layout_engine(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        limits(),
        runtime_capacity(),
        Box::new(engine),
    )
    .expect("owned spatial source should initialize");
    let committed = runtime.committed();

    drop(runtime);
    assert_eq!(program_drops.load(Ordering::SeqCst), 1);
    assert_eq!(engine_drops.load(Ordering::SeqCst), 1);
    let upgraded = weak
        .upgrade()
        .expect("committed snapshot should retain the source");
    identity.assert_source(&upgraded);
    drop(upgraded);
    drop(committed);
    assert!(weak.upgrade().is_none());
}

#[test]
fn ordinary_headless_and_spatial_observation_modes_are_exclusive() {
    let ordinary = UiRuntime::new(construction(), runtime_capacity())
        .expect("ordinary runtime should initialize");
    let headless = UiRuntime::new_headless(
        exact_style(),
        HeadlessSpecBuilder::new().build(),
        surface(),
        runtime_capacity(),
    )
    .expect("headless runtime should initialize");
    let (program, _) = ProgramSpy::new(SourcePlan::Canonical, MappingPlan::Canonical);
    let spatial = UiRuntime::new_spatial(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        limits(),
        runtime_capacity(),
    )
    .expect("spatial runtime should initialize");

    assert!(ordinary.committed().headless_projection().is_none());
    assert!(ordinary.committed().spatial().is_none());
    assert!(headless.committed().headless_projection().is_some());
    assert!(headless.committed().spatial().is_none());
    assert!(spatial.committed().headless_projection().is_none());
    assert!(spatial.committed().spatial().is_some());
}

fn foreign_ids() -> ForeignIds {
    let runtime = UiRuntime::new(construction(), runtime_capacity())
        .expect("foreign runtime should initialize");
    let committed = runtime.committed();
    let root = committed.root();
    let container = committed
        .children(root)
        .expect("foreign root should be live")[0];
    ForeignIds {
        node: root,
        fragment: committed
            .fragment(container, ITEMS)
            .expect("foreign fragment should be live"),
    }
}

fn foreign_root() -> NodeId {
    UiRuntime::new(construction(), runtime_capacity())
        .expect("foreign runtime should initialize")
        .committed()
        .root()
}

fn logical(value: SpatialScalarV2) -> i64 {
    assert_eq!(value.raw() % SpatialScalarV2::SCALE, 0);
    value.raw() / SpatialScalarV2::SCALE
}
