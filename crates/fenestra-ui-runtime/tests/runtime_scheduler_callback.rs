mod support;

use std::panic::{AssertUnwindSafe, catch_unwind};

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{
    CallbackFinish, ControlAdmission, QueueCapacity, SchedulerAction, SchedulerCapacity,
    SchedulerErrorKind, SchedulerInput, SchedulerInputResult, SchedulerLane, SchedulerState,
    SchedulerTick, UiRuntime, UiScheduler,
};

use support::{WIDTH, capacity, construction};

fn scheduler(deferred: QueueCapacity) -> UiScheduler {
    let runtime = UiRuntime::new(construction(), capacity().with_retained_generations(3))
        .expect("runtime should initialize");
    let capacity = SchedulerCapacity::new(
        deferred,
        QueueCapacity::new(4, 128, 8),
        QueueCapacity::new(1, 40, 8),
        QueueCapacity::new(2, 80, 8),
    );
    UiScheduler::new(runtime, capacity).expect("scheduler capacity should be valid")
}

#[test]
fn nested_callback_uses_one_snapshot_and_publishes_only_on_a_later_turn() {
    let mut scheduler = scheduler(QueueCapacity::new(1, 80, 8));
    let before = scheduler.committed();
    let root = before.root();

    let mut scope = scheduler
        .begin_callback(SchedulerTick::new(10))
        .expect("outer callback should begin");
    assert_eq!(scope.depth(), 1);
    assert!(scope.committed().shares_state_with(&before));
    {
        let mut nested = scope.begin_nested();
        assert_eq!(nested.depth(), 2);
        assert!(nested.committed().shares_state_with(&before));
        {
            let mut grandchild = nested.begin_nested();
            assert_eq!(grandchild.depth(), 3);
            assert!(grandchild.committed().shares_state_with(&before));
            grandchild
                .transaction()
                .set_property(root, WIDTH, PropertyValue::ScalarI32(130))
                .expect("nested property write should stage");
        }
    }
    assert_eq!(
        scope.committed().property(root, WIDTH),
        Some(&PropertyValue::ScalarI32(120))
    );
    assert_eq!(
        scope.finish().expect("callback should enter deferred lane"),
        CallbackFinish::Deferred {
            operation_count: 1,
            accounted_bytes: 80,
        }
    );

    assert!(scheduler.committed().shares_state_with(&before));
    assert_eq!(scheduler.stats().deferred().items(), 1);
    assert_eq!(scheduler.stats().deferred().accounted_bytes(), 80);
    assert_eq!(
        scheduler.stats().deferred().earliest_tick(),
        Some(SchedulerTick::new(10))
    );

    let mut ordinary = scheduler.begin_transaction();
    ordinary
        .set_property(root, WIDTH, PropertyValue::ScalarI32(140))
        .expect("ordinary property write should stage");
    let error = scheduler
        .commit(ordinary, SchedulerTick::new(11))
        .expect_err("ordinary commit must not overtake deferred work");
    assert_eq!(error.kind(), SchedulerErrorKind::ControlPending);
    let error = scheduler
        .begin_callback(SchedulerTick::new(11))
        .err()
        .expect("a second callback must not create another stale base");
    assert_eq!(error.kind(), SchedulerErrorKind::ControlPending);

    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(11))
            .expect("later scheduler turn should publish deferred work"),
        Some(SchedulerAction::RequestFrame)
    );
    assert_eq!(scheduler.stats().deferred().items(), 0);
    assert_eq!(scheduler.committed().generation().get(), 1);
    assert_eq!(
        scheduler.committed().property(root, WIDTH),
        Some(&PropertyValue::ScalarI32(130))
    );
}

#[test]
fn empty_drop_and_unwind_never_publish_callback_mutations() {
    let mut scheduler = scheduler(QueueCapacity::new(1, 80, 8));
    assert_eq!(
        scheduler
            .begin_callback(SchedulerTick::new(10))
            .expect("empty callback should begin")
            .finish()
            .expect("empty callback should finish"),
        CallbackFinish::NoChanges
    );
    assert_eq!(scheduler.stats().deferred().items(), 0);

    let root = scheduler.committed().root();
    {
        let mut scope = scheduler
            .begin_callback(SchedulerTick::new(11))
            .expect("droppable callback should begin");
        scope
            .transaction()
            .set_property(root, WIDTH, PropertyValue::ScalarI32(130))
            .expect("property write should stage");
    }
    assert_eq!(scheduler.committed().generation().get(), 0);
    assert_eq!(scheduler.stats().deferred().items(), 0);

    let unwind = catch_unwind(AssertUnwindSafe(|| {
        let mut scope = scheduler
            .begin_callback(SchedulerTick::new(12))
            .expect("unwinding callback should begin");
        scope
            .transaction()
            .set_property(root, WIDTH, PropertyValue::ScalarI32(140))
            .expect("property write should stage");
        panic!("synthetic callback panic");
    }));
    assert!(unwind.is_err());
    assert_eq!(scheduler.committed().generation().get(), 0);
    assert_eq!(scheduler.stats().deferred().items(), 0);
}

