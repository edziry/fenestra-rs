use fenestra_ui_runtime::prototype::SchedulerState;

use super::super::super::driver::NativeDriverActionV1;
use super::super::super::trace::{
    NativeFailureCauseV1, NativeObservationV1, NativeOutcomeV1, NativeTraceStageV1,
};
use super::support::{PresenterMode, driver, physical, tick};

#[test]
fn preaccept_failures_reject_the_offer_without_accepting_a_submission() {
    for (mode, cause, notify_count) in [
        (
            PresenterMode::FailPreflight,
            NativeFailureCauseV1::Presenter,
            0,
        ),
        (
            PresenterMode::FailNotify,
            NativeFailureCauseV1::PrePresent,
            1,
        ),
    ] {
        let mut driver = driver(mode);
        driver
            .observe_surface(physical(640, 480), 2.0, tick(0))
            .expect("surface should stage");
        driver.drain_scheduler(tick(1)).expect("surface publish");

        assert_eq!(
            driver
                .redraw_requested(tick(2))
                .expect_err("preaccept failure should be typed"),
            cause
        );
        assert_eq!(driver.presenter().preflight_count(), 1);
        assert_eq!(driver.presenter().notify_count(), notify_count);
        assert_eq!(driver.presenter().present_count(), 0);
        let stats = driver.scheduler_stats();
        assert_eq!(stats.in_flight().items(), 0);
        assert_eq!(stats.visual().items(), 1);
        assert_eq!(driver.presenter_pending_count(), 0);
        assert_eq!(driver.scheduler_state(), SchedulerState::Running);
        assert!(driver.trace().events().iter().any(|event| {
            event.stage() == NativeTraceStageV1::Renderer
                && event.observation() == NativeObservationV1::Frame
                && event.outcome() == NativeOutcomeV1::Rejected
                && event.frame().is_some_and(|frame| frame == 0)
                && event.submission().is_none()
                && event.control().is_none()
        }));
    }
}

#[test]
fn postaccept_failure_reports_renderer_loss_without_rejecting_the_frame() {
    let mut driver = driver(PresenterMode::FailPresent);
    driver
        .observe_surface(physical(640, 480), 2.0, tick(0))
        .expect("surface should stage");
    driver.drain_scheduler(tick(1)).expect("surface publish");

    assert_eq!(
        driver
            .redraw_requested(tick(2))
            .expect_err("present failure should be typed"),
        NativeFailureCauseV1::Presenter
    );
    assert_eq!(driver.presenter().preflight_count(), 1);
    assert_eq!(driver.presenter().notify_count(), 1);
    assert_eq!(driver.presenter().present_count(), 1);
    assert_eq!(driver.scheduler_stats().in_flight().items(), 1);
    assert_eq!(driver.scheduler_stats().controls().items(), 1);
    assert!(driver.trace().events().iter().any(|event| {
        event.stage() == NativeTraceStageV1::Scheduler
            && event.observation() == NativeObservationV1::Frame
            && event.outcome() == NativeOutcomeV1::Accepted
            && event.frame().is_some_and(|frame| frame == 0)
            && event
                .submission()
                .is_some_and(|submission| submission.epoch() == 0 && submission.token() == 0)
    }));
    assert!(driver.trace().events().iter().any(|event| {
        event.stage() == NativeTraceStageV1::Scheduler
            && event.observation() == NativeObservationV1::Present
            && event.outcome() == NativeOutcomeV1::Accepted
            && event.control().is_some_and(|control| control == 0)
    }));

    assert_eq!(
        driver
            .drain_scheduler(tick(3))
            .expect("renderer loss control should drain"),
        NativeDriverActionV1::Idle
    );
    assert_eq!(driver.scheduler_state(), SchedulerState::Faulted);
    assert_eq!(driver.scheduler_stats().visual().items(), 0);
    assert_eq!(driver.scheduler_stats().controls().items(), 0);
    assert_eq!(driver.scheduler_stats().in_flight().items(), 1);
    assert_eq!(driver.presenter_pending_count(), 0);
    assert!(!driver.redraw_armed());
}
