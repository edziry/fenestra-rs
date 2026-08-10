use fenestra_ui_runtime::prototype::{HeadlessSurface, SchedulerState};
use fenestra_ui_testkit::prototype::HeadlessPointerTargetV1;

use super::super::super::driver::NativeDriverActionV1;
use super::super::super::trace::{NativeFailureCauseV1, NativeInputSourceV1};
use super::super::super::{NativePhysicalPointV1, NativeSurfaceChangeV1};
use super::support::{PresenterMode, driver, physical, tick};

#[test]
fn initial_surface_publishes_before_it_becomes_input_visible() {
    let mut driver = driver(PresenterMode::Success);
    assert_eq!(driver.runtime_generation().get(), 0);
    assert_eq!(driver.accepted_surface(), None);
    assert_eq!(driver.pending_surface(), None);

    assert_eq!(
        driver
            .observe_surface(physical(640, 480), 2.0, tick(0))
            .expect("initial observation should defer"),
        NativeSurfaceChangeV1::Initialized
    );
    let pending = driver.pending_surface().expect("one tuple should wait");
    assert_eq!(pending.generation().get(), 0);
    assert_eq!(pending.logical_surface(), HeadlessSurface::new(320, 240));
    assert_eq!(driver.accepted_surface(), None);
    assert_eq!(driver.runtime_generation().get(), 0);
    assert_eq!(driver.scheduler_stats().deferred().items(), 0);
    let observation = driver
        .trace()
        .events()
        .last()
        .expect("surface observation should record");
    assert_eq!(observation.pending().surface(), 1);
    assert_eq!(observation.pending().presenter(), 0);

    let NativeDriverActionV1::RequestFrame {
        generation,
        surface_generation,
    } = driver
        .drain_scheduler(tick(1))
        .expect("deferred publication should drain")
    else {
        panic!("initial publication should request a frame");
    };
    assert_eq!(generation.get(), 1);
    assert_eq!(surface_generation.get(), 0);
    assert_eq!(driver.runtime_generation().get(), 1);
    assert_eq!(driver.accepted_surface(), Some(pending));
    assert_eq!(driver.pending_surface(), None);
    assert!(driver.redraw_armed());
    assert_eq!(driver.scheduler_state(), SchedulerState::Running);
}

#[test]
fn pending_resize_keeps_pointer_on_the_previous_accepted_tuple() {
    let mut driver = driver(PresenterMode::Success);
    driver
        .observe_surface(physical(50, 30), 2.0, tick(0))
        .expect("initial surface should stage");
    driver
        .drain_scheduler(tick(1))
        .expect("initial surface should publish");
    driver
        .redraw_requested(tick(2))
        .expect("initial frame should retire");
    let accepted = driver
        .accepted_surface()
        .expect("initial tuple should publish");

    assert_eq!(
        driver
            .observe_surface(physical(720, 520), 2.0, tick(3))
            .expect("logical resize should stage"),
        NativeSurfaceChangeV1::LogicalResize
    );
    assert_eq!(driver.accepted_surface(), Some(accepted));
    assert_eq!(driver.runtime_generation().get(), 1);
    driver
        .cursor_moved(
            NativePhysicalPointV1::new(50.0, 10.0),
            NativeInputSourceV1::Scripted,
            tick(3),
        )
        .expect("pointer position should stage");
    assert_eq!(
        driver
            .pointer_pressed(NativeInputSourceV1::Scripted, tick(3))
            .expect("pointer should use the committed tuple"),
        HeadlessPointerTargetV1::None
    );
    let pointer = driver
        .trace()
        .events()
        .last()
        .expect("pointer should record");
    assert_eq!(
        pointer.captured_generation().map(|value| value.get()),
        Some(1)
    );
    assert_eq!(pointer.surface(), Some(accepted));

    let NativeDriverActionV1::RequestFrame {
        generation,
        surface_generation,
    } = driver
        .drain_scheduler(tick(4))
        .expect("resize should publish")
    else {
        panic!("resize publication should request a frame");
    };
    assert_eq!(generation.get(), 2);
    assert_eq!(surface_generation.get(), 1);
    assert_eq!(driver.runtime_generation().get(), 2);
    assert_eq!(
        driver
            .accepted_surface()
            .expect("resize tuple should publish")
            .logical_surface(),
        HeadlessSurface::new(360, 260)
    );
    driver
        .cursor_moved(
            NativePhysicalPointV1::new(50.0, 10.0),
            NativeInputSourceV1::Scripted,
            tick(4),
        )
        .expect("pointer position should stage again");
    assert_eq!(
        driver
            .pointer_pressed(NativeInputSourceV1::Scripted, tick(4))
            .expect("published resize should enter input"),
        HeadlessPointerTargetV1::StaticControl
    );
}

