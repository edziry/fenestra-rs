use fenestra_ui_runtime::prototype::{
    ControlSequence, HeadlessSurface, QueueCapacity, RuntimeGeneration, SchedulerCapacity,
    SchedulerState, SchedulerStats, SchedulerTick, UiRuntime, UiScheduler,
};
use fenestra_ui_testkit::prototype::{
    HeadlessFixtureV1, HeadlessOracleV1, NormalizedHeadlessProjectionV1,
    compare_headless_projection_v1, observe_headless_projection_v1,
};

use super::super::surface::{NativeSurfaceStateV1, NativeSurfaceTupleV1};
use super::super::trace::{
    NativeFailureCauseV1, NativeObservationV1, NativeOutcomeV1, NativeTraceStageV1,
    NativeTraceStepV1, NativeTraceV1,
};
use super::super::types::NativePhysicalPointV1;
use super::super::types::{NativeContractErrorKindV1, NativeLimitKindV1};
use super::types::PresenterPortV1;

const EMPTY_SURFACE: HeadlessSurface = HeadlessSurface::new(0, 0);

#[derive(Clone, Copy)]
pub(super) enum PendingControlV1 {
    Completion {
        submission: fenestra_ui_runtime::prototype::SubmissionId,
        control: ControlSequence,
    },
    Loss {
        frame: fenestra_ui_runtime::prototype::FrameId,
        submission: fenestra_ui_runtime::prototype::SubmissionId,
        control: ControlSequence,
    },
}

pub(crate) struct NativeDriverV1<P> {
    pub(super) fixture: HeadlessFixtureV1,
    pub(super) scheduler: UiScheduler,
    pub(super) surface: NativeSurfaceStateV1,
    pub(super) pending_pointer: Option<NativePhysicalPointV1>,
    pub(super) trace: NativeTraceV1,
    pub(super) presenter: P,
    pub(super) presenter_pending: bool,
    pub(super) redraw_armed: bool,
    pub(super) scheduler_turn: u64,
    pub(super) pending_control: Option<PendingControlV1>,
    pub(super) retiring_submission: Option<fenestra_ui_runtime::prototype::SubmissionId>,
    pub(super) shutdown_control: Option<ControlSequence>,
}

impl<P: PresenterPortV1> NativeDriverV1<P> {
    pub(crate) fn new(presenter: P) -> Result<Self, NativeFailureCauseV1> {
        Self::with_trace(presenter, NativeTraceV1::new())
    }

    fn with_trace(presenter: P, trace: NativeTraceV1) -> Result<Self, NativeFailureCauseV1> {
        let fixture = HeadlessFixtureV1::build().map_err(|_| NativeFailureCauseV1::Runtime)?;
        let mut oracle =
            HeadlessOracleV1::new(&fixture).map_err(|_| NativeFailureCauseV1::Oracle)?;
        oracle
            .resize(EMPTY_SURFACE)
            .map_err(|_| NativeFailureCauseV1::Oracle)?;
        let expected = oracle.rebuild().map_err(|_| NativeFailureCauseV1::Oracle)?;
        let runtime = UiRuntime::new_headless(
            fixture.style().clone(),
            fixture.spec(),
            EMPTY_SURFACE,
            fixture.runtime_capacity(),
        )
        .map_err(|_| NativeFailureCauseV1::Runtime)?;
        let scheduler = UiScheduler::new(runtime, scheduler_capacity())
            .map_err(|_| NativeFailureCauseV1::Scheduler)?;
        let mut driver = Self {
            fixture,
            scheduler,
            surface: NativeSurfaceStateV1::new(),
            pending_pointer: None,
            trace,
            presenter,
            presenter_pending: false,
            redraw_armed: false,
            scheduler_turn: 0,
            pending_control: None,
            retiring_submission: None,
            shutdown_control: None,
        };
        let manifest = NativeTraceStepV1::new(
            NativeTraceStageV1::Manifest,
            NativeObservationV1::Build,
            NativeOutcomeV1::Observed,
        );
        driver.record(tick(0), manifest)?;
        driver.compare_projection(&expected)?;
        Ok(driver)
    }

    #[cfg(test)]
    pub(crate) fn with_trace_capacity_for_test(
        presenter: P,
        capacity: super::super::trace::NativeTraceCapacityV1,
    ) -> Result<Self, NativeFailureCauseV1> {
        Self::with_trace(presenter, NativeTraceV1::with_capacity_for_test(capacity))
    }

