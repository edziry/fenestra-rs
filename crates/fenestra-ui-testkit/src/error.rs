use std::error::Error;
use std::fmt;

use crate::case::{OperationIdV1, TransactionIdV1};

/// Bounded resource classes enforced by the V1 runtime-oracle harness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HarnessLimitKind {
    /// Transactions in one generated case.
    Transactions,
    /// Operations in one transaction.
    OperationsPerTransaction,
    /// Operations in one generated case.
    Operations,
    /// Live keyed memberships in the complete desired state.
    LiveMemberships,
    /// Segments in one semantic node path.
    PathDepth,
    /// Nodes in one normalized state.
    NormalizedNodes,
    /// Fragments in one normalized state.
    NormalizedFragments,
    /// Property slots in one normalized state.
    NormalizedProperties,
    /// Applicable actions at one generator choice.
    ApplicableActions,
    /// Bytes in one transient logical trace.
    TraceBytes,
}

impl HarnessLimitKind {
    /// Deterministic tie-break order when several harness limits are crossed.
    pub const ALL: [Self; 10] = [
        Self::Transactions,
        Self::OperationsPerTransaction,
        Self::Operations,
        Self::LiveMemberships,
        Self::PathDepth,
        Self::NormalizedNodes,
        Self::NormalizedFragments,
        Self::NormalizedProperties,
        Self::ApplicableActions,
        Self::TraceBytes,
    ];
}

/// Closed failure classes produced by the V1 runtime-oracle harness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HarnessErrorKind {
    /// A fixture, generator, trace, or artifact version is unsupported.
    UnsupportedVersion,
    /// A caller-supplied configuration violates the V1 contract.
    InvalidConfiguration,
    /// A bounded harness resource crossed its inclusive ceiling.
    LimitExceeded(HarnessLimitKind),
    /// The registered synthetic IR fixture did not validate.
    FixtureValidation,
    /// A semantic node or fragment path does not resolve as authored.
    InvalidSemanticPath,
    /// A semantic operation is invalid for its transaction base.
    InvalidOperation,
    /// The candidate runtime could not initialize the registered fixture.
    RuntimeInitialization,
    /// A generated operation was rejected by the candidate runtime.
    UnexpectedCandidateRejection,
    /// Clean reconstruction and candidate observation differ.
    StateMismatch,
    /// A runtime identity violated its semantic lifecycle contract.
    IdentityMismatch,
    /// Logical trace generation or replay differs.
    TraceMismatch,
    /// Checked harness arithmetic could not represent a required value.
    ArithmeticExhausted,
}

/// Privacy-safe error returned by the V1 runtime-oracle harness.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct HarnessError {
    kind: HarnessErrorKind,
    transaction: Option<TransactionIdV1>,
    operation: Option<OperationIdV1>,
}

impl HarnessError {
    pub(crate) const fn new(kind: HarnessErrorKind) -> Self {
        Self {
            kind,
            transaction: None,
            operation: None,
        }
    }

    pub(crate) const fn limit(kind: HarnessLimitKind) -> Self {
        Self::new(HarnessErrorKind::LimitExceeded(kind))
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

    /// Returns the closed failure class without private runtime data.
    #[must_use]
    pub const fn kind(self) -> HarnessErrorKind {
        self.kind
    }

    /// Returns the semantic transaction location when one exists.
    #[must_use]
    pub const fn transaction(self) -> Option<TransactionIdV1> {
        self.transaction
    }

    /// Returns the semantic operation location when one exists.
    #[must_use]
    pub const fn operation(self) -> Option<OperationIdV1> {
        self.operation
    }
}

impl fmt::Debug for HarnessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HarnessError")
            .field("kind", &self.kind)
            .field("transaction", &self.transaction)
            .field("operation", &self.operation)
            .finish()
    }
}

impl fmt::Display for HarnessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "runtime oracle harness failed: {:?}", self.kind)
    }
}

impl Error for HarnessError {}

/// Closed failures produced while constructing a deterministic V1 case.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeneratorErrorKind {
    /// A configuration is too small to contain the directed prefix.
    InvalidConfig,
    /// A configuration or generated case crossed a harness ceiling.
    LimitExceeded(HarnessLimitKind),
    /// No valid semantic action was available for a generated choice.
    NoApplicableAction,
    /// Checked generator arithmetic could not represent a required value.
    ArithmeticExhausted,
}

/// Privacy-safe error returned by deterministic V1 case generation.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct GeneratorError {
    kind: GeneratorErrorKind,
}

impl GeneratorError {
    pub(crate) const fn new(kind: GeneratorErrorKind) -> Self {
        Self { kind }
    }

    pub(crate) const fn limit(kind: HarnessLimitKind) -> Self {
        Self::new(GeneratorErrorKind::LimitExceeded(kind))
    }

    /// Returns the closed generator failure class.
    #[must_use]
    pub const fn kind(self) -> GeneratorErrorKind {
        self.kind
    }
}

impl fmt::Debug for GeneratorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GeneratorError")
            .field("kind", &self.kind)
            .finish()
    }
}

impl fmt::Display for GeneratorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "runtime oracle generation failed: {:?}",
            self.kind
        )
    }
}

impl Error for GeneratorError {}