#[test]
fn native_only_and_post_initial_scale_changes_do_not_bypass_the_scheduler() {
    let mut driver = driver(PresenterMode::Success);
    driver
        .observe_surface(physical(640, 480), 2.0, tick(0))
        .expect("initial surface should stage");
    driver
        .drain_scheduler(tick(1))
        .expect("initial surface should publish");
    driver
        .redraw_requested(tick(2))
        .expect("initial frame should retire");
    let accepted = driver.accepted_surface();
    let stats = driver.scheduler_stats();

    assert_eq!(
        driver
            .observe_surface(physical(639, 480), 2.0, tick(3))
            .expect_err("native-only repaint is not admitted by the scheduler"),
        NativeFailureCauseV1::SurfaceRepaintUnavailable
    );
    assert_eq!(driver.accepted_surface(), accepted);
    assert!(driver.pending_surface().is_some());
    assert_eq!(driver.scheduler_stats(), stats);
    assert!(!driver.redraw_armed());

    let pending = driver.pending_surface();
    assert_eq!(
        driver
            .observe_surface(physical(720, 520), 2.01, tick(4))
            .expect_err("post-initial scale change is terminal"),
        NativeFailureCauseV1::EnvironmentScaleChanged
    );
    assert_eq!(driver.accepted_surface(), accepted);
    assert_eq!(driver.pending_surface(), pending);
    assert_eq!(driver.scheduler_stats(), stats);
    let failure = driver
        .trace()
        .events()
        .last()
        .expect("scale rejection should retain its observation");
    assert_eq!(failure.surface(), None);
    let observed = failure
        .surface_observation()
        .expect("scale rejection should not masquerade as an accepted tuple");
    assert_eq!(observed.physical(), physical(720, 520));
    assert_eq!(observed.scale().micros(), 2_010_000);
    assert_eq!(observed.logical_surface(), HeadlessSurface::new(359, 259));
}

#[test]
fn environment_surface_change_is_typed_before_redraw_and_preserves_accepted_state() {
    let mut driver = driver(PresenterMode::Success);
    driver
        .observe_surface(physical(640, 480), 2.0, tick(0))
        .expect("initial surface should stage");
    driver
        .drain_scheduler(tick(1))
        .expect("initial surface should publish");
    let accepted = driver.accepted_surface();
    let generation = driver.runtime_generation();
    let stats = driver.scheduler_stats();

    driver
        .observe_surface(physical(720, 520), 2.0, tick(2))
        .expect("environment change should remain pending");
    assert_eq!(
        driver
            .reject_environment_surface_before_redraw(tick(3))
            .expect_err("unstable surface must preempt the frame"),
        NativeFailureCauseV1::EnvironmentSurfaceChanged
    );

    assert_eq!(driver.accepted_surface(), accepted);
    assert_eq!(driver.pending_surface(), None);
    assert_eq!(driver.runtime_generation(), generation);
    assert_eq!(driver.scheduler_stats(), stats);
    let events = driver.trace().events();
    let [ignored, failed] = &events[events.len() - 2..] else {
        panic!("redraw rejection should append two events");
    };
    assert_eq!(
        ignored.stage(),
        super::super::super::trace::NativeTraceStageV1::Platform
    );
    assert_eq!(
        ignored.observation(),
        super::super::super::trace::NativeObservationV1::Redraw
    );
    assert_eq!(
        ignored.outcome(),
        super::super::super::trace::NativeOutcomeV1::Ignored
    );
    assert_eq!(
        failed.outcome(),
        super::super::super::trace::NativeOutcomeV1::Failed(
            NativeFailureCauseV1::EnvironmentSurfaceChanged,
        )
    );
    assert!(failed.surface().is_some());
}

#[test]
fn environment_surface_change_between_script_directives_has_no_fake_redraw() {
    let mut driver = driver(PresenterMode::Success);
    driver
        .observe_surface(physical(640, 480), 2.0, tick(0))
        .expect("initial surface should stage");
    driver
        .drain_scheduler(tick(1))
        .expect("initial surface should publish");
    driver
        .observe_surface(physical(720, 520), 2.0, tick(2))
        .expect("environment change should remain pending");
    let before = driver.trace().events().len();

    assert_eq!(
        driver
            .reject_environment_surface_between_directives(tick(3))
            .expect_err("settlement barrier must reject an effective change"),
        NativeFailureCauseV1::EnvironmentSurfaceChanged
    );
    assert_eq!(driver.trace().events().len(), before + 1);
    let failed = driver
        .trace()
        .events()
        .last()
        .expect("failure should record");
    assert_eq!(
        failed.observation(),
        super::super::super::trace::NativeObservationV1::Surface
    );
    assert_eq!(
        failed.outcome(),
        super::super::super::trace::NativeOutcomeV1::Failed(
            NativeFailureCauseV1::EnvironmentSurfaceChanged,
        )
    );
}

#[test]
fn refused_current_resize_uses_the_accepted_tuple_without_creating_pending_work() {
    let mut driver = driver(PresenterMode::Success);
    driver
        .observe_surface(physical(640, 480), 2.0, tick(0))
        .expect("initial surface should stage");
    driver
        .drain_scheduler(tick(1))
        .expect("initial surface should publish");
    let accepted = driver.accepted_surface();
    let generation = driver.runtime_generation();
    let stats = driver.scheduler_stats();

    assert_eq!(
        driver
            .reject_environment_surface_between_directives(tick(2))
            .expect_err("a refused resize should fail from the accepted observation"),
        NativeFailureCauseV1::EnvironmentSurfaceChanged
    );
    assert_eq!(driver.accepted_surface(), accepted);
    assert_eq!(driver.pending_surface(), None);
    assert_eq!(driver.runtime_generation(), generation);
    assert_eq!(driver.scheduler_stats(), stats);
    assert_eq!(
        driver
            .trace()
            .events()
            .last()
            .and_then(|event| event.surface()),
        accepted
    );
}
