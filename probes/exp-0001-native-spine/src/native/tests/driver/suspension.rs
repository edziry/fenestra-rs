use fenestra_ui_runtime::prototype::HeadlessSurface;

use super::super::super::driver::{NativeDriverActionV1, NativeRedrawResultV1};
use super::support::{PresenterMode, driver, physical, tick};

#[test]
fn zero_surface_publishes_without_arming_or_offering_a_frame() {
    let mut driver = driver(PresenterMode::Success);
    driver
        .observe_surface(physical(640, 480), 2.0, tick(0))
        .expect("initial surface should stage");
    driver.drain_scheduler(tick(1)).expect("initial publish");
    driver.redraw_requested(tick(2)).expect("initial present");

    driver
        .observe_surface(physical(0, 480), 2.0, tick(3))
        .expect("zero width should stage suspension");
    let NativeDriverActionV1::Suspended {
        generation,
        surface_generation,
    } = driver
        .drain_scheduler(tick(4))
        .expect("suspension should publish and cancel visual work")
    else {
        panic!("zero surface should suspend without requesting a redraw");
    };
    assert_eq!(generation.get(), 2);
    assert_eq!(surface_generation.get(), 1);
    assert_eq!(
        driver
            .accepted_surface()
            .expect("suspended tuple should become input-visible")
            .logical_surface(),
        HeadlessSurface::new(0, 0)
    );
    assert!(!driver.redraw_armed());
    assert_eq!(driver.scheduler_stats().visual().items(), 1);
    let suspended_stats = driver.scheduler_stats();
    assert_eq!(
        driver
            .redraw_requested(tick(4))
            .expect("a suspended redraw should be ignored"),
        NativeRedrawResultV1::Ignored
    );
    assert_eq!(driver.scheduler_stats(), suspended_stats);
    assert_eq!(driver.presenter().present_count(), 1);

    driver
        .observe_surface(physical(640, 480), 2.0, tick(5))
        .expect("nonzero surface should restore");
    assert!(matches!(
        driver
            .drain_scheduler(tick(6))
            .expect("restore should reuse the outstanding request"),
        NativeDriverActionV1::RequestFrame { .. }
    ));
    let NativeRedrawResultV1::Presented {
        frame, submission, ..
    } = driver
        .redraw_requested(tick(7))
        .expect("restored surface should present")
    else {
        panic!("restore should consume the next frame identity");
    };
    assert_eq!(frame.get(), 1);
    assert_eq!(submission.token(), 1);
}

#[test]
fn restoring_a_suspended_surface_arms_exactly_one_fresh_frame() {
    let mut driver = driver(PresenterMode::Success);
    driver
        .observe_surface(physical(0, 480), 2.0, tick(0))
        .expect("initial zero surface should stage");
    assert!(matches!(
        driver
            .drain_scheduler(tick(1))
            .expect("initial zero surface should settle"),
        NativeDriverActionV1::Suspended { .. }
    ));
    assert!(!driver.redraw_armed());
    assert_eq!(driver.scheduler_stats().visual().items(), 0);

    driver
        .observe_surface(physical(640, 480), 2.0, tick(2))
        .expect("nonzero surface should restore");
    let NativeDriverActionV1::RequestFrame {
        generation,
        surface_generation,
    } = driver
        .drain_scheduler(tick(3))
        .expect("restore should publish")
    else {
        panic!("restore should request one fresh frame");
    };
    assert_eq!(generation.get(), 1);
    assert_eq!(surface_generation.get(), 1);
    assert!(driver.redraw_armed());
    assert_eq!(driver.scheduler_stats().visual().items(), 1);
}
