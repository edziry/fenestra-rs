use std::error::Error;
use std::fmt;

use fenestra_ui_runtime::prototype::{
    CallbackFinish, ControlSequence, FrameId, SchedulerAction, SchedulerErrorKind, SchedulerInput,
    SchedulerInputResult,
};

use super::super::FakeCallbackDepthV1;

/// Inclusive storage bounds for one scheduler trace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerTraceCapacityV1 {
    max_events: usize,
    max_bytes: usize,
}

impl SchedulerTraceCapacityV1 {
    /// Creates explicit event and protocol-byte bounds.
    #[must_use]
    pub const fn new(max_events: usize, max_bytes: usize) -> Self {
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

    /// Returns the inclusive protocol-accounted byte ceiling.
    #[must_use]
    pub const fn max_bytes(self) -> usize {
        self.max_bytes
    }
}

/// Independently bounded scheduler trace dimensions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerTraceLimitV1 {
    /// Number of accepted events.
    Events,
    /// Sum of fixed event accounting weights.
    AccountedBytes,
}

/// Closed failures produced while recording a scheduler trace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerTraceErrorKindV1 {
    /// One declared trace dimension would be exceeded.
    LimitExceeded(SchedulerTraceLimitV1),
    /// The supplied clock belongs to another deterministic domain.
    ClockDomainMismatch,
    /// The supplied clock or captured state moved backward.
    TickRegression,
    /// Checked event, byte, or sequence arithmetic was exhausted.
    ArithmeticExhausted,
}

/// Privacy-safe scheduler trace failure.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SchedulerTraceErrorV1 {
    kind: SchedulerTraceErrorKindV1,
}

impl SchedulerTraceErrorV1 {
    pub(super) const fn new(kind: SchedulerTraceErrorKindV1) -> Self {
        Self { kind }
    }

    pub(super) const fn arithmetic_exhausted() -> Self {
        Self::new(SchedulerTraceErrorKindV1::ArithmeticExhausted)
    }

    /// Returns the closed trace failure category.
    #[must_use]
    pub const fn kind(self) -> SchedulerTraceErrorKindV1 {
        self.kind
    }
}

impl fmt::Debug for SchedulerTraceErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SchedulerTraceErrorV1")
            .field("kind", &self.kind)
            .finish()
    }
}

impl fmt::Display for SchedulerTraceErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "scheduler trace failed: {:?}", self.kind)
    }
}

impl Error for SchedulerTraceErrorV1 {}

/// Closed scheduler stages captured by one trace event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerTraceStageV1 {
    /// A fake platform callback finished or was rejected.
    Callback,
    /// A direct scheduler transaction was attempted.
    Commit,
    /// Typed adapter feedback was attempted.
    Input,
    /// One scheduler turn returned a normalized action or failure.
    Action,
}

/// Owned callback outcome without a transaction or committed snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerTraceCallbackOutcomeV1 {
    /// The outer callback returned its closed finish result.
    Finished(CallbackFinish),
    /// The scheduler rejected callback entry or publication.
    Rejected(SchedulerErrorKind),
}

/// Closed outcome of one direct transaction attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerTraceCommitOutcomeV1 {
    /// The transaction published a new committed generation.
    Published,
    /// The transaction was a true no-op.
    Noop,
    /// The scheduler rejected the transaction.
    Rejected(SchedulerErrorKind),
}

/// Closed outcome of one fake adapter input attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerTraceInputOutcomeV1 {
    /// The runtime accepted the typed input.
    Accepted(SchedulerInputResult),
    /// A fake retained the typed input after transient pressure.
    Retained(SchedulerErrorKind),
    /// A terminal transition made retained input obsolete.
    Canceled,
    /// The runtime rejected the input without fake retention.
    Rejected(SchedulerErrorKind),
}

/// Copyable scheduler action projection that cannot retain frame work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerTraceActionV1 {
    /// No scheduler action was ready.
    Idle,
    /// Request one future platform frame boundary.
    RequestFrame,
    /// Offer the frame identity without retaining its immutable snapshot.
    OfferFrame(FrameId),
    /// Stop renderer intake at one ordered control sequence.
    StopRenderer(ControlSequence),
    /// The scheduler turn failed with a closed reason.
    Rejected(SchedulerErrorKind),
}

impl SchedulerTraceActionV1 {
    /// Normalizes a borrowed scheduler action without cloning frame work.
    #[must_use]
    pub fn from_action(action: Option<&SchedulerAction>) -> Self {
        match action {
            None => Self::Idle,
            Some(SchedulerAction::RequestFrame) => Self::RequestFrame,
            Some(SchedulerAction::OfferFrame(work)) => Self::OfferFrame(work.id()),
            Some(SchedulerAction::StopRenderer(sequence)) => Self::StopRenderer(*sequence),
        }
    }
}

/// One typed scheduler operation and its closed result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerTraceStepV1 {
    /// A callback script ran at one bounded nesting depth.
    Callback {
        /// Deepest scripted callback scope.
        depth: FakeCallbackDepthV1,
        /// Closed callback result.
        outcome: SchedulerTraceCallbackOutcomeV1,
    },
    /// One direct transaction attempt completed.
    Commit(SchedulerTraceCommitOutcomeV1),
    /// One typed adapter input was attempted.
    Input {
        /// Input supplied to the runtime or retained by a fake.
        input: SchedulerInput,
        /// Closed input result.
        outcome: SchedulerTraceInputOutcomeV1,
    },
    /// One scheduler turn returned a normalized action or failure.
    Action(SchedulerTraceActionV1),
}

impl SchedulerTraceStepV1 {
    pub(super) const fn stage(self) -> SchedulerTraceStageV1 {
        match self {
            Self::Callback { .. } => SchedulerTraceStageV1::Callback,
            Self::Commit(_) => SchedulerTraceStageV1::Commit,
            Self::Input { .. } => SchedulerTraceStageV1::Input,
            Self::Action(_) => SchedulerTraceStageV1::Action,
        }
    }

    pub(super) const fn callback_depth(self) -> Option<usize> {
        match self {
            Self::Callback { depth, .. } => Some(match depth {
                FakeCallbackDepthV1::Outer => 1,
                FakeCallbackDepthV1::Nested => 2,
                FakeCallbackDepthV1::Grandchild => 3,
            }),
            _ => None,
        }
    }
}
