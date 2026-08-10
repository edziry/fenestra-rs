use fenestra_ui_runtime::prototype::RuntimeGeneration;
use fenestra_ui_testkit::prototype::HeadlessPointerTargetV1;

use super::super::surface::NativeSurfaceTupleV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeTraceStageV1 {
    Manifest,
    Shell,
    Platform,
    Scheduler,
    Renderer,
    Oracle,
}

impl NativeTraceStageV1 {
    pub(crate) const ALL: [Self; 6] = [
        Self::Manifest,
        Self::Shell,
        Self::Platform,
        Self::Scheduler,
        Self::Renderer,
        Self::Oracle,
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeObservationV1 {
    Build,
    Resumed,
    Surface,
    Scale,
    Pointer,
    Redraw,
    Frame,
    Present,
    Close,
    Completion,
    Shutdown,
    Timeout,
}

impl NativeObservationV1 {
    pub(crate) const ALL: [Self; 12] = [
        Self::Build,
        Self::Resumed,
        Self::Surface,
        Self::Scale,
        Self::Pointer,
        Self::Redraw,
        Self::Frame,
        Self::Present,
        Self::Close,
        Self::Completion,
        Self::Shutdown,
        Self::Timeout,
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeFailureCauseV1 {
    InvalidScale,
    InvalidPoint,
    Arithmetic,
    WidthLimit,
    HeightLimit,
    PixelLimit,
    ByteLimit,
    UnsupportedAlpha,
    Storage,
    EnvironmentScaleChanged,
    SurfaceRepaintUnavailable,
    Runtime,
    Oracle,
    Scheduler,
    PrePresent,
    Presenter,
    Trace,
    Timeout,
    Invariant,
}

impl NativeFailureCauseV1 {
    pub(crate) const ALL: [Self; 19] = [
        Self::InvalidScale,
        Self::InvalidPoint,
        Self::Arithmetic,
        Self::WidthLimit,
        Self::HeightLimit,
        Self::PixelLimit,
        Self::ByteLimit,
        Self::UnsupportedAlpha,
        Self::Storage,
        Self::EnvironmentScaleChanged,
        Self::SurfaceRepaintUnavailable,
        Self::Runtime,
        Self::Oracle,
        Self::Scheduler,
        Self::PrePresent,
        Self::Presenter,
        Self::Trace,
        Self::Timeout,
        Self::Invariant,
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeOutcomeV1 {
    Observed,
    Coalesced,
    Ignored,
    Deferred,
    Published,
    Armed,
    Offered,
    Accepted,
    Rejected,
    Completed,
    Matched,
    Stopped,
    Failed(NativeFailureCauseV1),
}

impl NativeOutcomeV1 {
    pub(crate) const ALL: [Self; 13] = [
        Self::Observed,
        Self::Coalesced,
        Self::Ignored,
        Self::Deferred,
        Self::Published,
        Self::Armed,
        Self::Offered,
        Self::Accepted,
        Self::Rejected,
        Self::Completed,
        Self::Matched,
        Self::Stopped,
        Self::Failed(NativeFailureCauseV1::Trace),
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeTraceLaneStatsV1 {
    items: usize,
    accounted_bytes: usize,
}

impl NativeTraceLaneStatsV1 {
    pub(crate) const fn new(items: usize, accounted_bytes: usize) -> Self {
        Self {
            items,
            accounted_bytes,
        }
    }

    pub(crate) const fn items(self) -> usize {
        self.items
    }

    pub(crate) const fn accounted_bytes(self) -> usize {
        self.accounted_bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeTracePendingV1 {
    surface: usize,
    pointer: usize,
    presenter: usize,
}

impl NativeTracePendingV1 {
    pub(crate) const fn new(surface: usize, pointer: usize, presenter: usize) -> Self {
        Self {
            surface,
            pointer,
            presenter,
        }
    }

    pub(crate) const fn is_bounded(self) -> bool {
        self.surface <= 1 && self.pointer <= 1 && self.presenter <= 1
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeTraceSubmissionV1 {
    epoch: u64,
    token: u64,
}

impl NativeTraceSubmissionV1 {
    pub(crate) const fn new(epoch: u64, token: u64) -> Self {
        Self { epoch, token }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeTraceStepV1 {
    pub(crate) stage: NativeTraceStageV1,
    pub(crate) observation: NativeObservationV1,
    pub(crate) outcome: NativeOutcomeV1,
    pub(crate) scheduler_turn: Option<u64>,
    pub(crate) captured_generation: Option<RuntimeGeneration>,
    pub(crate) published_generation: Option<RuntimeGeneration>,
    pub(crate) surface: Option<NativeSurfaceTupleV1>,
    pub(crate) target: Option<HeadlessPointerTargetV1>,
    pub(crate) frame: Option<u64>,
    pub(crate) submission: Option<NativeTraceSubmissionV1>,
    pub(crate) control: Option<u64>,
    pub(crate) redraw_armed: bool,
    pub(crate) pending: NativeTracePendingV1,
    pub(crate) deferred: NativeTraceLaneStatsV1,
    pub(crate) controls: NativeTraceLaneStatsV1,
    pub(crate) visual: NativeTraceLaneStatsV1,
    pub(crate) in_flight: NativeTraceLaneStatsV1,
}

impl NativeTraceStepV1 {
    pub(crate) const fn new(
        stage: NativeTraceStageV1,
        observation: NativeObservationV1,
        outcome: NativeOutcomeV1,
    ) -> Self {
        Self {
            stage,
            observation,
            outcome,
            scheduler_turn: None,
            captured_generation: None,
            published_generation: None,
            surface: None,
            target: None,
            frame: None,
            submission: None,
            control: None,
            redraw_armed: false,
            pending: NativeTracePendingV1::new(0, 0, 0),
            deferred: NativeTraceLaneStatsV1::new(0, 0),
            controls: NativeTraceLaneStatsV1::new(0, 0),
            visual: NativeTraceLaneStatsV1::new(0, 0),
            in_flight: NativeTraceLaneStatsV1::new(0, 0),
        }
    }
}
