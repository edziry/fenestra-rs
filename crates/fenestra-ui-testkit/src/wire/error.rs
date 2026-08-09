use std::error::Error;
use std::fmt;

use crate::case::{OperationIdV1, TransactionIdV1};

/// Bounded artifact resources in deterministic V1 diagnostic order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactLimitKind {
    /// Bytes in one complete failure artifact.
    ArtifactBytes,
    /// Bytes in one line, excluding its line feed.
    LineBytes,
    /// Lines in one complete failure artifact.
    Lines,
    /// Bytes in one encoded semantic case.
    CaseBytes,
    /// Bytes in one embedded minimized trace.
    TraceBytes,
    /// Transactions in one semantic case.
    Transactions,
    /// Operations in one transaction.
    OperationsPerTransaction,
    /// Operations in one semantic case.
    Operations,
    /// Segments in one semantic node path.
    PathDepth,
    /// Events in one embedded minimized trace.
    TraceEvents,
    /// Replay attempts made by one reduction.
    ReductionEvaluations,
}

impl ArtifactLimitKind {
    /// All artifact limits in deterministic V1 tie-break order.
    pub const ALL: [Self; 11] = [
        Self::ArtifactBytes,
        Self::LineBytes,
        Self::Lines,
        Self::CaseBytes,
        Self::TraceBytes,
        Self::Transactions,
        Self::OperationsPerTransaction,
        Self::Operations,
        Self::PathDepth,
        Self::TraceEvents,
        Self::ReductionEvaluations,
    ];
}

/// Independently versioned portions of the V1 artifact contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VersionKind {
    /// Failure-envelope grammar.
    Envelope,
    /// Registered fixture.
    Fixture,
    /// Deterministic generator.
    Generator,
    /// Semantic case grammar.
    Case,
    /// Normalized-state schema.
    State,
    /// Logical-trace schema.
    Trace,
    /// Failure-fingerprint schema.
    Fingerprint,
    /// Reducer and reduction metric.
    Reducer,
}

/// Singleton sections in canonical V1 envelope order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SectionKind {
    /// Envelope header.
    Header,
    /// Version declarations.
    Versions,
    /// Registered fixture metadata.
    Fixture,
    /// Runtime replay capacity.
    Replay,
    /// Generator configuration.
    Generator,
    /// Generator seed.
    Seed,
    /// Original case.
    Original,
    /// Injected fault.
    Fault,
    /// Original failure.
    OriginalFailure,
    /// Reducer result.
    Reducer,
    /// Minimized case.
    Minimized,
    /// Minimized failure.
    MinimizedFailure,
    /// Minimized logical trace.
    Trace,
    /// Terminal marker.
    End,
}

/// Declared section counts checked by the V1 decoder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountKind {
    /// Transaction records.
    Transactions,
    /// Operation records.
    Operations,
    /// Encoded case bytes.
    CaseBytes,
    /// Operations declared by one transaction record.
    OperationsPerTransaction,
    /// Logical trace events.
    TraceEvents,
    /// Encoded trace bytes.
    TraceBytes,
}

/// Closed structural decode failures for V1 artifact data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactDecodeErrorKind {
    /// One inclusive artifact ceiling was crossed.
    LimitExceeded(ArtifactLimitKind),
    /// A byte was outside printable ASCII plus line feed.
    InvalidAscii,
    /// A record did not match the closed grammar.
    MalformedRecord,
    /// A declared schema version is unsupported.
    UnsupportedVersion(VersionKind),
    /// A primitive used a noncanonical representation.
    NonCanonicalValue,
    /// A required singleton section was absent.
    MissingSection(SectionKind),
    /// A singleton section occurred more than once.
    DuplicateSection(SectionKind),
    /// Canonical records occurred in an invalid order.
    OrderingViolation,
    /// A declared count differed from the corresponding records.
    CountMismatch(CountKind),
    /// An artifact-local reference did not resolve.
    InvalidReference,
    /// Fingerprint fields formed an illegal combination.
    InvalidFingerprint,
    /// Records followed the terminal marker.
    TrailingData,
}

/// Privacy-safe failure to decode caller-owned V1 bytes.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ArtifactDecodeError {
    kind: ArtifactDecodeErrorKind,
    line: Option<u32>,
}

impl ArtifactDecodeError {
    pub(crate) const fn new(kind: ArtifactDecodeErrorKind, line: Option<u32>) -> Self {
        Self { kind, line }
    }

