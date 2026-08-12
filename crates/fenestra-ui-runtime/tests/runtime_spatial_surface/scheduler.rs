use std::ptr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{
    FrameWork, QueueCapacity, SchedulerAction, SchedulerCapacity, SchedulerErrorKind,
    SchedulerInput, SchedulerInputResult, SchedulerTick, TransactionErrorKind, UiRuntime,
    UiScheduler,
};
use fenestra_ui_spatial::prototype::{SpatialNodeKeyV2, SpatialViewportV2};

use crate::RuntimeSpatialErrorV2;
use crate::spatial_support::dynamic::DynamicProgram;
use crate::spatial_support::engine::{EnginePlan, EngineSpy};
use crate::spatial_support::input::{SourceIdentity, canonical_source};
use crate::spatial_support::program::{MappingPlan, ProgramSpy, SourcePlan};
use crate::spatial_support::{VIEWPORT, limits, styled_program};
use crate::support::headless::{FIRST_KEY, ITEMS, SECOND_KEY, WIDTH, runtime_capacity};

fn capacity() -> SchedulerCapacity {
    SchedulerCapacity::new(
        QueueCapacity::new(1, 80, 8),
        QueueCapacity::new(4, 128, 8),
        QueueCapacity::new(1, 40, 8),
        QueueCapacity::new(2, 80, 8),
    )
}

fn commit_width(scheduler: &mut UiScheduler, value: i32, tick: u64) {
    let root = scheduler.committed().root();
    let mut transaction = scheduler.begin_transaction();
    transaction
        .set_property(root, WIDTH, PropertyValue::ScalarI32(value))
        .expect("scheduler property should stage");
    scheduler
        .commit(transaction, SchedulerTick::new(tick))
        .expect("scheduler property should publish");
}

fn request_and_offer(scheduler: &mut UiScheduler, tick: u64) -> FrameWork {
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(tick))
            .expect("request tick should be monotonic"),
        Some(SchedulerAction::RequestFrame)
    );
    assert_eq!(
        scheduler
            .process_input(SchedulerInput::FrameReady, SchedulerTick::new(tick + 1))
            .expect("frame ready should be accepted"),
        SchedulerInputResult::FrameReady
    );
    let Some(SchedulerAction::OfferFrame(work)) = scheduler
        .next_action(SchedulerTick::new(tick + 1))
        .expect("offer tick should be monotonic")
    else {
        panic!("one frame offer should be ready");
    };
    work
}

