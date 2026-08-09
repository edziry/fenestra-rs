mod support;

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{
    QueueCapacity, SchedulerAction, SchedulerCapacity, SchedulerErrorKind, SchedulerLane,
    SchedulerTick, UiRuntime, UiScheduler,
};

use support::{WIDTH, capacity, construction, layout};

fn scheduler() -> UiScheduler {
    let runtime = UiRuntime::new(construction(), capacity()).expect("runtime should initialize");
    let scheduler_capacity =
        scheduler_capacity(QueueCapacity::new(1, 40, 8), QueueCapacity::new(2, 80, 8));

    UiScheduler::new(runtime, scheduler_capacity).expect("scheduler capacity should be valid")
}

fn scheduler_capacity(visual: QueueCapacity, in_flight: QueueCapacity) -> SchedulerCapacity {
    SchedulerCapacity::new(
        QueueCapacity::new(1, 80, 8),
        QueueCapacity::new(4, 128, 8),
        visual,
        in_flight,
    )
}

#[test]
fn construction_reserves_visual_and_retained_generation_capacity() {
    for visual in [QueueCapacity::new(0, 40, 8), QueueCapacity::new(1, 39, 8)] {
        let runtime =
            UiRuntime::new(construction(), capacity()).expect("runtime should initialize");
        let error = UiScheduler::new(
            runtime,
            scheduler_capacity(visual, QueueCapacity::new(2, 80, 8)),
        )
        .err()
        .expect("visual lane should be rejected before publication");

        assert_eq!(
            error.kind(),
            SchedulerErrorKind::CapacityTooSmall(SchedulerLane::Visual)
        );
    }

    for controls in [QueueCapacity::new(0, 32, 8), QueueCapacity::new(1, 31, 8)] {
        let runtime =
            UiRuntime::new(construction(), capacity()).expect("runtime should initialize");
        let capacity = SchedulerCapacity::new(
            QueueCapacity::new(1, 80, 8),
            controls,
            QueueCapacity::new(1, 40, 8),
            QueueCapacity::new(2, 80, 8),
        );
        let error = UiScheduler::new(runtime, capacity)
            .err()
            .expect("shutdown reserve should be validated at construction");

        assert_eq!(
            error.kind(),
            SchedulerErrorKind::CapacityTooSmall(SchedulerLane::Controls)
        );
    }

    let runtime = UiRuntime::new(construction(), capacity().with_retained_generations(2))
        .expect("runtime should initialize");
    let error = UiScheduler::new(
        runtime,
        scheduler_capacity(QueueCapacity::new(1, 40, 8), QueueCapacity::new(2, 80, 8)),
    )
    .err()
    .expect("two in-flight generations require one publication edge");

    assert_eq!(error.kind(), SchedulerErrorKind::RetainedGenerationCapacity);
}

#[test]
fn true_noop_schedules_no_visual_work() {
    let mut scheduler = scheduler();
    let before = scheduler.committed();
    let transaction = scheduler.begin_transaction();

    let summary = scheduler
        .commit(transaction, SchedulerTick::new(10))
        .expect("empty transaction should commit");

    assert!(summary.is_empty());
    assert_eq!(summary.generation(), before.generation());
    assert!(before.shares_state_with(&scheduler.committed()));
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(10))
            .expect("action tick should be monotonic"),
        None
    );

    let visual = scheduler.stats().visual();
    assert_eq!(visual.items(), 0);
    assert_eq!(visual.accounted_bytes(), 0);
    assert_eq!(visual.earliest_tick(), None);
    assert_eq!(visual.latest_tick(), None);
}

#[test]
fn scheduler_time_is_monotonic_and_rejection_preserves_state() {
    let mut scheduler = scheduler();
    let transaction = scheduler.begin_transaction();
    scheduler
        .commit(transaction, SchedulerTick::new(10))
        .expect("first tick should be accepted");

    let error = scheduler
        .next_action(SchedulerTick::new(9))
        .expect_err("scheduler tick must not regress");

    assert_eq!(error.kind(), SchedulerErrorKind::TickRegression);
    assert_eq!(scheduler.committed().generation().get(), 0);
    assert_eq!(scheduler.stats().visual().items(), 0);
}

#[test]
fn commits_before_frame_ready_keep_one_request_for_the_latest_generation() {
    let mut scheduler = scheduler();
    let root = scheduler.committed().root();

    for (tick, width, generation) in [(10, 130, 1), (14, 140, 2), (18, 150, 3)] {
        let mut transaction = scheduler.begin_transaction();
        transaction
            .set_property(root, WIDTH, PropertyValue::ScalarI32(width))
            .expect("property write should stage");

        let summary = scheduler
            .commit(transaction, SchedulerTick::new(tick))
            .expect("property write should commit");

        assert!(!summary.is_empty());
        assert_eq!(summary.generation().get(), generation);
        assert_eq!(summary.mutation_count(), 1);
        assert_eq!(summary.invalidation(), layout());

        if generation == 1 {
            assert!(matches!(
                scheduler
                    .next_action(SchedulerTick::new(tick))
                    .expect("action tick should be monotonic"),
                Some(SchedulerAction::RequestFrame)
            ));
        } else {
            assert_eq!(
                scheduler
                    .next_action(SchedulerTick::new(tick))
                    .expect("action tick should be monotonic"),
                None
            );
        }
    }

    assert_eq!(scheduler.committed().generation().get(), 3);
    assert_eq!(
        scheduler.committed().property(root, WIDTH),
        Some(&PropertyValue::ScalarI32(150))
    );

    let visual = scheduler.stats().visual();
    assert_eq!(visual.items(), 1);
    assert_eq!(visual.accounted_bytes(), 40);
    assert_eq!(visual.earliest_tick(), Some(SchedulerTick::new(10)));
    assert_eq!(visual.latest_tick(), Some(SchedulerTick::new(18)));
}
