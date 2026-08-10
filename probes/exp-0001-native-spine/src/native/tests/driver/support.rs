use fenestra_ui_runtime::prototype::{RuntimeGeneration, SchedulerState, SchedulerTick};

use super::super::super::driver::{NativeDriverV1, PresenterPortV1};
use super::super::super::raster::CpuFrameV1;
use super::super::super::trace::NativeFailureCauseV1;
use super::super::super::{NativePhysicalExtentV1, NativePhysicalPointV1, NativeSurfaceTupleV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PresenterMode {
    Success,
    FailPreflight,
    FailNotify,
    FailPresent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PresenterPhase {
    Preflight,
    PrePresent,
    Accept,
    Present,
}

pub(super) struct TestPresenter {
    mode: PresenterMode,
    preflight_count: usize,
    notify_count: usize,
    present_count: usize,
    last_generation: Option<RuntimeGeneration>,
    last_surface: Option<NativeSurfaceTupleV1>,
    last_digest: Option<u64>,
    phases: Vec<PresenterPhase>,
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
            phases: Vec::new(),
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

    pub(super) fn phases(&self) -> &[PresenterPhase] {
        &self.phases
    }
}

impl PresenterPortV1 for TestPresenter {
    fn present_offer<A>(
        &mut self,
        frame: CpuFrameV1,
        accept_once: A,
    ) -> Result<(), NativeFailureCauseV1>
    where
        A: FnOnce() -> Result<fenestra_ui_runtime::prototype::SubmissionId, NativeFailureCauseV1>,
    {
        self.phases.push(PresenterPhase::Preflight);
        self.preflight_count += 1;
        if self.mode == PresenterMode::FailPreflight {
            return Err(NativeFailureCauseV1::Presenter);
        }

        self.phases.push(PresenterPhase::PrePresent);
        self.notify_count += 1;
        if self.mode == PresenterMode::FailNotify {
            return Err(NativeFailureCauseV1::PrePresent);
        }

        self.phases.push(PresenterPhase::Accept);
        accept_once()?;
        self.phases.push(PresenterPhase::Present);
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

pub(super) const fn scripted_point() -> NativePhysicalPointV1 {
    NativePhysicalPointV1::new(10.0, 10.0)
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
    assert!(driver.pending_surface().is_none());
    assert_eq!(driver.pending_pointer_count(), 0);
    assert_eq!(driver.presenter_pending_count(), 0);
}