#[test]
fn rejected_and_replaced_frame_work_retains_its_exact_spatial_generation() {
    let (program, program_state) = DynamicProgram::new();
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Reference);
    let runtime = UiRuntime::new_spatial_with_layout_engine(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        limits(),
        runtime_capacity().with_retained_generations(4),
        Box::new(engine),
    )
    .expect("spatial runtime should initialize");
    let mut scheduler = UiScheduler::new(runtime, capacity()).expect("scheduler should initialize");
    let initial = scheduler.committed();
    let root = initial.root();
    let container = initial.children(root).expect("root should be live")[0];
    let fragment = initial
        .fragment(container, ITEMS)
        .expect("items fragment should be live");
    let first = initial
        .keyed_member(fragment, FIRST_KEY)
        .expect("first item should be live");
    let second = initial
        .keyed_member(fragment, SECOND_KEY)
        .expect("second item should be live");
    drop(initial);

    commit_width(&mut scheduler, 83, 10);
    let offer = request_and_offer(&mut scheduler, 10);
    assert_eq!(program_state.calls(), 2);
    assert_eq!(engine_state.calls(), 2);
    let offered_spatial = offer
        .snapshot()
        .spatial()
        .expect("offered spatial state should exist");
    assert_eq!(offer.accounted_bytes(), 40);
    assert_eq!(offer.generation().get(), 1);
    assert_eq!(
        offer
            .snapshot()
            .keyed_members(fragment)
            .expect("offered items should be live")
            .collect::<Vec<_>>(),
        vec![(FIRST_KEY, first), (SECOND_KEY, second)]
    );
    assert_eq!(
        offered_spatial.logical_node(SpatialNodeKeyV2::new(1)),
        Some(second)
    );
    assert_eq!(
        offered_spatial.logical_node(SpatialNodeKeyV2::new(2)),
        Some(first)
    );
    assert_eq!(
        offered_spatial.spatial_key(second),
        Some(SpatialNodeKeyV2::new(1))
    );
    assert_eq!(
        offered_spatial.spatial_key(first),
        Some(SpatialNodeKeyV2::new(2))
    );

    assert_eq!(
        scheduler
            .process_input(
                SchedulerInput::RejectFrame(offer.id()),
                SchedulerTick::new(12),
            )
            .expect("offer should be rejected"),
        SchedulerInputResult::FrameRejected(offer.id())
    );
    assert_eq!(program_state.calls(), 2);
    assert_eq!(engine_state.calls(), 2);
    let Some(SchedulerAction::OfferFrame(retry)) = scheduler
        .next_action(SchedulerTick::new(12))
        .expect("retry tick should be monotonic")
    else {
        panic!("rejected work should be offered again");
    };
    assert_eq!(program_state.calls(), 2);
    assert_eq!(engine_state.calls(), 2);
    assert!(retry.snapshot().shares_state_with(offer.snapshot()));
    assert!(ptr::eq(
        retry
            .snapshot()
            .spatial()
            .expect("retry spatial state should exist")
            .snapshot(),
        offered_spatial.snapshot()
    ));
    assert_eq!(
        retry
            .snapshot()
            .spatial()
            .expect("retry spatial state should exist")
            .logical_node(SpatialNodeKeyV2::new(1)),
        Some(second)
    );

    assert_eq!(
        scheduler
            .process_input(
                SchedulerInput::RejectFrame(retry.id()),
                SchedulerTick::new(13),
            )
            .expect("retry should be rejected"),
        SchedulerInputResult::FrameRejected(retry.id())
    );
    let mut movement = scheduler.begin_transaction();
    movement
        .move_keyed(fragment, FIRST_KEY, 1)
        .expect("keyed movement should stage");
    scheduler
        .commit(movement, SchedulerTick::new(14))
        .expect("keyed movement should publish");
    let Some(SchedulerAction::OfferFrame(latest)) = scheduler
        .next_action(SchedulerTick::new(14))
        .expect("latest tick should be monotonic")
    else {
        panic!("latest replacement should be offered");
    };

    assert_eq!(latest.generation().get(), 2);
    assert!(latest.snapshot().shares_state_with(&scheduler.committed()));
    assert!(!latest.snapshot().shares_state_with(offer.snapshot()));
    assert!(!ptr::eq(
        latest
            .snapshot()
            .spatial()
            .expect("latest spatial state should exist")
            .snapshot(),
        offered_spatial.snapshot()
    ));
    assert_eq!(offer.generation().get(), 1);
    assert_eq!(retry.generation().get(), 1);
    let latest_spatial = latest
        .snapshot()
        .spatial()
        .expect("latest spatial state should exist");
    assert_eq!(
        latest
            .snapshot()
            .keyed_members(fragment)
            .expect("latest items should be live")
            .collect::<Vec<_>>(),
        vec![(SECOND_KEY, second), (FIRST_KEY, first)]
    );
    assert_eq!(
        latest_spatial.logical_node(SpatialNodeKeyV2::new(1)),
        Some(first)
    );
    assert_eq!(
        latest_spatial.logical_node(SpatialNodeKeyV2::new(2)),
        Some(second)
    );
    assert_eq!(
        latest_spatial.spatial_key(first),
        Some(SpatialNodeKeyV2::new(1))
    );
    assert_eq!(
        latest_spatial.spatial_key(second),
        Some(SpatialNodeKeyV2::new(2))
    );
    assert_eq!(
        offer
            .snapshot()
            .spatial()
            .expect("offered spatial state should remain")
            .logical_node(SpatialNodeKeyV2::new(1)),
        Some(second)
    );
    assert_eq!(
        retry
            .snapshot()
            .spatial()
            .expect("retry spatial state should remain")
            .spatial_key(first),
        Some(SpatialNodeKeyV2::new(2))
    );
    assert_eq!(program_state.calls(), 3);
    assert_eq!(engine_state.calls(), 3);
}

