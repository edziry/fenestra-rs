use fenestra_ui_runtime::prototype::{
    ControlSequence, FrameId, HeadlessSurface, QueueStats, RuntimeGeneration,
};

use crate::headless::HeadlessPointerTargetV1;
use crate::scheduler::FakeRendererStatsV1;

/// Closed failure sources for the fixed headless experiment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessFailureCauseV1 {
    /// Runtime construction, mutation, or publication failed.
    Runtime,
    /// Candidate projection observation or comparison failed.
    Projection,
    /// Independent clean-rebuild oracle work failed.
    Oracle,
    /// Scheduler protocol work failed.
    Scheduler,
    /// Fake renderer work failed.
    Renderer,
    /// Bounded trace recording failed.
    Trace,
}

/// Closed stages represented in the headless trace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessTraceStageV1 {
    /// Fixed fixture construction.
    Build,
    /// Synthetic input observation or admission.
    Input,
    /// A committed-only callback scope.
    Callback,
    /// One direct logical transaction.
    Transaction,
    /// Independent projection comparison.
    Projection,
    /// One scheduler turn.
    Scheduler,
    /// Fake renderer feedback.
    Renderer,
}

/// Closed causal input vocabulary for one trace event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessInputKindV1 {
    /// No external or logical input is associated with the event.
    None,
    /// Headless pointer query or callback input.
    Pointer,
    /// Direct property mutation.
    Direct,
    /// Keyed insertion.
    Insert,
    /// Keyed move.
    Move,
    /// Keyed update.
    Update,
    /// Keyed removal.
    Remove,
    /// Headless surface resize.
    Resize,
    /// Frame-ready delivery.
    FrameReady,
    /// Renderer completion.
    Completion,
    /// Renderer loss.
    Loss,
    /// Shutdown request.
    Shutdown,
}

/// Closed result of one headless trace event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessOutcomeV1 {
    /// State was observed without mutation.
    Observed,
    /// Callback work was deferred.
    Deferred,
    /// One effective generation was published.
    Published,
    /// The operation was a true no-op.
    NoChange,
    /// Candidate and clean projection matched.
    Matched,
    /// One scheduler action or idle result was produced.
    Action,
    /// Input or renderer work was accepted.
    Accepted,
    /// Renderer work was recoverably rejected.
    Rejected,
    /// One completion was admitted or processed.
    Completed,
    /// Renderer loss was admitted or processed.
    Lost,
    /// The scheduler reached its stopped state.
    Stopped,
    /// The operation failed with one privacy-safe source.
    Failed(HeadlessFailureCauseV1),
}

/// Inclusive storage bounds for one headless trace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeadlessTraceCapacityV1 {
    max_events: usize,
    max_bytes: usize,
}

impl HeadlessTraceCapacityV1 {
    pub(crate) const fn new(max_events: usize, max_bytes: usize) -> Self {
        Self {
            max_events,
            max_bytes,
        }
    }

    /// Returns the inclusive accepted-event ceiling.
    #[must_use]
    pub const fn max_events(self) -> usize {
        self.max_events
    }

    /// Returns the inclusive accounted-byte ceiling.
    #[must_use]
    pub const fn max_bytes(self) -> usize {
        self.max_bytes
    }
}

/// Copyable counts for the five deterministic projection families.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeadlessTraceProjectionCountsV1 {
    computed_styles: usize,
    geometries: usize,
    semantics: usize,
    hit_regions: usize,
    scene_rectangles: usize,
}

impl HeadlessTraceProjectionCountsV1 {
    pub(in crate::headless) const fn new(
        computed_styles: usize,
        geometries: usize,
        semantics: usize,
        hit_regions: usize,
        scene_rectangles: usize,
    ) -> Self {
        Self {
            computed_styles,
            geometries,
            semantics,
            hit_regions,
            scene_rectangles,
        }
    }

