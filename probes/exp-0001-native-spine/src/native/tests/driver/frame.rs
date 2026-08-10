use super::super::super::driver::{NativeDriverActionV1, NativeRedrawResultV1};
use super::support::{PresenterMode, driver, physical, tick};

#[test]
fn only_an_armed_redraw_can_offer_accept_present_and_complete() {
    let mut driver = driver(PresenterMode::Success);
    assert_eq!(
        driver
            .redraw_requested(tick(0))
            .expect("spontaneous redraw should be harmless"),
        NativeRedrawResultV1::Ignored
    );
    assert_eq!(driver.presenter().present_count(), 0);

    driver
        .observe_surface(physical(640, 480), 2.0, tick(0))
        .expect("initial surface should stage");
    assert!(matches!(
        driver
            .drain_scheduler(tick(1))
            .expect("initial publication should request a frame"),
        NativeDriverActionV1::RequestFrame { .. }
    ));
    let surface = driver.accepted_surface().expect("surface should publish");

    let NativeRedrawResultV1::Presented {
        frame,
        submission,
        completion_control,
    } = driver
        .redraw_requested(tick(2))
        .expect("armed redraw should present")
    else {
        panic!("armed redraw should present one frame");
    };
    assert_eq!(frame.get(), 0);
    assert_eq!(submission.epoch().get(), 0);
    assert_eq!(submission.token(), 0);
    assert_eq!(completion_control.get(), 0);
    assert_eq!(driver.presenter().preflight_count(), 1);
    assert_eq!(driver.presenter().notify_count(), 1);
    assert_eq!(driver.presenter().present_count(), 1);
    assert_eq!(
        driver.presenter().last_generation(),
        Some(driver.runtime_generation())
    );
    assert_eq!(driver.presenter().last_surface(), Some(surface));
    assert!(driver.presenter().last_digest().is_some());
    assert_eq!(driver.presenter_pending_count(), 0);
    assert!(!driver.redraw_armed());
    let stats = driver.scheduler_stats();
    assert_eq!(stats.visual().items(), 0);
    assert_eq!(stats.in_flight().items(), 0);
    assert_eq!(stats.controls().items(), 0);

    let stats = driver.scheduler_stats();
    assert_eq!(
        driver
            .redraw_requested(tick(2))
            .expect("duplicate redraw should be ignored"),
        NativeRedrawResultV1::Ignored
    );
    assert_eq!(driver.scheduler_stats(), stats);
    assert_eq!(driver.presenter().present_count(), 1);
}

#[test]
fn second_surface_publication_uses_the_next_frame_and_submission() {
    let mut driver = driver(PresenterMode::Success);
    driver
        .observe_surface(physical(640, 480), 2.0, tick(0))
        .expect("initial surface should stage");
    driver.drain_scheduler(tick(1)).expect("initial publish");
    driver.redraw_requested(tick(2)).expect("initial present");
    driver
        .redraw_requested(tick(2))
        .expect("duplicate redraw should stay ignored");

    driver
        .observe_surface(physical(720, 520), 2.0, tick(3))
        .expect("resize should stage");
    let NativeDriverActionV1::RequestFrame {
        generation,
        surface_generation,
    } = driver.drain_scheduler(tick(4)).expect("resize publish")
    else {
        panic!("resize should request a frame");
    };
    assert_eq!(generation.get(), 2);
    assert_eq!(surface_generation.get(), 1);
    let NativeRedrawResultV1::Presented {
        frame,
        submission,
        completion_control,
    } = driver.redraw_requested(tick(5)).expect("resize present")
    else {
        panic!("resize should present a frame");
    };
    assert_eq!(frame.get(), 1);
    assert_eq!(submission.epoch().get(), 0);
    assert_eq!(submission.token(), 1);
    assert_eq!(completion_control.get(), 1);
}
