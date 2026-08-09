use std::error::Error;
use std::fmt;

use fenestra_ui_ir::prototype::InvalidationSet;

use super::super::error::TransactionErrorKind;
use super::super::state::RuntimeGeneration;
use super::control::ControlSequence;
use super::frame::FrameWork;

pub(super) const VISUAL_ENVELOPE_BYTES: usize = 40;

/// Monotonic tick supplied by one scheduler clock domain.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SchedulerTick(u64);

impl SchedulerTick {
    /// Creates a scheduler tick in the caller-selected clock domain.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the numeric tick used by deterministic experiment traces.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Inclusive item, protocol-byte, and residence bounds for one queue lane.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueueCapacity {
    max_items: usize,
    max_bytes: usize,
    residence_ticks: u64,
}

impl QueueCapacity {
    /// Creates explicit bounds for one scheduler lane.
    #[must_use]
    pub const fn new(max_items: usize, max_bytes: usize, residence_ticks: u64) -> Self {
        Self {
            max_items,
            max_bytes,
            residence_ticks,
        }
    }

    /// Returns the inclusive item ceiling.
    #[must_use]
    pub const fn max_items(self) -> usize {
        self.max_items
    }

    /// Returns the inclusive protocol-accounted byte ceiling.
    #[must_use]
    pub const fn max_bytes(self) -> usize {
        self.max_bytes
    }

    /// Returns the deadline-to-pressure threshold in scheduler ticks.
    #[must_use]
    pub const fn residence_ticks(self) -> u64 {
        self.residence_ticks
    }
}

/// Explicit capacities for every runtime-owned scheduler lane.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerCapacity {
    deferred: QueueCapacity,
    controls: QueueCapacity,
    visual: QueueCapacity,
    in_flight: QueueCapacity,
}

impl SchedulerCapacity {
    /// Creates the complete scheduler capacity set.
    #[must_use]
    pub const fn new(
        deferred: QueueCapacity,
        controls: QueueCapacity,
        visual: QueueCapacity,
        in_flight: QueueCapacity,
    ) -> Self {
        Self {
            deferred,
            controls,
            visual,
            in_flight,
        }
    }

    /// Returns the deferred-mutation lane capacity.
    #[must_use]
    pub const fn deferred(self) -> QueueCapacity {
        self.deferred
    }

    /// Returns the non-droppable control lane capacity.
    #[must_use]
    pub const fn controls(self) -> QueueCapacity {
        self.controls
    }

    /// Returns the replaceable visual lane capacity.
    #[must_use]
    pub const fn visual(self) -> QueueCapacity {
        self.visual
    }

    /// Returns the submitted-frame lane capacity.
    #[must_use]
    pub const fn in_flight(self) -> QueueCapacity {
        self.in_flight
    }
}

/// Scheduler lanes in deterministic diagnostic order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerLane {
    /// One deferred callback transaction.
    Deferred,
    /// Accepted non-droppable controls.
    Controls,
    /// Coalesced request, pending publication, or outstanding offer.
    Visual,
    /// Accepted renderer submissions.
    InFlight,
}

/// Closed lifecycle states for the single-owner scheduler.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerState {
    /// Ordinary commits and adapter interactions are accepted.
    Running,
    /// One idempotent shutdown control is waiting for delivery.
    ShutdownQueued,
    /// Shutdown was delivered while accepted submissions remain.
    Draining,
    /// Shutdown completed after every accepted submission retired.
    Stopped,
    /// A typed terminal pressure or renderer failure was observed.
    Faulted,
}

/// Result of explicitly canceling replaceable visual work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VisualCancelResult {
    /// One request, pending publication, or unaccepted offer was canceled.
    Canceled,
    /// No replaceable visual work remained.
    AlreadyEmpty,
}

/// Closed failure categories for the scheduler prototype.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerErrorKind {
    /// A lane cannot hold its mandatory protocol state.
    CapacityTooSmall(SchedulerLane),
    /// Runtime retention cannot cover submissions plus a publication edge.
    RetainedGenerationCapacity,
    /// The supplied logical clock moved backward.
    TickRegression,
    /// Adapter feedback does not match the current scheduler phase.
    InputOutOfOrder,
    /// Adapter feedback names an offer other than the outstanding one.
    FrameIdMismatch,
    /// An unresolved renderer offer blocks another state publication.
    ControlPending,
    /// One bounded lane cannot accept the complete typed record.
    CapacityExceeded(SchedulerLane),
    /// A checked scheduler identifier or accounting operation overflowed.
    ArithmeticExhausted,
    /// One bounded lane crossed its inclusive residence deadline.
    ResidenceExceeded(SchedulerLane),
    /// A completion observation names another renderer epoch.
    ForeignRendererEpoch,
    /// A completion observation regressed below the accepted watermark.
    CompletionRegression,
    /// A completion observation exceeds the latest accepted submission.
    CompletionBeyondAccepted,
    /// The underlying atomic transaction failed.
    Transaction(TransactionErrorKind),
}

/// Typed scheduler failure without private state or payload values.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SchedulerError {
    kind: SchedulerErrorKind,
    operation_index: Option<usize>,
}

impl SchedulerError {
    pub(super) const fn new(kind: SchedulerErrorKind, operation_index: Option<usize>) -> Self {
        Self {
            kind,
            operation_index,
        }
    }

    /// Returns the closed scheduler failure category.
    #[must_use]
    pub const fn kind(self) -> SchedulerErrorKind {
        self.kind
    }

    /// Returns the staged operation index when the transaction supplied one.
    #[must_use]
    pub const fn operation_index(self) -> Option<usize> {
        self.operation_index
    }
}

impl fmt::Debug for SchedulerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SchedulerError")
            .field("kind", &self.kind)
            .field("operation_index", &self.operation_index)
            .finish()
    }
}

impl fmt::Display for SchedulerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "runtime scheduling failed: {:?}", self.kind)
    }
}

impl Error for SchedulerError {}

/// Adapter action emitted by one deterministic scheduler turn.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SchedulerAction {
    /// Ask the platform for one future frame-ready observation.
    RequestFrame,
    /// Offer immutable committed work to the renderer for atomic admission.
    OfferFrame(FrameWork),
    /// Stop renderer intake after all earlier accepted controls are processed.
    StopRenderer(ControlSequence),
}

/// Owned summary of one scheduler-controlled transaction attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScheduledCommit {
    generation: RuntimeGeneration,
    mutation_count: usize,
    invalidation: InvalidationSet,
}

impl ScheduledCommit {
    pub(super) const fn new(
        generation: RuntimeGeneration,
        mutation_count: usize,
        invalidation: InvalidationSet,
    ) -> Self {
        Self {
            generation,
            mutation_count,
            invalidation,
        }
    }

    /// Returns whether the transaction published no state.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.mutation_count == 0
    }

    /// Returns the committed generation after the attempt.
    #[must_use]
    pub const fn generation(self) -> RuntimeGeneration {
        self.generation
    }

    /// Returns the number of effective typed mutations.
    #[must_use]
    pub const fn mutation_count(self) -> usize {
        self.mutation_count
    }

    /// Returns the deterministic invalidation union.
    #[must_use]
    pub const fn invalidation(self) -> InvalidationSet {
        self.invalidation
    }
}