    pub(crate) fn runtime_generation(&self) -> RuntimeGeneration {
        self.scheduler.committed().generation()
    }

    pub(crate) const fn accepted_surface(&self) -> Option<NativeSurfaceTupleV1> {
        self.surface.accepted_tuple()
    }

    pub(crate) const fn pending_surface(&self) -> Option<NativeSurfaceTupleV1> {
        self.surface.pending_tuple()
    }

    pub(crate) const fn pending_pointer_count(&self) -> usize {
        if self.pending_pointer.is_some() { 1 } else { 0 }
    }

    pub(crate) const fn redraw_armed(&self) -> bool {
        self.redraw_armed
    }

    pub(crate) const fn presenter_pending_count(&self) -> usize {
        if self.presenter_pending { 1 } else { 0 }
    }

    pub(crate) const fn presenter(&self) -> &P {
        &self.presenter
    }

    pub(crate) const fn scheduler_state(&self) -> SchedulerState {
        self.scheduler.state()
    }

    pub(crate) fn scheduler_stats(&self) -> SchedulerStats {
        self.scheduler.stats()
    }

    pub(crate) const fn trace(&self) -> &NativeTraceV1 {
        &self.trace
    }

    pub(super) fn expected_projection(
        &self,
        surface: HeadlessSurface,
    ) -> Result<NormalizedHeadlessProjectionV1, NativeFailureCauseV1> {
        let mut oracle =
            HeadlessOracleV1::new(&self.fixture).map_err(|_| NativeFailureCauseV1::Oracle)?;
        oracle
            .resize(surface)
            .map_err(|_| NativeFailureCauseV1::Oracle)?;
        oracle.rebuild().map_err(|_| NativeFailureCauseV1::Oracle)
    }

    pub(super) fn compare_projection(
        &self,
        expected: &NormalizedHeadlessProjectionV1,
    ) -> Result<(), NativeFailureCauseV1> {
        let snapshot = self.scheduler.committed();
        let observed = observe_headless_projection_v1(&self.fixture, &snapshot)
            .map_err(|_| NativeFailureCauseV1::Oracle)?;
        if observed.generation() != snapshot.generation()
            || compare_headless_projection_v1(expected, observed.projection())
                .map_err(|_| NativeFailureCauseV1::Oracle)?
                .is_some()
        {
            return Err(NativeFailureCauseV1::Oracle);
        }
        Ok(())
    }
}

pub(super) const fn map_contract_error(error: NativeContractErrorKindV1) -> NativeFailureCauseV1 {
    match error {
        NativeContractErrorKindV1::InvalidScale => NativeFailureCauseV1::InvalidScale,
        NativeContractErrorKindV1::InvalidPoint => NativeFailureCauseV1::InvalidPoint,
        NativeContractErrorKindV1::EnvironmentScaleChanged => {
            NativeFailureCauseV1::EnvironmentScaleChanged
        }
        NativeContractErrorKindV1::ArithmeticExhausted => NativeFailureCauseV1::Arithmetic,
        NativeContractErrorKindV1::LimitExceeded(NativeLimitKindV1::Width) => {
            NativeFailureCauseV1::WidthLimit
        }
        NativeContractErrorKindV1::LimitExceeded(NativeLimitKindV1::Height) => {
            NativeFailureCauseV1::HeightLimit
        }
        NativeContractErrorKindV1::LimitExceeded(NativeLimitKindV1::Pixels) => {
            NativeFailureCauseV1::PixelLimit
        }
        NativeContractErrorKindV1::LimitExceeded(NativeLimitKindV1::Bytes) => {
            NativeFailureCauseV1::ByteLimit
        }
        NativeContractErrorKindV1::InvalidRectangle(_) => NativeFailureCauseV1::Arithmetic,
        NativeContractErrorKindV1::UnsupportedAlpha(_) => NativeFailureCauseV1::UnsupportedAlpha,
        NativeContractErrorKindV1::Allocation => NativeFailureCauseV1::Storage,
        NativeContractErrorKindV1::Invariant => NativeFailureCauseV1::Invariant,
    }
}

const fn scheduler_capacity() -> SchedulerCapacity {
    SchedulerCapacity::new(
        QueueCapacity::new(1, 80, 8),
        QueueCapacity::new(4, 128, 8),
        QueueCapacity::new(1, 40, 8),
        QueueCapacity::new(2, 80, 8),
    )
}

const fn tick(value: u64) -> SchedulerTick {
    SchedulerTick::new(value)
}
