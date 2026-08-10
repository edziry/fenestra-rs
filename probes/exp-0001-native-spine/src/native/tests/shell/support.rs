use fenestra_ui_runtime::prototype::SchedulerTick;
use fenestra_ui_testkit::prototype::HeadlessPointerTargetV1;

use super::super::super::driver::{
    NativeDriverActionV1, NativeDriverV1, NativeRedrawResultV1, PresenterPortV1,
};
use super::super::super::raster::CpuFrameV1;
use super::super::super::trace::{NativeFailureCauseV1, NativeInputSourceV1};
use super::super::super::types::{NativePhysicalExtentV1, NativePhysicalPointV1};

pub(super) struct AcceptingPresenter;

impl PresenterPortV1 for AcceptingPresenter {
    fn present_offer<A>(
        &mut self,
        _frame: CpuFrameV1,
        accept_once: A,
    ) -> Result<(), NativeFailureCauseV1>
    where
        A: FnOnce() -> Result<fenestra_ui_runtime::prototype::SubmissionId, NativeFailureCauseV1>,
    {
        accept_once()?;
        Ok(())
    }
}

pub(super) fn completed_reference_driver() -> NativeDriverV1<AcceptingPresenter> {
    let mut driver =
        NativeDriverV1::new(AcceptingPresenter).expect("reference driver should initialize");
    driver
        .observe_surface(NativePhysicalExtentV1::new(640, 480), 2.0, tick(0))
        .expect("initial surface should stage");
    assert_eq!(
        driver.drain_scheduler(tick(1)),
        Ok(NativeDriverActionV1::RequestFrame {
            generation: driver.runtime_generation(),
            surface_generation: driver
                .accepted_surface()
                .expect("initial surface should publish")
                .generation(),
        })
    );
    let NativeRedrawResultV1::Presented {
        frame,
        submission,
        completion_control,
    } = driver
        .redraw_requested(tick(2))
        .expect("first frame should present")
    else {
        panic!("armed first redraw should present");
    };
    assert_eq!(driver.runtime_generation().get(), 1);
    assert_eq!(frame.get(), 0);
    assert_eq!(submission.token(), 0);
    assert_eq!(completion_control.get(), 0);

    driver
        .cursor_moved(
            NativePhysicalPointV1::new(10.0, 10.0),
            NativeInputSourceV1::Scripted,
            tick(3),
        )
        .expect("scripted cursor should reduce");
    assert_eq!(
        driver
            .pointer_pressed(NativeInputSourceV1::Scripted, tick(3))
            .expect("scripted press should reduce"),
        HeadlessPointerTargetV1::StaticControl
    );

    driver
        .observe_surface(NativePhysicalExtentV1::new(720, 520), 2.0, tick(4))
        .expect("fixed resize should stage");
    driver
        .drain_scheduler(tick(5))
        .expect("fixed resize should publish");
    let NativeRedrawResultV1::Presented {
        frame,
        submission,
        completion_control,
    } = driver
        .redraw_requested(tick(6))
        .expect("second frame should present")
    else {
        panic!("armed second redraw should present");
    };
    assert_eq!(driver.runtime_generation().get(), 2);
    assert_eq!(frame.get(), 1);
    assert_eq!(submission.token(), 1);
    assert_eq!(completion_control.get(), 1);

    let admission = driver
        .close_requested(NativeInputSourceV1::Scripted, tick(7))
        .expect("scripted close should admit");
    let control = match admission {
        fenestra_ui_runtime::prototype::ControlAdmission::Accepted(control)
        | fenestra_ui_runtime::prototype::ControlAdmission::AlreadyAccepted(control) => control,
    };
    assert_eq!(control.get(), 2);
    assert_eq!(
        driver
            .drain_scheduler(tick(8))
            .expect("stop control should drain"),
        NativeDriverActionV1::StopRenderer { control }
    );
    driver
}

pub(super) const fn tick(value: u64) -> SchedulerTick {
    SchedulerTick::new(value)
}
