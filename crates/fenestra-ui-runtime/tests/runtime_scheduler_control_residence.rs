mod support;

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{
    ControlAdmission, QueueCapacity, SchedulerAction, SchedulerCapacity, SchedulerErrorKind,
    SchedulerInput, SchedulerInputResult, SchedulerLane, SchedulerState, SchedulerTick, UiRuntime,
    UiScheduler,
};

use support::{WIDTH, capacity, construction};

fn scheduler() -> UiScheduler {
    let runtime = UiRuntime::new(construction(), capacity()).expect("runtime should initialize");
    UiScheduler::new(
        runtime,
        SchedulerCapacity::new(
            QueueCapacity::new(1, 80, 8),
            QueueCapacity::new(2, 64, 8),
            QueueCapacity::new(1, 40, 8),
            QueueCapacity::new(2, 80, 8),
        ),
    )
    .expect("scheduler capacity should be valid")
}

fn request_shutdown(scheduler: &mut UiScheduler, tick: u64) -> ControlAdmission {
    let result = scheduler
        .process_input(SchedulerInput::RequestShutdown, SchedulerTick::new(tick))
        .expect("shutdown should use its reserved control slot");
    let SchedulerInputResult::Control(admission) = result else {
        panic!("shutdown should return a control admission");
    };
    admission
}

#[test]
fn control_residence_is_inclusive_and_terminal_delivery_preserves_pressure() {
    let mut inclusive = scheduler();
    let ControlAdmission::Accepted(inclusive_shutdown) = request_shutdown(&mut inclusive, 1) else {
        panic!("first shutdown should be accepted");
    };
    assert_eq!(
        inclusive
            .next_action(SchedulerTick::new(9))
            .expect("the exact control deadline remains deliverable"),
        Some(SchedulerAction::StopRenderer(inclusive_shutdown))
    );
    assert_eq!(inclusive.state(), SchedulerState::Stopped);

    let mut crossed = scheduler();
    let ControlAdmission::Accepted(crossed_shutdown) = request_shutdown(&mut crossed, 1) else {
        panic!("first shutdown should be accepted");
    };
    let error = crossed
        .next_action(SchedulerTick::new(10))
        .expect_err("the first observed crossing should report control pressure");
    assert_eq!(
        error.kind(),
        SchedulerErrorKind::ResidenceExceeded(SchedulerLane::Controls)
    );
    assert_eq!(crossed.state(), SchedulerState::Faulted);
    assert_eq!(crossed.stats().controls().items(), 1);
    assert_eq!(crossed.stats().controls().accounted_bytes(), 32);

    assert_eq!(
        crossed
            .next_action(SchedulerTick::new(10))
            .expect("terminal cleanup must still deliver accepted shutdown"),
        Some(SchedulerAction::StopRenderer(crossed_shutdown))
    );
    assert_eq!(crossed.state(), SchedulerState::Faulted);
    assert_eq!(crossed.stats().controls().items(), 0);

    let root = crossed.committed().root();
    let mut transaction = crossed.begin_transaction();
    transaction
        .set_property(root, WIDTH, PropertyValue::ScalarI32(130))
        .expect("property write should stage");
    let error = crossed
        .commit(transaction, SchedulerTick::new(10))
        .expect_err("terminal delivery must not clear residence pressure");
    assert_eq!(
        error.kind(),
        SchedulerErrorKind::ResidenceExceeded(SchedulerLane::Controls)
    );
}
