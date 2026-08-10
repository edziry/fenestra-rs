use fenestra_ui_runtime::prototype::HeadlessSurface;

use super::super::shell::script::NativeRunDirectiveV1;
use super::app::{
    NativeDirectiveSlotV1, WatchdogExpectationV1, active_resize_refuses_surface_v1,
    requested_surface_matches_v1, surface_preemption_watchdog_v1, surface_preempts_directive_v1,
    surface_preempts_redraw_v1,
};

#[test]
fn scripted_resize_waits_in_one_slot_for_the_next_native_batch_boundary() {
    let resize = NativeRunDirectiveV1::RequestLogicalResize {
        width: 360,
        height: 260,
    };
    let mut slot = NativeDirectiveSlotV1::default();

    assert_eq!(slot.pending_count(), 0);
    slot.defer_until_barrier(resize)
        .expect("empty slot should accept resize");
    assert_eq!(slot.pending_count(), 1);
    assert_eq!(slot.take_ready(), None);
    assert!(slot.defer_until_barrier(resize).is_err());
    slot.release_barrier()
        .expect("matching native barrier should release directive");
    assert!(slot.release_barrier().is_err());
    assert_eq!(slot.take_ready(), Some(resize));
    assert_eq!(slot.pending_count(), 0);

    slot.defer_until_barrier(resize)
        .expect("slot should be reusable after dispatch");
    slot.clear();
    assert_eq!(slot.pending_count(), 0);
    assert!(slot.release_barrier().is_err());
}

#[test]
fn effective_surface_preempts_frame_and_settlement_but_not_requested_resize() {
    assert!(surface_preempts_redraw_v1(
        true,
        Some(WatchdogExpectationV1::Redraw)
    ));
    assert!(surface_preempts_redraw_v1(
        true,
        Some(WatchdogExpectationV1::PresentSettled)
    ));
    assert!(!surface_preempts_redraw_v1(
        true,
        Some(WatchdogExpectationV1::Resize)
    ));
    assert!(!surface_preempts_redraw_v1(false, None));
    assert_eq!(
        surface_preemption_watchdog_v1(true, Some(WatchdogExpectationV1::Redraw)),
        Some(WatchdogExpectationV1::Redraw)
    );
    assert_eq!(
        surface_preemption_watchdog_v1(true, Some(WatchdogExpectationV1::PresentSettled)),
        Some(WatchdogExpectationV1::PresentSettled)
    );
    assert_eq!(
        surface_preemption_watchdog_v1(true, Some(WatchdogExpectationV1::Resize)),
        None
    );
    assert!(surface_preempts_directive_v1(true, false));
    assert!(!surface_preempts_directive_v1(false, false));
    assert!(!surface_preempts_directive_v1(true, true));
}

#[test]
fn requested_resize_matches_only_its_exact_logical_surface() {
    let requested = NativeRunDirectiveV1::RequestLogicalResize {
        width: 360,
        height: 260,
    };
    assert!(requested_surface_matches_v1(
        requested,
        HeadlessSurface::new(360, 260)
    ));
    assert!(!requested_surface_matches_v1(
        requested,
        HeadlessSurface::new(350, 250)
    ));
    assert!(!requested_surface_matches_v1(
        NativeRunDirectiveV1::AwaitRedraw,
        HeadlessSurface::new(360, 260)
    ));

    assert!(!active_resize_refuses_surface_v1(
        Some(WatchdogExpectationV1::Resize),
        requested,
        Some(HeadlessSurface::new(360, 260)),
    ));
    assert!(active_resize_refuses_surface_v1(
        Some(WatchdogExpectationV1::Resize),
        requested,
        Some(HeadlessSurface::new(320, 240)),
    ));
    assert!(active_resize_refuses_surface_v1(
        Some(WatchdogExpectationV1::Resize),
        requested,
        Some(HeadlessSurface::new(350, 250)),
    ));
    assert!(!active_resize_refuses_surface_v1(
        Some(WatchdogExpectationV1::Redraw),
        requested,
        Some(HeadlessSurface::new(350, 250)),
    ));
}
