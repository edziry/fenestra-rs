use fenestra_ui_runtime::prototype::{ControlAdmission, SchedulerState};

use super::super::super::driver::NativeDriverActionV1;
use super::super::super::trace::{NativeInputSourceV1, NativeTraceStageV1};
use super::support::{PresenterMode, assert_terminal_empty, driver, physical, tick};

#[test]
fn close_is_idempotent_and_stops_only_through_the_scheduler_control() {
    let mut driver = driver(PresenterMode::Success);
    driver
        .observe_surface(physical(640, 480), 2.0, tick(0))
        .expect("surface should stage");
    driver.drain_scheduler(tick(1)).expect("surface publish");
    driver.redraw_requested(tick(2)).expect("frame present");

    let first = driver
        .close_requested(NativeInputSourceV1::Scripted, tick(3))
        .expect("shutdown should admit");
    let second = driver
        .close_requested(NativeInputSourceV1::Scripted, tick(3))
        .expect("duplicate shutdown should be idempotent");
    let (
        ControlAdmission::Accepted(first_control),
        ControlAdmission::AlreadyAccepted(second_control),
    ) = (first, second)
    else {
        panic!("shutdown admissions should preserve one control identity");
    };
    assert_eq!(first_control, second_control);
    assert_eq!(first_control.get(), 1);
    assert_eq!(driver.scheduler_state(), SchedulerState::ShutdownQueued);
    assert_eq!(
        driver
            .drain_scheduler(tick(4))
            .expect("shutdown control should drain"),
        NativeDriverActionV1::StopRenderer {
            control: first_control
        }
    );
    assert_terminal_empty(&driver);
}

#[test]
fn scheduler_trace_turns_are_dense_and_terminal_state_is_observed() {
    let mut driver = driver(PresenterMode::Success);
    driver
        .observe_surface(physical(640, 480), 2.0, tick(0))
        .expect("surface should stage");
    driver.drain_scheduler(tick(1)).expect("surface publish");
    driver.redraw_requested(tick(2)).expect("frame present");
    driver
        .close_requested(NativeInputSourceV1::Scripted, tick(3))
        .expect("shutdown admit");
    driver.drain_scheduler(tick(4)).expect("shutdown stop");

    let scheduler_events: Vec<_> = driver
        .trace()
        .events()
        .iter()
        .filter(|event| event.stage() == NativeTraceStageV1::Scheduler)
        .collect();
    assert_eq!(scheduler_events.len(), 9);
    assert_eq!(
        scheduler_events
            .iter()
            .map(|event| event.tick().get())
            .collect::<Vec<_>>(),
        [1, 1, 2, 2, 2, 2, 2, 3, 4]
    );
    for (expected, event) in scheduler_events.iter().enumerate() {
        assert_eq!(event.scheduler_turn(), Some(expected as u64));
    }
    assert!(
        scheduler_events
            .iter()
            .take(7)
            .all(|event| event.scheduler_state() == SchedulerState::Running)
    );
    assert_eq!(
        scheduler_events[7].scheduler_state(),
        SchedulerState::ShutdownQueued
    );
    assert_eq!(
        scheduler_events[8].scheduler_state(),
        SchedulerState::Stopped
    );
    assert_eq!(
        scheduler_events
            .iter()
            .map(|event| event.current_generation().get())
            .collect::<Vec<_>>(),
        [0, 1, 1, 1, 1, 1, 1, 1, 1]
    );
    assert!(
        scheduler_events
            .windows(2)
            .all(|events| events[0].tick() <= events[1].tick())
    );
    assert_terminal_empty(&driver);
}

#[test]
fn shutdown_preempts_pending_surface_and_disarms_outstanding_redraw() {
    let mut driver = driver(PresenterMode::Success);
    driver
        .observe_surface(physical(640, 480), 2.0, tick(0))
        .expect("initial surface should stage");
    driver
        .drain_scheduler(tick(1))
        .expect("initial surface should publish");
    assert!(driver.redraw_armed());
    driver
        .observe_surface(physical(720, 520), 2.0, tick(2))
        .expect("resize should remain pending");
    assert!(driver.pending_surface().is_some());

    let admission = driver
        .close_requested(NativeInputSourceV1::Scripted, tick(3))
        .expect("shutdown should preempt pending visual work");
    let ControlAdmission::Accepted(control) = admission else {
        panic!("first shutdown must allocate one control");
    };
    assert!(!driver.redraw_armed());
    assert!(driver.pending_surface().is_none());
    assert_eq!(
        driver
            .drain_scheduler(tick(4))
            .expect("stop must not be blocked by a pending resize"),
        NativeDriverActionV1::StopRenderer { control }
    );
    assert_terminal_empty(&driver);
    let terminal = driver.trace().events().last().expect("stop should record");
    assert_eq!(terminal.scheduler_state(), SchedulerState::Stopped);
    assert!(!terminal.redraw_armed());
    assert_eq!(terminal.pending().surface(), 0);
    assert_eq!(terminal.visual().items(), 0);
}
