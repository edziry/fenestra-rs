#![allow(dead_code)]

use fenestra_ui_ir::prototype::{InvalidationSet, PropertyId, PropertyValue};
use fenestra_ui_runtime::prototype::{
    FrameWork, QueueCapacity, RuntimeCapacity, SchedulerAction, SchedulerCapacity, SchedulerInput,
    SchedulerInputResult, SchedulerTick, UiRuntime, UiScheduler,
};
use fenestra_ui_testkit::prototype::RuntimeOracleFixtureV1;

const WIDTH: PropertyId = PropertyId::new(0);

pub fn scheduler() -> UiScheduler {
    scheduler_with_controls(QueueCapacity::new(4, 128, 100))
}

pub fn scheduler_with_controls(controls: QueueCapacity) -> UiScheduler {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let runtime = UiRuntime::new(
        fixture.construction().clone(),
        RuntimeCapacity::new(4, 64, 256, 128, 1_024, 3),
    )
    .expect("runtime should initialize");
    UiScheduler::new(
        runtime,
        SchedulerCapacity::new(
            QueueCapacity::new(1, 80, 100),
            controls,
            QueueCapacity::new(1, 40, 100),
            QueueCapacity::new(2, 80, 100),
        ),
    )
    .expect("scheduler capacity should be valid")
}

pub fn offer_width(scheduler: &mut UiScheduler, width: i32, tick: u64) -> FrameWork {
    let root = scheduler.committed().root();
    let mut transaction = scheduler.begin_transaction();
    transaction
        .set_property(root, WIDTH, PropertyValue::ScalarI32(width))
        .expect("property write should stage");
    let commit = scheduler
        .commit(transaction, SchedulerTick::new(tick))
        .expect("property write should commit");
    assert_eq!(commit.mutation_count(), 1);
    assert_ne!(commit.invalidation(), InvalidationSet::NONE);
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(tick))
            .expect("frame request should advance"),
        Some(SchedulerAction::RequestFrame)
    );
    assert_eq!(
        scheduler
            .process_input(SchedulerInput::FrameReady, SchedulerTick::new(tick))
            .expect("frame-ready input should be accepted"),
        SchedulerInputResult::FrameReady
    );
    let Some(SchedulerAction::OfferFrame(work)) = scheduler
        .next_action(SchedulerTick::new(tick))
        .expect("offer should advance")
    else {
        panic!("one frame offer should be emitted");
    };
    work
}

pub fn next_offer(scheduler: &mut UiScheduler, tick: u64) -> FrameWork {
    let Some(SchedulerAction::OfferFrame(work)) = scheduler
        .next_action(SchedulerTick::new(tick))
        .expect("retry offer should advance")
    else {
        panic!("one retry offer should be emitted");
    };
    work
}

pub fn process_control(scheduler: &mut UiScheduler, tick: u64) {
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(tick))
            .expect("control turn should advance"),
        None
    );
}