    /// Returns the computed-style record count.
    #[must_use]
    pub const fn computed_styles(self) -> usize {
        self.computed_styles
    }

    /// Returns the geometry record count.
    #[must_use]
    pub const fn geometries(self) -> usize {
        self.geometries
    }

    /// Returns the semantic record count.
    #[must_use]
    pub const fn semantics(self) -> usize {
        self.semantics
    }

    /// Returns the hit-region count.
    #[must_use]
    pub const fn hit_regions(self) -> usize {
        self.hit_regions
    }

    /// Returns the scene-rectangle count.
    #[must_use]
    pub const fn scene_rectangles(self) -> usize {
        self.scene_rectangles
    }
}

/// Copyable item and byte accounting for one scheduler lane.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeadlessTraceQueueStatsV1 {
    items: usize,
    accounted_bytes: usize,
}

impl HeadlessTraceQueueStatsV1 {
    pub(super) const fn project(stats: QueueStats) -> Self {
        Self {
            items: stats.items(),
            accounted_bytes: stats.accounted_bytes(),
        }
    }

    /// Returns the retained item count.
    #[must_use]
    pub const fn items(self) -> usize {
        self.items
    }

    /// Returns protocol-accounted bytes.
    #[must_use]
    pub const fn accounted_bytes(self) -> usize {
        self.accounted_bytes
    }
}

/// Copyable item and byte accounting for the fake renderer ledger.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeadlessTraceRendererStatsV1 {
    items: usize,
    accounted_bytes: usize,
}

impl HeadlessTraceRendererStatsV1 {
    pub(super) fn project(stats: FakeRendererStatsV1) -> Self {
        Self {
            items: stats.items(),
            accounted_bytes: stats.accounted_bytes(),
        }
    }

    /// Returns the retained resource count.
    #[must_use]
    pub const fn items(self) -> usize {
        self.items
    }

    /// Returns protocol-accounted bytes.
    #[must_use]
    pub const fn accounted_bytes(self) -> usize {
        self.accounted_bytes
    }
}

#[derive(Clone, Copy)]
pub(crate) struct HeadlessTraceStep {
    pub(crate) stage: HeadlessTraceStageV1,
    pub(crate) input: HeadlessInputKindV1,
    pub(crate) outcome: HeadlessOutcomeV1,
    pub(crate) captured_generation: Option<RuntimeGeneration>,
    pub(crate) records_publication: bool,
    pub(crate) target: HeadlessPointerTargetV1,
    pub(crate) frame: Option<FrameId>,
    pub(crate) control: Option<ControlSequence>,
}

impl HeadlessTraceStep {
    pub(crate) const fn new(
        stage: HeadlessTraceStageV1,
        input: HeadlessInputKindV1,
        outcome: HeadlessOutcomeV1,
    ) -> Self {
        Self {
            stage,
            input,
            outcome,
            captured_generation: None,
            records_publication: false,
            target: HeadlessPointerTargetV1::None,
            frame: None,
            control: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HeadlessTraceErrorKind {
    EventLimitExceeded,
    ByteLimitExceeded,
    ClockDomainMismatch,
    TickRegression,
    ArithmeticExhausted,
    ProjectionUnavailable,
    IdentityMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HeadlessTraceError {
    kind: HeadlessTraceErrorKind,
}

impl HeadlessTraceError {
    pub(crate) const fn new(kind: HeadlessTraceErrorKind) -> Self {
        Self { kind }
    }

    pub(crate) const fn arithmetic_exhausted() -> Self {
        Self::new(HeadlessTraceErrorKind::ArithmeticExhausted)
    }

    #[cfg(test)]
    pub(super) const fn kind(self) -> HeadlessTraceErrorKind {
        self.kind
    }
}

pub(super) struct HeadlessTraceState {
    pub(super) surface: HeadlessSurface,
    pub(super) counts: HeadlessTraceProjectionCountsV1,
    pub(super) generation: RuntimeGeneration,
}
