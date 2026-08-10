use super::super::super::trace::{NativeInputSourceV1, NativeObservationV1, NativeTraceStageV1};
use super::support::{PresenterMode, PresenterPhase, driver, physical, scripted_point, tick};

#[test]
fn presenter_keeps_staging_acceptance_and_present_in_one_scope() {
    let mut driver = driver(PresenterMode::Success);
    driver
        .observe_surface(physical(640, 480), 2.0, tick(0))
        .expect("surface should stage");
    driver.drain_scheduler(tick(1)).expect("surface publish");
    driver.redraw_requested(tick(2)).expect("frame present");

    assert_eq!(
        driver.presenter().phases(),
        [
            PresenterPhase::Preflight,
            PresenterPhase::PrePresent,
            PresenterPhase::Accept,
            PresenterPhase::Present,
        ]
    );
}

#[test]
fn scripted_pointer_and_close_are_labeled_without_changing_the_reducer() {
    let mut driver = driver(PresenterMode::Success);
    driver
        .observe_surface(physical(640, 480), 2.0, tick(0))
        .expect("surface should stage");
    driver.drain_scheduler(tick(1)).expect("surface publish");
    driver.redraw_requested(tick(2)).expect("frame present");

    driver
        .pointer_pressed(scripted_point(), NativeInputSourceV1::Scripted, tick(3))
        .expect("scripted pointer should use the normal reducer");
    driver
        .close_requested(NativeInputSourceV1::Scripted, tick(4))
        .expect("scripted close should use the normal reducer");

    let inputs: Vec<_> = driver
        .trace()
        .events()
        .iter()
        .filter(|event| {
            event.stage() == NativeTraceStageV1::Platform
                && matches!(
                    event.observation(),
                    NativeObservationV1::Pointer | NativeObservationV1::Close
                )
        })
        .collect();
    assert_eq!(inputs.len(), 2);
    assert_eq!(
        inputs[0].input_source(),
        Some(NativeInputSourceV1::Scripted)
    );
    assert_eq!(
        inputs[1].input_source(),
        Some(NativeInputSourceV1::Scripted)
    );
}