#[test]
fn failed_spatial_rebuild_preserves_already_queued_visual_work() {
    let source = canonical_source(VIEWPORT);
    let (program, program_state) =
        ProgramSpy::new(SourcePlan::Exact(source), MappingPlan::Canonical);
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Reference);
    let runtime = UiRuntime::new_spatial_with_layout_engine(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        limits(),
        runtime_capacity().with_retained_generations(4),
        Box::new(engine),
    )
    .expect("spatial runtime should initialize");
    let mut scheduler = UiScheduler::new(runtime, capacity()).expect("scheduler should initialize");
    commit_width(&mut scheduler, 83, 10);
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(10))
            .expect("request tick should be monotonic"),
        Some(SchedulerAction::RequestFrame)
    );
    let visual_before = scheduler.stats().visual();
    let committed_before = scheduler.committed();
    let mut resize = scheduler.begin_transaction();
    resize
        .resize_spatial(SpatialViewportV2::new(120, 80))
        .expect("resize should stage");
    let error = scheduler
        .commit(resize, SchedulerTick::new(11))
        .expect_err("fixed source viewport should reject rebuild");

    assert_eq!(
        error.kind(),
        SchedulerErrorKind::Transaction(TransactionErrorKind::Spatial(
            RuntimeSpatialErrorV2::ViewportMismatch,
        ))
    );
    assert_eq!(error.operation_index(), None);
    assert!(committed_before.shares_state_with(&scheduler.committed()));
    assert_eq!(scheduler.stats().visual(), visual_before);
    assert_eq!(program_state.calls(), 3);
    assert_eq!(engine_state.calls(), 2);
    assert_eq!(
        scheduler
            .process_input(SchedulerInput::FrameReady, SchedulerTick::new(12))
            .expect("queued request should remain valid"),
        SchedulerInputResult::FrameReady
    );
    let Some(SchedulerAction::OfferFrame(work)) = scheduler
        .next_action(SchedulerTick::new(12))
        .expect("offer tick should be monotonic")
    else {
        panic!("prior visual work should remain queued");
    };
    assert!(work.snapshot().shares_state_with(&committed_before));
}

#[test]
fn offered_work_retains_source_but_not_runtime_callback_boxes() {
    let source = canonical_source(VIEWPORT);
    let identity = SourceIdentity::capture(&source);
    let weak = Arc::downgrade(&source);
    let program_drops = Arc::new(AtomicUsize::new(0));
    let engine_drops = Arc::new(AtomicUsize::new(0));
    let (program, program_state) = ProgramSpy::with_drop_probe(
        SourcePlan::Exact(source),
        MappingPlan::Canonical,
        Arc::clone(&program_drops),
    );
    let (engine, engine_state) =
        EngineSpy::with_drops(EnginePlan::Reference, Arc::clone(&engine_drops));
    let runtime = UiRuntime::new_spatial_with_layout_engine(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        limits(),
        runtime_capacity().with_retained_generations(4),
        Box::new(engine),
    )
    .expect("spatial runtime should initialize");
    let mut scheduler = UiScheduler::new(runtime, capacity()).expect("scheduler should initialize");
    commit_width(&mut scheduler, 83, 10);
    assert_eq!(program_state.calls(), 2);
    assert_eq!(engine_state.calls(), 2);
    let offer = request_and_offer(&mut scheduler, 10);
    assert_eq!(program_state.calls(), 2);
    assert_eq!(engine_state.calls(), 2);
    let facts = program_state.facts();
    let logical = facts.last().expect("latest build facts should exist").nodes;
    let spatial = offer
        .snapshot()
        .spatial()
        .expect("offered spatial state should exist");
    for (key, node) in [
        (1, logical.second_item),
        (2, logical.container),
        (3, logical.first_item),
        (4, logical.control),
    ] {
        let key = SpatialNodeKeyV2::new(key);
        assert_eq!(spatial.logical_node(key), Some(node));
        assert_eq!(spatial.spatial_key(node), Some(key));
    }

    drop(scheduler);
    assert_eq!(program_drops.load(Ordering::SeqCst), 1);
    assert_eq!(engine_drops.load(Ordering::SeqCst), 1);
    let upgraded = weak
        .upgrade()
        .expect("offered frame should retain its spatial source");
    identity.assert_source(&upgraded);
    drop(upgraded);
    drop(offer);
    assert!(weak.upgrade().is_none());
}