#[test]
fn empty_and_shutdown_only_callbacks_do_not_require_deferred_capacity() {
    let mut empty = scheduler(QueueCapacity::new(0, 0, 8));
    assert_eq!(
        empty
            .begin_callback(SchedulerTick::new(1))
            .expect("query-only callback should not reserve deferred storage")
            .finish()
            .expect("empty callback should finish"),
        CallbackFinish::NoChanges
    );

    let mut shutdown = scheduler(QueueCapacity::new(0, 0, 8));
    let mut scope = shutdown
        .begin_callback(SchedulerTick::new(1))
        .expect("shutdown-only callback should not reserve deferred storage");
    scope.request_shutdown();
    assert_eq!(
        scope.finish().expect("shutdown should latch on drop"),
        CallbackFinish::ShutdownRequested
    );
    assert_eq!(shutdown.state(), SchedulerState::ShutdownQueued);
    assert_eq!(shutdown.stats().controls().items(), 1);
    assert_eq!(shutdown.stats().deferred().items(), 0);

    let result = shutdown
        .process_input(SchedulerInput::RequestShutdown, SchedulerTick::new(1))
        .expect("explicit duplicate should observe the callback latch");
    let SchedulerInputResult::Control(ControlAdmission::AlreadyAccepted(sequence)) = result else {
        panic!("callback shutdown and explicit shutdown must share one control");
    };
    assert_eq!(sequence.get(), 0);
    assert_eq!(
        shutdown
            .next_action(SchedulerTick::new(1))
            .expect("callback shutdown should remain deliverable"),
        Some(SchedulerAction::StopRenderer(sequence))
    );
    assert_eq!(shutdown.state(), SchedulerState::Stopped);
    assert_eq!(shutdown.stats().controls().items(), 0);
}

#[test]
fn deferred_capacity_and_residence_are_checked_before_publication() {
    let mut scheduler = scheduler(QueueCapacity::new(1, 80, 8));
    let root = scheduler.committed().root();
    let mut scope = scheduler
        .begin_callback(SchedulerTick::new(10))
        .expect("callback should begin");
    scope
        .transaction()
        .set_property(root, WIDTH, PropertyValue::ScalarI32(130))
        .expect("first property write should stage");
    scope
        .transaction()
        .set_property(root, WIDTH, PropertyValue::ScalarI32(140))
        .expect("second property write should stage");
    let error = scope
        .finish()
        .expect_err("two operations exceed the exact 80-byte lane");
    assert_eq!(
        error.kind(),
        SchedulerErrorKind::CapacityExceeded(SchedulerLane::Deferred)
    );
    assert_eq!(scheduler.committed().generation().get(), 0);
    assert_eq!(scheduler.stats().deferred().items(), 0);

    let mut scope = scheduler
        .begin_callback(SchedulerTick::new(11))
        .expect("second callback should begin");
    scope
        .transaction()
        .set_property(root, WIDTH, PropertyValue::ScalarI32(150))
        .expect("bounded property write should stage");
    scope.finish().expect("one operation should fit exactly");
    let pending = scheduler
        .begin_callback(SchedulerTick::new(19))
        .err()
        .expect("an occupied deferred lane blocks another callback");
    assert_eq!(pending.kind(), SchedulerErrorKind::ControlPending);

    let error = scheduler
        .next_action(SchedulerTick::new(20))
        .expect_err("deferred work crossed its inclusive residence deadline");
    assert_eq!(
        error.kind(),
        SchedulerErrorKind::ResidenceExceeded(SchedulerLane::Deferred)
    );
    assert_eq!(scheduler.state(), SchedulerState::Faulted);
    assert_eq!(scheduler.committed().generation().get(), 0);
    assert_eq!(scheduler.stats().deferred().items(), 1);
}

#[test]
fn shutdown_requested_during_unwind_is_latched_once_and_discards_mutation() {
    let mut explicit = scheduler(QueueCapacity::new(1, 80, 8));
    let explicit_root = explicit.committed().root();
    let mut explicit_scope = explicit
        .begin_callback(SchedulerTick::new(9))
        .expect("explicit callback should begin");
    explicit_scope
        .transaction()
        .set_property(explicit_root, WIDTH, PropertyValue::ScalarI32(125))
        .expect("explicit property write should stage");
    explicit_scope.request_shutdown();
    assert_eq!(
        explicit_scope
            .finish()
            .expect("shutdown should win callback finish"),
        CallbackFinish::ShutdownRequested
    );
    assert_eq!(explicit.state(), SchedulerState::ShutdownQueued);
    assert_eq!(explicit.stats().controls().items(), 1);
    assert_eq!(explicit.stats().deferred().items(), 0);
    assert_eq!(explicit.committed().generation().get(), 0);

    let mut scheduler = scheduler(QueueCapacity::new(1, 80, 8));
    let root = scheduler.committed().root();
    let unwind = catch_unwind(AssertUnwindSafe(|| {
        let mut scope = scheduler
            .begin_callback(SchedulerTick::new(10))
            .expect("callback should begin");
        scope.request_shutdown();
        {
            let mut nested = scope.begin_nested();
            nested.request_shutdown();
            nested
                .transaction()
                .set_property(root, WIDTH, PropertyValue::ScalarI32(130))
                .expect("property write should stage");
        }
        panic!("synthetic callback panic after shutdown");
    }));
    assert!(unwind.is_err());
    assert_eq!(scheduler.state(), SchedulerState::ShutdownQueued);
    assert_eq!(scheduler.stats().controls().items(), 1);
    assert_eq!(scheduler.stats().controls().accounted_bytes(), 32);
    assert_eq!(scheduler.stats().deferred().items(), 0);
    assert_eq!(scheduler.committed().generation().get(), 0);

    let error = scheduler
        .begin_callback(SchedulerTick::new(10))
        .err()
        .expect("shutdown latch should block later callbacks");
    assert_eq!(error.kind(), SchedulerErrorKind::ControlPending);
}
