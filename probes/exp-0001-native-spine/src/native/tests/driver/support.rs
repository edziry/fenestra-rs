use fenestra_ui_runtime::prototype::{RuntimeGeneration, SchedulerState, SchedulerTick};

use super::super::super::driver::{NativeDriverV1, PresenterPortV1};
use super::super::super::raster::CpuFrameV1;
use super::super::super::trace::NativeFailureCauseV1;
use super::super::super::{NativePhysicalExtentV1, NativeSurfaceTupleV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PresenterMode {
    Success,
    FailPreflight,
    FailNotify,
    FailPresent,
}

pub(super) struct TestPresenter {
    mode: PresenterMode,
    preflight_count: usize,
    notify_count: usize,
    present_count: usize,
    last_generation: Option<RuntimeGeneration>,
    last_surface: Option<NativeSurfaceTupleV1>,
    last_digest: Option<u64>,
}

impl TestPresenter {
    pub(super) const fn new(mode: PresenterMode) -> Self {
        Self {
            mode,
            preflight_count: 0,
            notify_count: 0,
            present_count: 0,
            last_generation: None,
            last_surface: None,
            last_digest: None,
        }
    }

    pub(super) const fn preflight_count(&self) -> usize {
        self.preflight_count
    }

    pub(super) const fn notify_count(&self) -> usize {
        self.notify_count
    }

    pub(super) const fn present_count(&self) -> usize {
        self.present_count
    }

    pub(super) const fn last_generation(&self) -> Option<RuntimeGeneration> {
        self.last_generation
    }

    pub(super) const fn last_surface(&self) -> Option<NativeSurfaceTupleV1> {
        self.last_surface
    }

    pub(super) const fn last_digest(&self) -> Option<u64> {
        self.last_digest
    }
}

impl PresenterPortV1 for TestPresenter {
    fn preflight(&mut self, _frame: &CpuFrameV1) -> Result<(), NativeFailureCauseV1> {
        self.preflight_count += 1;
        if self.mode == PresenterMode::FailPreflight {
            Err(NativeFailureCauseV1::Presenter)
        } else {
            Ok(())
        }
    }

    fn pre_present_notify(&mut self, _frame: &CpuFrameV1) -> Result<(), NativeFailureCauseV1> {
        self.notify_count += 1;
        if self.mode == PresenterMode::FailNotify {
            Err(NativeFailureCauseV1::PrePresent)
        } else {
            Ok(())
        }
    }

    fn present(&mut self, frame: CpuFrameV1) -> Result<(), NativeFailureCauseV1> {
        self.present_count += 1;
        if self.mode == PresenterMode::FailPresent {
            return Err(NativeFailureCauseV1::Presenter);
        }
        self.last_generation = Some(frame.runtime_generation());
        self.last_surface = Some(frame.surface_tuple());
        self.last_digest = Some(frame.digest());
        Ok(())
    }
}

pub(super) fn driver(mode: PresenterMode) -> NativeDriverV1<TestPresenter> {
    NativeDriverV1::new(TestPresenter::new(mode)).expect("registered native driver should build")
}

pub(super) const fn physical(width: u32, height: u32) -> NativePhysicalExtentV1 {
    NativePhysicalExtentV1::new(width, height)
}

pub(super) const fn tick(value: u64) -> SchedulerTick {
    SchedulerTick::new(value)
}

pub(super) fn assert_terminal_empty(driver: &NativeDriverV1<TestPresenter>) {
    let stats = driver.scheduler_stats();
    assert_eq!(driver.scheduler_state(), SchedulerState::Stopped);
    assert_eq!(stats.deferred().items(), 0);
    assert_eq!(stats.controls().items(), 0);
    assert_eq!(stats.visual().items(), 0);
    assert_eq!(stats.in_flight().items(), 0);
    assert!(!driver.redraw_armed());
    assert_eq!(driver.presenter_pending_count(), 0);
}
