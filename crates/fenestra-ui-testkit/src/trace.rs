use std::fmt;

use fenestra_ui_ir::prototype::InvalidationSet;

use crate::case::{GeneratedCaseV1, GeneratorConfigV1, OperationIdV1, SeedV1, TransactionIdV1};
use crate::error::HarnessError;
use crate::failure::ReplayFailureV1;
use crate::fixture::ReplayConfigV1;
use crate::identity::IdentitySummaryV1;
use crate::replay::ReplayReportV1;

const TRANSIENT_TRACE_BYTES_V1: usize = 262_144;

#[cfg(test)]
mod tests;
mod validate;

use validate::{derive_report_v1, termination_v1, validate_failure_v1};

pub(crate) fn encode_events_v1(
    events: &[TraceEventV1],
    max_bytes: usize,
) -> Result<Vec<u8>, HarnessError> {
    validate::encode_events_v1(events, max_bytes)
}

/// Closed candidate rejection taxonomy retained by logical trace V1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CandidateRejectionV1 {
    /// The staged-operation capacity was exceeded.
    CapacityOperations,
    /// The structural-change capacity was exceeded.
    CapacityStructural,
    /// The live-node capacity was exceeded.
    CapacityLiveNodes,
    /// The live-fragment capacity was exceeded.
    CapacityLiveFragments,
    /// The live-property capacity was exceeded.
    CapacityLiveProperties,
    /// The retained-generation capacity was exceeded.
    CapacityRetainedGenerations,
    /// The candidate no longer targeted the committed base.
    StaleBase,
    /// A target node was absent or stale.
    MissingNode,
    /// A target fragment was absent or stale.
    MissingFragment,
    /// A keyed member was absent.
    MissingKey,
    /// A keyed member already existed.
    DuplicateKey,
    /// A component did not declare the target property.
    UnknownProperty,
    /// A property value had the wrong closed type.
    PropertyTypeMismatch,
    /// A keyed destination index was outside its final range.
    IndexOutOfBounds,
    /// The logical runtime generation could not advance.
    GenerationExhausted,
    /// The candidate violated a runtime invariant.
    InvariantViolation,
}

impl CandidateRejectionV1 {
    /// Returns the canonical V1 rejection word.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::CapacityOperations => "capacity-operations",
            Self::CapacityStructural => "capacity-structural",
            Self::CapacityLiveNodes => "capacity-live-nodes",
            Self::CapacityLiveFragments => "capacity-live-fragments",
            Self::CapacityLiveProperties => "capacity-live-properties",
            Self::CapacityRetainedGenerations => "capacity-retained-generations",
            Self::StaleBase => "stale-base",
            Self::MissingNode => "missing-node",
            Self::MissingFragment => "missing-fragment",
            Self::MissingKey => "missing-key",
            Self::DuplicateKey => "duplicate-key",
            Self::UnknownProperty => "unknown-property",
            Self::PropertyTypeMismatch => "property-type-mismatch",
            Self::IndexOutOfBounds => "index-out-of-bounds",
            Self::GenerationExhausted => "generation-exhausted",
            Self::InvariantViolation => "invariant-violation",
        }
    }
}

/// Candidate outcome recorded for one logical transaction event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraceOutcomeV1 {
    /// A nonempty transaction published one generation.
    Commit,
    /// An accepted transaction produced no logical change.
    Noop,
    /// The candidate rejected the transaction with a closed reason.
    Reject(CandidateRejectionV1),
}

/// Oracle comparison result recorded for one logical transaction event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraceComparisonV1 {
    /// Clean reconstruction and candidate observation agreed.
    Match,
    /// The transaction produced the terminal oracle mismatch.
    Mismatch,
}

/// Closed test-only fault provenance retained by a logical trace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraceFaultV1 {
    /// Candidate fault that omits one authored move operation.
    OmitMove {
        /// Artifact-local operation selected by the fault.
        target: OperationIdV1,
    },
}

impl TraceFaultV1 {
    /// Returns the artifact-local operation selected by the fault.
    #[must_use]
    pub const fn target(self) -> OperationIdV1 {
        match self {
            Self::OmitMove { target } => target,
        }
    }
}

/// Physical-identity-free provenance for one exact logical trace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TraceProvenanceV1 {
    fixture_revision: u32,
    replay_config: ReplayConfigV1,
    generator_config: GeneratorConfigV1,
    seed: SeedV1,
    fault: Option<TraceFaultV1>,
}

impl TraceProvenanceV1 {
    const fn from_trace(
        case: &GeneratedCaseV1,
        replay_config: ReplayConfigV1,
        fault: Option<TraceFaultV1>,
    ) -> Self {
        Self {
            fixture_revision: case.fixture_revision(),
            replay_config,
            generator_config: case.config(),
            seed: case.seed(),
            fault,
        }
    }

    /// Returns the registered fixture revision.
    #[must_use]
    pub const fn fixture_revision(self) -> u32 {
        self.fixture_revision
    }

    /// Returns the exact candidate replay capacity.
    #[must_use]
    pub const fn replay_config(self) -> ReplayConfigV1 {
        self.replay_config
    }

    /// Returns the exact deterministic generator configuration.
    #[must_use]
    pub const fn generator_config(self) -> GeneratorConfigV1 {
        self.generator_config
    }

    /// Returns the exact deterministic generator seed.
    #[must_use]
    pub const fn seed(self) -> SeedV1 {
        self.seed
    }

    /// Returns closed injected-fault provenance when present.
    #[must_use]
    pub const fn fault(self) -> Option<TraceFaultV1> {
        self.fault
    }
}

