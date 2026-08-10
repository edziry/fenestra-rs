use fenestra_ui_testkit::prototype::HeadlessPointerTargetV1;

use super::super::super::trace::{
    NativeFailureCauseV1, NativeInputSourceV1, NativeObservationV1, NativeOutcomeV1,
    NativeTraceStageV1,
};
use super::super::super::{NativePhysicalPointV1, NativeSurfaceChangeV1};
use super::support::{PresenterMode, driver, physical, scripted_point, tick};

#[test]
fn cursor_moves_replace_one_slot_and_press_consumes_the_latest_point() {
    let mut driver = ready_driver();
    let scheduler_stats = driver.scheduler_stats();

    for value in 0_u32..31 {
        driver
            .cursor_moved(
                NativePhysicalPointV1::new(500.0 + f64::from(value), 500.0),
                NativeInputSourceV1::Scripted,
                tick(2 + u64::from(value)),
            )
            .expect("cursor movement should replace the bounded slot");
        assert_eq!(driver.pending_pointer_count(), 1);
        assert_eq!(driver.scheduler_stats(), scheduler_stats);
    }
    driver
        .cursor_moved(scripted_point(), NativeInputSourceV1::Scripted, tick(33))
        .expect("the latest cursor point should replace the prior point");
    assert_eq!(driver.pending_pointer_count(), 1);

    assert_eq!(
        driver
            .pointer_pressed(NativeInputSourceV1::Scripted, tick(34))
            .expect("the press should consume the latest cursor point"),
        HeadlessPointerTargetV1::StaticControl
    );
    assert_eq!(driver.pending_pointer_count(), 0);
    assert_eq!(driver.scheduler_stats(), scheduler_stats);

    let pointer_events: Vec<_> = driver
        .trace()
        .events()
        .iter()
        .filter(|event| {
            event.stage() == NativeTraceStageV1::Platform
                && event.observation() == NativeObservationV1::Pointer
        })
        .collect();
    assert_eq!(pointer_events.len(), 33);
    assert!(pointer_events[..32].iter().all(|event| {
        event.outcome() == NativeOutcomeV1::Coalesced
            && event.target().is_none()
            && event.captured_generation().is_none()
            && event.surface().is_none()
            && event.pending().pointer() == 1
    }));
    let press = pointer_events[32];
    assert_eq!(press.outcome(), NativeOutcomeV1::Observed);
    assert_eq!(press.target(), Some(HeadlessPointerTargetV1::StaticControl));
    assert_eq!(press.pending().pointer(), 0);
}

#[test]
fn pointer_press_without_cursor_position_is_typed_and_atomic() {
    let mut driver = ready_driver();
    let generation = driver.runtime_generation();
    let accepted = driver.accepted_surface();
    let pending = driver.pending_surface();
    let scheduler_stats = driver.scheduler_stats();
    let redraw_armed = driver.redraw_armed();
    let trace = driver.trace().events().to_vec();

    assert_eq!(driver.pending_pointer_count(), 0);
    assert_eq!(
        driver
            .pointer_pressed(NativeInputSourceV1::Native, tick(2))
            .expect_err("a press cannot invent a cursor position"),
        NativeFailureCauseV1::Invariant
    );
    assert_eq!(driver.runtime_generation(), generation);
    assert_eq!(driver.accepted_surface(), accepted);
    assert_eq!(driver.pending_surface(), pending);
    assert_eq!(driver.scheduler_stats(), scheduler_stats);
    assert_eq!(driver.redraw_armed(), redraw_armed);
    assert_eq!(driver.pending_pointer_count(), 0);
    assert_eq!(driver.trace().events(), trace.as_slice());
}

#[test]
fn pointer_press_uses_the_accepted_tuple_while_a_resize_is_pending() {
    let mut driver = driver(PresenterMode::Success);
    driver
        .observe_surface(physical(50, 30), 2.0, tick(0))
        .expect("initial surface should stage");
    driver
        .drain_scheduler(tick(1))
        .expect("initial surface should publish");
    let accepted = driver
        .accepted_surface()
        .expect("initial tuple should be input-visible");
    assert_eq!(
        driver
            .observe_surface(physical(720, 520), 2.0, tick(2))
            .expect("logical resize should stage"),
        NativeSurfaceChangeV1::LogicalResize
    );

    driver
        .cursor_moved(
            NativePhysicalPointV1::new(50.0, 10.0),
            NativeInputSourceV1::Native,
            tick(2),
        )
        .expect("cursor should stage while resize is pending");
    assert_eq!(
        driver
            .pointer_pressed(NativeInputSourceV1::Native, tick(2))
            .expect("press should use the accepted tuple"),
        HeadlessPointerTargetV1::None
    );

    let press = driver.trace().events().last().expect("press should record");
    assert_eq!(
        press.captured_generation().map(|value| value.get()),
        Some(1)
    );
    assert_eq!(press.surface(), Some(accepted));
    assert_eq!(press.pending().surface(), 1);
    assert_eq!(press.pending().pointer(), 0);
}

#[test]
fn native_and_scripted_pointer_inputs_share_the_same_reducer() {
    for source in NativeInputSourceV1::ALL {
        let mut driver = ready_driver();
        driver
            .cursor_moved(scripted_point(), source, tick(2))
            .expect("both sources should update the same cursor slot");
        assert_eq!(
            driver
                .pointer_pressed(source, tick(3))
                .expect("both sources should use the same press reducer"),
            HeadlessPointerTargetV1::StaticControl
        );

        let pointer_events: Vec<_> = driver
            .trace()
            .events()
            .iter()
            .filter(|event| {
                event.stage() == NativeTraceStageV1::Platform
                    && event.observation() == NativeObservationV1::Pointer
            })
            .collect();
        assert_eq!(pointer_events.len(), 2);
        assert_eq!(pointer_events[0].outcome(), NativeOutcomeV1::Coalesced);
        assert_eq!(pointer_events[1].outcome(), NativeOutcomeV1::Observed);
        assert!(
            pointer_events
                .iter()
                .all(|event| event.input_source() == Some(source))
        );
    }
}

#[test]
fn first_close_clears_the_pending_cursor_slot() {
    let mut driver = ready_driver();
    driver
        .cursor_moved(scripted_point(), NativeInputSourceV1::Native, tick(2))
        .expect("cursor should occupy one slot");
    assert_eq!(driver.pending_pointer_count(), 1);

    driver
        .close_requested(NativeInputSourceV1::Scripted, tick(3))
        .expect("first close should be accepted");
    assert_eq!(driver.pending_pointer_count(), 0);
    let close = driver
        .trace()
        .events()
        .iter()
        .find(|event| {
            event.stage() == NativeTraceStageV1::Platform
                && event.observation() == NativeObservationV1::Close
                && event.outcome() == NativeOutcomeV1::Observed
        })
        .expect("first close should record");
    assert_eq!(close.pending().pointer(), 0);
}

fn ready_driver() -> super::super::super::driver::NativeDriverV1<super::support::TestPresenter> {
    let mut driver = driver(PresenterMode::Success);
    driver
        .observe_surface(physical(640, 480), 2.0, tick(0))
        .expect("surface should stage");
    driver
        .drain_scheduler(tick(1))
        .expect("surface should publish");
    driver
}
