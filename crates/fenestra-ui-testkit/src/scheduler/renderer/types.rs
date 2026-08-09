use std::error::Error;
use std::fmt;

use fenestra_ui_runtime::prototype::{
    CompletionWatermark, ControlAdmission, FrameId, SchedulerErrorKind, SchedulerTick, SubmissionId,
};

/// Inclusive synthetic-retirement bounds for one fake renderer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FakeRendererCapacityV1 {
    max_items: usize,
    max_bytes: usize,
    residence_ticks: u64,
}

impl FakeRendererCapacityV1 {
    /// Creates explicit item, byte, and residence bounds.
    #[must_use]
    pub const fn new(max_items: usize, max_bytes: usize, residence_ticks: u64) -> Self {
        Self {
            max_items,
            max_bytes,
            residence_ticks,
        }
    }

    /// Returns the inclusive retirement-record ceiling.
    #[must_use]
    pub const fn max_items(self) -> usize {
        self.max_items
    }

    /// Returns the inclusive protocol-accounted byte ceiling.
    #[must_use]
    pub const fn max_bytes(self) -> usize {
        self.max_bytes
    }

    /// Returns the inclusive residence deadline in scheduler ticks.
    #[must_use]
    pub const fn residence_ticks(self) -> u64 {
        self.residence_ticks
    }
}

/// Stable identity of one synthetic renderer resource.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SyntheticResourceIdV1(u64);

impl SyntheticResourceIdV1 {
    /// Creates a synthetic resource identity.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the numeric identity used by deterministic traces.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Declared use of one fixed-size synthetic renderer resource.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyntheticResourceUseV1 {
    id: SyntheticResourceIdV1,
    synthetic_bytes: usize,
}

impl SyntheticResourceUseV1 {
    /// Creates one synthetic resource use declaration.
    #[must_use]
    pub const fn new(id: SyntheticResourceIdV1, synthetic_bytes: usize) -> Self {
        Self {
            id,
            synthetic_bytes,
        }
    }

    /// Returns the resource identity.
    #[must_use]
    pub const fn id(self) -> SyntheticResourceIdV1 {
        self.id
    }

    /// Returns the declared synthetic payload bytes.
    #[must_use]
    pub const fn synthetic_bytes(self) -> usize {
        self.synthetic_bytes
    }
}

/// Closed scripted behavior for one fake renderer offer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FakeRendererModeV1 {
    /// Accept and immediately observe ordered completion.
    Immediate,
    /// Accept while retaining completion for a later explicit input.
    Late,
    /// Reject the offer before accepting a submission.
    Fail,
    /// Inject renderer loss without accepting the outstanding offer.
    Loss,
}

/// Typed result of processing one fake renderer offer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FakeRendererOfferOutcomeV1 {
    /// The offer became an accepted late submission.
    Accepted(SubmissionId),
    /// The offer was accepted and its immediate completion was handled.
    Immediate {
        /// The accepted renderer submission.
        submission: SubmissionId,
        /// The admitted or retained completion control.
        completion: FakeControlDeliveryV1,
    },
    /// The offer was rejected without creating a submission.
    Rejected(FrameId),
    /// Renderer loss was admitted or retained for a later retry.
    Loss(FakeControlDeliveryV1),
}

/// Result of atomically attempting one fake renderer control delivery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FakeControlDeliveryV1 {
    /// The runtime control lane accepted or recognized the control.
    Accepted(ControlAdmission),
    /// Control capacity rejected the record, so the fake retained it.
    Retained(SchedulerErrorKind),
}

/// Closed failures produced by the bounded fake renderer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FakeRendererErrorKindV1 {
    /// The complete projected retirement ledger exceeds an item or byte bound.
    CapacityExceeded,
    /// An accepted retirement record crossed its inclusive residence deadline.
    ResidenceExceeded,
    /// The runtime scheduler rejected the corresponding typed interaction.
    Scheduler(SchedulerErrorKind),
}

/// Privacy-safe fake renderer failure.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct FakeRendererErrorV1 {
    kind: FakeRendererErrorKindV1,
}

impl FakeRendererErrorV1 {
    pub(super) const fn new(kind: FakeRendererErrorKindV1) -> Self {
        Self { kind }
    }

    /// Returns the closed fake renderer failure category.
    #[must_use]
    pub const fn kind(self) -> FakeRendererErrorKindV1 {
        self.kind
    }
}

impl fmt::Debug for FakeRendererErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FakeRendererErrorV1")
            .field("kind", &self.kind)
            .finish()
    }
}

impl fmt::Display for FakeRendererErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fake renderer failed: {:?}", self.kind)
    }
}

impl Error for FakeRendererErrorV1 {}

/// Bounded retirement accounting observed without exposing resource payloads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FakeRendererStatsV1 {
    pub(super) items: usize,
    pub(super) accounted_bytes: usize,
    pub(super) earliest_tick: Option<SchedulerTick>,
    pub(super) latest_tick: Option<SchedulerTick>,
    pub(super) last_accepted: Option<SubmissionId>,
    pub(super) completed: Option<CompletionWatermark>,
    pub(super) has_pending_control: bool,
}

impl FakeRendererStatsV1 {
    /// Returns the retained synthetic-resource count.
    #[must_use]
    pub const fn items(self) -> usize {
        self.items
    }

    /// Returns the retained protocol-accounted bytes.
    #[must_use]
    pub const fn accounted_bytes(self) -> usize {
        self.accounted_bytes
    }

    /// Returns the oldest unretired resource-use tick.
    #[must_use]
    pub const fn earliest_tick(self) -> Option<SchedulerTick> {
        self.earliest_tick
    }

    /// Returns the latest resource-use tick represented by the ledger.
    #[must_use]
    pub const fn latest_tick(self) -> Option<SchedulerTick> {
        self.latest_tick
    }

    /// Returns the latest submission accepted through this fake renderer.
    #[must_use]
    pub const fn last_accepted(self) -> Option<SubmissionId> {
        self.last_accepted
    }

    /// Returns the latest completion admitted through this fake renderer.
    #[must_use]
    pub const fn completed(self) -> Option<CompletionWatermark> {
        self.completed
    }

    /// Reports whether one renderer control is retained for explicit retry.
    #[must_use]
    pub const fn has_pending_control(self) -> bool {
        self.has_pending_control
    }
}
