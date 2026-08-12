use std::fmt;

use fenestra_ui_ir::prototype::InvalidationSet;
use fenestra_ui_spatial::prototype::SpatialPaintFrameV2;

use super::super::state::RuntimeGeneration;
use super::super::view::CommittedRuntimeSnapshot;
use super::control::ControlAdmission;
use super::types::{SchedulerTick, VISUAL_ENVELOPE_BYTES};

/// Opaque identity of one renderer offer.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FrameId(u64);

impl FrameId {
    pub(super) const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the numeric identity used by deterministic scheduler traces.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Identity of one ordered renderer submission epoch.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RendererEpoch(u64);

impl RendererEpoch {
    /// Creates a renderer epoch for typed adapter feedback.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the numeric epoch used by deterministic scheduler traces.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Ordered identity assigned only after a renderer accepts an offer.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SubmissionId {
    epoch: RendererEpoch,
    token: u64,
}

impl SubmissionId {
    pub(super) const fn new(epoch: RendererEpoch, token: u64) -> Self {
        Self { epoch, token }
    }

    /// Returns the renderer epoch that owns the submission.
    #[must_use]
    pub const fn epoch(self) -> RendererEpoch {
        self.epoch
    }

    /// Returns the ordered token within the renderer epoch.
    #[must_use]
    pub const fn token(self) -> u64 {
        self.token
    }
}

/// Ordered completion observation for one renderer epoch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompletionWatermark {
    epoch: RendererEpoch,
    token: u64,
}

impl CompletionWatermark {
    /// Creates one typed completion watermark.
    #[must_use]
    pub const fn new(epoch: RendererEpoch, token: u64) -> Self {
        Self { epoch, token }
    }

    /// Creates the watermark that completes through one submission.
    #[must_use]
    pub const fn from_submission(submission: SubmissionId) -> Self {
        Self::new(submission.epoch(), submission.token())
    }

    /// Returns the renderer epoch named by this observation.
    #[must_use]
    pub const fn epoch(self) -> RendererEpoch {
        self.epoch
    }

    /// Returns the inclusive completed submission token.
    #[must_use]
    pub const fn token(self) -> u64 {
        self.token
    }
}

/// Immutable, bounded work offered to a renderer adapter.
#[derive(Clone)]
pub struct FrameWork {
    id: FrameId,
    work: VisualWork,
}

impl FrameWork {
    /// Returns the opaque offer identity.
    #[must_use]
    pub const fn id(&self) -> FrameId {
        self.id
    }

    /// Returns the committed generation represented by this work.
    #[must_use]
    pub fn generation(&self) -> RuntimeGeneration {
        self.work.snapshot.generation()
    }

    /// Returns the immutable committed snapshot retained by this work.
    #[must_use]
    pub const fn snapshot(&self) -> &CommittedRuntimeSnapshot {
        &self.work.snapshot
    }

    /// Returns the invalidation accumulated since the prior submission.
    #[must_use]
    pub const fn invalidation(&self) -> InvalidationSet {
        self.work.invalidation
    }

    /// Returns the earliest unconsumed visual request tick.
    #[must_use]
    pub const fn earliest_tick(&self) -> SchedulerTick {
        self.work.earliest_tick
    }

    /// Returns the latest replacement tick represented by this work.
    #[must_use]
    pub const fn latest_tick(&self) -> SchedulerTick {
        self.work.latest_tick
    }

    /// Returns the fixed V1 protocol-accounted envelope bytes.
    #[must_use]
    pub const fn accounted_bytes(&self) -> usize {
        VISUAL_ENVELOPE_BYTES
    }

    /// Returns the optional paint frame sealed to this offered generation.
    #[must_use]
    pub fn paint_frame(&self) -> Option<RuntimePaintFrameV2<'_>> {
        let spatial = self.work.snapshot.spatial()?.snapshot().paint_frame();
        Some(RuntimePaintFrameV2 {
            generation: self.work.snapshot.generation(),
            spatial,
        })
    }
}

