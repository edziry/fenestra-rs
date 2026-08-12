use std::error::Error;
use std::fmt;

use super::headless::HeadlessProjectionErrorKind;
use super::spatial::RuntimeSpatialErrorV2;

/// Bounded resource categories in deterministic diagnostic order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapacityKind {
    /// Staged operations in one transaction.
    Operations,
    /// Nodes and fragments created or retired in one transaction.
    StructuralChanges,
    /// Live logical nodes.
    LiveNodes,
    /// Live structural fragment instances.
    LiveFragments,
    /// Live typed property slots.
    LivePropertySlots,
    /// Distinct retired committed generations.
    RetainedGenerations,
}

impl CapacityKind {
    /// All capacity kinds in deterministic tie-break order.
    pub const ALL: [Self; 6] = [
        Self::Operations,
        Self::StructuralChanges,
        Self::LiveNodes,
        Self::LiveFragments,
        Self::LivePropertySlots,
        Self::RetainedGenerations,
    ];
}

/// Typed runtime initialization failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeInitializationErrorKind {
    /// An explicit runtime capacity would be exceeded.
    CapacityExceeded(CapacityKind),
    /// A provisional headless specification or projection failed.
    Headless(HeadlessProjectionErrorKind),
    /// Spatial wrapper validation or reference resolution failed.
    Spatial(RuntimeSpatialErrorV2),
    /// The materialized runtime state violated an internal invariant.
    InvariantViolation,
}

/// Failure to materialize the initial committed runtime generation.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RuntimeInitializationError {
    kind: RuntimeInitializationErrorKind,
}

impl RuntimeInitializationError {
    pub(crate) const fn new(kind: RuntimeInitializationErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the typed failure category.
    #[must_use]
    pub const fn kind(self) -> RuntimeInitializationErrorKind {
        self.kind
    }
}

impl fmt::Debug for RuntimeInitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeInitializationError")
            .field("kind", &self.kind)
            .finish()
    }
}

impl fmt::Display for RuntimeInitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "runtime initialization failed: {:?}", self.kind)
    }
}

impl Error for RuntimeInitializationError {}

/// Closed transaction failure taxonomy for the prototype.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionErrorKind {
    /// An explicit capacity would be exceeded.
    CapacityExceeded(CapacityKind),
    /// Provisional headless projection work failed.
    Headless(HeadlessProjectionErrorKind),
    /// Spatial wrapper validation or reference resolution failed.
    Spatial(RuntimeSpatialErrorV2),
    /// A headless-only operation targeted an ordinary runtime.
    HeadlessUnavailable,
    /// A spatial-only operation targeted an ordinary or headless runtime.
    SpatialUnavailable,
    /// The transaction no longer targets the exact committed base.
    StaleBase,
    /// A node identity is absent, foreign, or stale.
    MissingNode,
    /// A fragment identity is absent, foreign, or stale.
    MissingFragment,
    /// A key is absent from the selected fragment.
    MissingKey,
    /// A key already exists in the selected fragment.
    DuplicateKey,
    /// The selected component does not declare the property.
    UnknownProperty,
    /// The supplied value has the wrong closed property type.
    PropertyTypeMismatch,
    /// A keyed destination index is outside its valid final range.
    IndexOutOfBounds,
    /// The committed runtime generation cannot advance without wrapping.
    GenerationExhausted,
    /// The candidate state violated an internal invariant.
    InvariantViolation,
}

/// Failure to stage or atomically commit one transaction.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct TransactionError {
    kind: TransactionErrorKind,
    operation_index: Option<usize>,
}

impl TransactionError {
    pub(crate) const fn new(kind: TransactionErrorKind, operation_index: Option<usize>) -> Self {
        Self {
            kind,
            operation_index,
        }
    }

    /// Returns the typed failure category.
    #[must_use]
    pub const fn kind(self) -> TransactionErrorKind {
        self.kind
    }

    /// Returns the zero-based attempted operation when applicable.
    #[must_use]
    pub const fn operation_index(self) -> Option<usize> {
        self.operation_index
    }
}

impl fmt::Debug for TransactionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TransactionError")
            .field("kind", &self.kind)
            .field("operation_index", &self.operation_index)
            .finish()
    }
}

impl fmt::Display for TransactionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "runtime transaction failed: {:?}", self.kind)
    }
}

impl Error for TransactionError {}