/// Structural completion derived from a validated logical trace's final event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraceTerminationV1 {
    /// Every transaction in the exact case matched the clean oracle.
    Success,
    /// The candidate rejected the terminal transaction.
    Rejected(CandidateRejectionV1),
    /// The terminal committed or no-op transaction mismatched the oracle.
    Mismatch,
}

/// One physical-identity-free logical replay event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceEventV1 {
    sequence: u32,
    transaction: TransactionIdV1,
    operations: Vec<OperationIdV1>,
    before_generation: u64,
    after_generation: u64,
    outcome: TraceOutcomeV1,
    mutation_count: u32,
    invalidation: InvalidationSet,
    comparison: TraceComparisonV1,
}

impl TraceEventV1 {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        sequence: u32,
        transaction: TransactionIdV1,
        operations: Vec<OperationIdV1>,
        before_generation: u64,
        after_generation: u64,
        outcome: TraceOutcomeV1,
        mutation_count: usize,
        invalidation: InvalidationSet,
        comparison: TraceComparisonV1,
    ) -> Result<Self, HarnessError> {
        Ok(Self {
            sequence,
            transaction,
            operations,
            before_generation,
            after_generation,
            outcome,
            mutation_count: u32::try_from(mutation_count).map_err(|_| arithmetic_error())?,
            invalidation,
            comparison,
        })
    }

    /// Returns the dense zero-based event sequence.
    #[must_use]
    pub const fn sequence(&self) -> u32 {
        self.sequence
    }

    /// Returns the artifact-local transaction identifier.
    #[must_use]
    pub const fn transaction(&self) -> TransactionIdV1 {
        self.transaction
    }

    /// Returns operation identifiers in transaction order.
    #[must_use]
    pub fn operations(&self) -> &[OperationIdV1] {
        &self.operations
    }

    /// Returns the logical generation before the candidate attempt.
    #[must_use]
    pub const fn before_generation(&self) -> u64 {
        self.before_generation
    }

    /// Returns the logical generation after the candidate attempt.
    #[must_use]
    pub const fn after_generation(&self) -> u64 {
        self.after_generation
    }

    /// Returns the closed candidate outcome.
    #[must_use]
    pub const fn outcome(&self) -> TraceOutcomeV1 {
        self.outcome
    }

    /// Returns the observed mutation-record count.
    #[must_use]
    pub const fn mutation_count(&self) -> u32 {
        self.mutation_count
    }

    /// Returns the deterministic invalidation union.
    #[must_use]
    pub const fn invalidation(&self) -> InvalidationSet {
        self.invalidation
    }

    /// Returns the oracle comparison result.
    #[must_use]
    pub const fn comparison(&self) -> TraceComparisonV1 {
        self.comparison
    }
}

/// Bounded logical trace for one successful or terminal generated-case replay.
#[derive(Clone, Eq, PartialEq)]
pub struct LogicalTraceV1 {
    case: GeneratedCaseV1,
    replay_config: ReplayConfigV1,
    fault: Option<TraceFaultV1>,
    failure: Option<ReplayFailureV1>,
    report: ReplayReportV1,
    events: Vec<TraceEventV1>,
    canonical_bytes: Vec<u8>,
}

impl LogicalTraceV1 {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build(
        case: GeneratedCaseV1,
        replay_config: ReplayConfigV1,
        fault: Option<TraceFaultV1>,
        identity: IdentitySummaryV1,
        events: Vec<TraceEventV1>,
        failure: Option<ReplayFailureV1>,
        max_bytes: usize,
    ) -> Result<Self, HarnessError> {
        let report = derive_report_v1(&case, identity, &events)?;
        validate_failure_v1(&events, failure.as_ref())?;
        let canonical_bytes = encode_events_v1(&events, max_bytes.min(TRANSIENT_TRACE_BYTES_V1))?;
        Ok(Self {
            case,
            replay_config,
            fault,
            failure,
            report,
            events,
            canonical_bytes,
        })
    }

    /// Returns the exact generated case replayed by this trace.
    #[must_use]
    pub const fn case(&self) -> &GeneratedCaseV1 {
        &self.case
    }

    /// Returns the summary derived from validated logical events.
    #[must_use]
    pub const fn report(&self) -> ReplayReportV1 {
        self.report
    }

    /// Returns closed provenance derived from the retained case and replay input.
    #[must_use]
    pub const fn provenance(&self) -> TraceProvenanceV1 {
        TraceProvenanceV1::from_trace(&self.case, self.replay_config, self.fault)
    }

    /// Returns the first semantic candidate failure for a terminal trace.
    #[must_use]
    pub const fn failure(&self) -> Option<&ReplayFailureV1> {
        self.failure.as_ref()
    }

    /// Returns structural termination derived from the validated final event.
    #[must_use]
    pub fn termination(&self) -> TraceTerminationV1 {
        termination_v1(&self.events)
    }

    /// Returns logical events in dense sequence order.
    #[must_use]
    pub fn events(&self) -> &[TraceEventV1] {
        &self.events
    }

    /// Returns canonical event records with exactly one final LF.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

impl fmt::Debug for LogicalTraceV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LogicalTraceV1")
            .field("fixture_revision", &self.case.fixture_revision())
            .field("event_count", &self.events.len())
            .field("canonical_byte_count", &self.canonical_bytes.len())
            .finish()
    }
}

fn arithmetic_error() -> HarnessError {
    HarnessError::new(crate::error::HarnessErrorKind::ArithmeticExhausted)
}