/// Borrowed spatial paint frame sealed to one offered runtime generation.
#[derive(Clone, Copy)]
pub struct RuntimePaintFrameV2<'a> {
    generation: RuntimeGeneration,
    spatial: SpatialPaintFrameV2<'a>,
}

impl<'a> RuntimePaintFrameV2<'a> {
    /// Returns the committed generation represented by this frame.
    #[must_use]
    pub const fn generation(self) -> RuntimeGeneration {
        self.generation
    }

    /// Returns the immutable spatial paint projection for this frame.
    #[must_use]
    pub const fn spatial(self) -> SpatialPaintFrameV2<'a> {
        self.spatial
    }
}

impl fmt::Debug for FrameWork {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FrameWork")
            .field("id", &self.id)
            .field("generation", &self.generation())
            .field("invalidation", &self.invalidation())
            .field("earliest_tick", &self.earliest_tick())
            .field("latest_tick", &self.latest_tick())
            .finish()
    }
}

impl PartialEq for FrameWork {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.invalidation() == other.invalidation()
            && self.earliest_tick() == other.earliest_tick()
            && self.latest_tick() == other.latest_tick()
            && self.snapshot().shares_state_with(other.snapshot())
    }
}

impl Eq for FrameWork {}

/// Closed renderer and platform feedback accepted by the frame scheduler.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerInput {
    /// The platform reached the previously requested frame boundary.
    FrameReady,
    /// The renderer atomically accepted one outstanding offer.
    AcceptFrame(FrameId),
    /// The renderer rejected one outstanding offer without submitting it.
    RejectFrame(FrameId),
    /// The renderer completed an ordered prefix of accepted submissions.
    Complete(CompletionWatermark),
    /// The renderer reported terminal loss for its current epoch.
    RendererLost(RendererEpoch),
    /// The platform requested one idempotent renderer shutdown.
    RequestShutdown,
}

/// Typed result of one accepted scheduler input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerInputResult {
    /// The latest visual work became eligible for a renderer offer.
    FrameReady,
    /// The outstanding offer became an accepted submission.
    FrameAccepted(SubmissionId),
    /// The outstanding offer returned to the replaceable visual lane.
    FrameRejected(FrameId),
    /// One non-droppable control was accepted or recognized idempotently.
    Control(ControlAdmission),
}

#[derive(Clone)]
pub(super) struct VisualWork {
    pub(super) snapshot: CommittedRuntimeSnapshot,
    pub(super) invalidation: InvalidationSet,
    pub(super) earliest_tick: SchedulerTick,
    pub(super) latest_tick: SchedulerTick,
}

impl VisualWork {
    pub(super) fn into_frame(self, id: FrameId) -> FrameWork {
        FrameWork { id, work: self }
    }

    pub(super) fn replace(
        &mut self,
        snapshot: CommittedRuntimeSnapshot,
        invalidation: InvalidationSet,
        tick: SchedulerTick,
    ) {
        self.snapshot = snapshot;
        self.invalidation = self.invalidation.union(invalidation);
        self.latest_tick = tick;
    }
}

impl From<FrameWork> for VisualWork {
    fn from(work: FrameWork) -> Self {
        work.work
    }
}

pub(super) enum VisualState {
    RequestOutstanding {
        work: VisualWork,
        action_pending: bool,
    },
    PendingPublication(VisualWork),
    OfferAwaitingDisposition(FrameWork),
}

impl VisualState {
    pub(super) fn ticks(&self) -> (SchedulerTick, SchedulerTick) {
        match self {
            Self::RequestOutstanding { work, .. } | Self::PendingPublication(work) => {
                (work.earliest_tick, work.latest_tick)
            }
            Self::OfferAwaitingDisposition(work) => (work.earliest_tick(), work.latest_tick()),
        }
    }
}

pub(super) struct SubmittedFrame {
    pub(super) submission: SubmissionId,
    pub(super) accepted_tick: SchedulerTick,
    pub(super) _snapshot: CommittedRuntimeSnapshot,
}
