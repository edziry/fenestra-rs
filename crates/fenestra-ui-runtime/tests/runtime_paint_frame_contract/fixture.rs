use fenestra_ui_ir::prototype::{PropertyId, PropertyValue};
use fenestra_ui_runtime::prototype::{
    FrameWork, QueueCapacity, SchedulerAction, SchedulerCapacity, SchedulerInput,
    SchedulerInputResult, SchedulerTick, UiRuntime, UiScheduler,
};

use crate::spatial_support::program::{MappingPlan, ProgramSpy, SourcePlan};
use crate::spatial_support::{VIEWPORT, limits, styled_program};
use crate::support::headless::{exact_style, runtime_capacity};
use crate::support::headless_spec::{HeadlessSpecBuilder, surface};

pub(super) fn ordinary_scheduler() -> UiScheduler {
    let runtime = UiRuntime::new(crate::support::construction(), crate::support::capacity())
        .expect("ordinary runtime should initialize");
    scheduler(runtime)
}

pub(super) fn headless_scheduler() -> UiScheduler {
    let runtime = UiRuntime::new_headless(
        exact_style(),
        HeadlessSpecBuilder::new().build(),
        surface(),
        runtime_capacity(),
    )
    .expect("headless runtime should initialize");
    scheduler(runtime)
}

pub(super) fn spatial_scheduler() -> UiScheduler {
    let (program, _) = ProgramSpy::new(SourcePlan::FreshCanonical, MappingPlan::Canonical);
    let runtime = UiRuntime::new_spatial(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        limits(),
        runtime_capacity().with_retained_generations(4),
    )
    .expect("spatial runtime should initialize");
    scheduler(runtime)
}

pub(super) fn commit_root_property(
    scheduler: &mut UiScheduler,
    property: PropertyId,
    value: i32,
    tick: u64,
) {
    let root = scheduler.committed().root();
    let mut transaction = scheduler.begin_transaction();
    transaction
        .set_property(root, property, PropertyValue::ScalarI32(value))
        .expect("root property should stage");
    scheduler
        .commit(transaction, SchedulerTick::new(tick))
        .expect("root property should publish");
}

pub(super) fn request_and_offer(scheduler: &mut UiScheduler, tick: u64) -> FrameWork {
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
    take_offer(scheduler, tick + 1)
}

pub(super) fn take_offer(scheduler: &mut UiScheduler, tick: u64) -> FrameWork {
    let Some(SchedulerAction::OfferFrame(work)) = scheduler
        .next_action(SchedulerTick::new(tick))
        .expect("offer tick should be monotonic")
    else {
        panic!("one frame offer should be ready");
    };
    work
}

pub(super) fn reject(scheduler: &mut UiScheduler, work: &FrameWork, tick: u64) {
    assert_eq!(
        scheduler
            .process_input(
                SchedulerInput::RejectFrame(work.id()),
                SchedulerTick::new(tick),
            )
            .expect("frame rejection should be accepted"),
        SchedulerInputResult::FrameRejected(work.id())
    );
}

fn scheduler(runtime: UiRuntime) -> UiScheduler {
    let capacity = SchedulerCapacity::new(
        QueueCapacity::new(1, 80, 8),
        QueueCapacity::new(4, 128, 8),
        QueueCapacity::new(1, 40, 8),
        QueueCapacity::new(2, 80, 8),
    );
    UiScheduler::new(runtime, capacity).expect("scheduler should initialize")
}