    pub(crate) const fn at(kind: ArtifactDecodeErrorKind, line: u32) -> Self {
        Self::new(kind, Some(line))
    }

    /// Returns the closed failure class.
    #[must_use]
    pub const fn kind(self) -> ArtifactDecodeErrorKind {
        self.kind
    }

    /// Returns the one-based source line when one exists.
    #[must_use]
    pub const fn line(self) -> Option<u32> {
        self.line
    }
}

impl fmt::Debug for ArtifactDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ArtifactDecodeError")
            .field("kind", &self.kind)
            .field("line", &self.line)
            .finish()
    }
}

impl fmt::Display for ArtifactDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "runtime oracle artifact decode failed: {:?}",
            self.kind
        )
    }
}

impl Error for ArtifactDecodeError {}

/// Privacy-safe failure to encode bounded V1 data.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ArtifactEncodeError {
    limit: ArtifactLimitKind,
}

impl ArtifactEncodeError {
    pub(crate) const fn limit(limit: ArtifactLimitKind) -> Self {
        Self { limit }
    }

    /// Returns the crossed output ceiling.
    #[must_use]
    pub const fn kind(self) -> ArtifactLimitKind {
        self.limit
    }

    /// Returns the crossed output ceiling.
    #[must_use]
    pub const fn limit_kind(self) -> ArtifactLimitKind {
        self.limit
    }
}

impl fmt::Debug for ArtifactEncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ArtifactEncodeError")
            .field("limit", &self.limit)
            .finish()
    }
}

impl fmt::Display for ArtifactEncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "runtime oracle artifact encode limit crossed: {:?}",
            self.limit
        )
    }
}

impl Error for ArtifactEncodeError {}

/// Closed semantic verification failures for V1 failure artifacts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactVerificationErrorKind {
    /// Declared fixture identity or schema metadata did not match the registry.
    FixtureMismatch,
    /// Declared runtime replay capacity did not match the registered fixture.
    ReplayConfigMismatch,
    /// A semantic path did not resolve through the authored construction.
    InvalidSemanticPath,
    /// A semantic operation was invalid for its transaction base.
    InvalidSemanticOperation,
    /// Regeneration did not reproduce the exact original case records.
    SeedMismatch,
    /// Original replay did not reproduce its stored first failure.
    OriginalFailureMismatch,
    /// Minimized replay did not reproduce its stored first failure.
    MinimizedFailureMismatch,
    /// Original and minimized failures retained different fingerprints.
    FingerprintMismatch,
    /// Fresh minimized replay did not reproduce the stored logical events.
    TraceMismatch,
    /// The minimized case failed without the injected candidate fault.
    FaultFreeReplayFailed,
    /// Deterministic reduction did not reproduce the stored result.
    ReductionMismatch,
}

/// Privacy-safe failure to semantically verify one decoded V1 artifact.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ArtifactVerificationError {
    kind: ArtifactVerificationErrorKind,
    transaction: Option<TransactionIdV1>,
    operation: Option<OperationIdV1>,
}

impl ArtifactVerificationError {
    pub(crate) const fn new(kind: ArtifactVerificationErrorKind) -> Self {
        Self {
            kind,
            transaction: None,
            operation: None,
        }
    }

    pub(crate) const fn at_transaction(mut self, transaction: TransactionIdV1) -> Self {
        self.transaction = Some(transaction);
        self.operation = None;
        self
    }

    pub(crate) const fn at_operation(
        mut self,
        transaction: TransactionIdV1,
        operation: OperationIdV1,
    ) -> Self {
        self.transaction = Some(transaction);
        self.operation = Some(operation);
        self
    }

    /// Returns the closed semantic verification class.
    #[must_use]
    pub const fn kind(self) -> ArtifactVerificationErrorKind {
        self.kind
    }

    /// Returns the artifact-local transaction when one is available.
    #[must_use]
    pub const fn transaction(self) -> Option<TransactionIdV1> {
        self.transaction
    }

    /// Returns the artifact-local operation when one is available.
    #[must_use]
    pub const fn operation(self) -> Option<OperationIdV1> {
        self.operation
    }
}

impl fmt::Debug for ArtifactVerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ArtifactVerificationError")
            .field("kind", &self.kind)
            .field("transaction", &self.transaction)
            .field("operation", &self.operation)
            .finish()
    }
}

impl fmt::Display for ArtifactVerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "runtime oracle artifact verification failed: {:?}",
            self.kind
        )
    }
}

impl Error for ArtifactVerificationError {}
