use fenestra_ui_ir::prototype::{PropertyId, PropertyValue};

use crate::semantic::{FragmentPathV1, NodePathV1};

pub(crate) const REGISTERED_FIXTURE_REVISION_V1: u32 = 1;

/// Deterministic input to the V1 case generator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SeedV1(u64);

impl SeedV1 {
    /// Creates a deterministic generator seed. Zero is valid.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the numeric generator seed.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Bounded inputs controlling the shape of a generated V1 case.
///
/// Validation belongs to the generator so this value can also represent an
/// invalid configuration in typed error tests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GeneratorConfigV1 {
    transaction_count: u32,
    max_operations_per_transaction: u32,
    max_live_memberships: u32,
}

impl GeneratorConfigV1 {
    /// Creates a generator configuration without validating its limits.
    #[must_use]
    pub const fn new(
        transaction_count: u32,
        max_operations_per_transaction: u32,
        max_live_memberships: u32,
    ) -> Self {
        Self {
            transaction_count,
            max_operations_per_transaction,
            max_live_memberships,
        }
    }

    /// Returns the requested total transaction count.
    #[must_use]
    pub const fn transaction_count(self) -> u32 {
        self.transaction_count
    }

    /// Returns the maximum operations permitted in one transaction.
    #[must_use]
    pub const fn max_operations_per_transaction(self) -> u32 {
        self.max_operations_per_transaction
    }

    /// Returns the maximum live keyed memberships permitted in the case.
    #[must_use]
    pub const fn max_live_memberships(self) -> u32 {
        self.max_live_memberships
    }
}

/// Artifact-local identifier of one generated transaction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransactionIdV1(u32);

impl TransactionIdV1 {
    /// Creates a transaction identifier. Zero is valid.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the artifact-local numeric identifier.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Artifact-local identifier of one generated operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OperationIdV1(u32);

impl OperationIdV1 {
    /// Creates an operation identifier. Zero is valid.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the artifact-local numeric identifier.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Closed semantic operation grammar understood by the V1 replay harness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SemanticOperationV1 {
    /// Replaces one effective property on an existing semantic node.
    SetProperty {
        /// Stable semantic address of the target node.
        node: NodePathV1,
        /// Property symbol local to the target node's component.
        property: PropertyId,
        /// Replacement property value.
        value: PropertyValue,
    },
    /// Inserts a key into an existing semantic fragment.
    InsertKeyed {
        /// Stable semantic address of the target fragment.
        fragment: FragmentPathV1,
        /// Key to insert into the target fragment.
        key: u64,
        /// Requested index in the fragment's final keyed order.
        final_index: u32,
    },
    /// Moves an existing key within one semantic fragment.
    MoveKeyed {
        /// Stable semantic address of the target fragment.
        fragment: FragmentPathV1,
        /// Existing key to move within the target fragment.
        key: u64,
        /// Requested index in the fragment's final keyed order.
        final_index: u32,
    },
    /// Replaces one property on an existing keyed member root.
    UpdateKeyed {
        /// Stable semantic address of the target fragment.
        fragment: FragmentPathV1,
        /// Existing member key in the target fragment.
        key: u64,
        /// Property symbol local to the repeat body's component.
        property: PropertyId,
        /// Replacement property value.
        value: PropertyValue,
    },
    /// Removes an existing key from one semantic fragment.
    RemoveKeyed {
        /// Stable semantic address of the target fragment.
        fragment: FragmentPathV1,
        /// Existing key to remove from the target fragment.
        key: u64,
    },
}

/// One identified semantic operation in a generated case.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationV1 {
    id: OperationIdV1,
    operation: SemanticOperationV1,
}

impl OperationV1 {
    pub(crate) const fn new(id: OperationIdV1, operation: SemanticOperationV1) -> Self {
        Self { id, operation }
    }

    /// Returns the operation's stable artifact-local identifier.
    #[must_use]
    pub const fn id(&self) -> OperationIdV1 {
        self.id
    }

    /// Returns the semantic operation payload.
    #[must_use]
    pub const fn operation(&self) -> &SemanticOperationV1 {
        &self.operation
    }
}

/// One identified ordered transaction in a generated case.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionV1 {
    id: TransactionIdV1,
    operations: Vec<OperationV1>,
}

impl TransactionV1 {
    pub(crate) const fn new(id: TransactionIdV1, operations: Vec<OperationV1>) -> Self {
        Self { id, operations }
    }

    /// Returns the transaction's stable artifact-local identifier.
    #[must_use]
    pub const fn id(&self) -> TransactionIdV1 {
        self.id
    }

    /// Returns the semantic operations in authored execution order.
    #[must_use]
    pub fn operations(&self) -> &[OperationV1] {
        &self.operations
    }
}

/// Exact ordered transaction sequence emitted by the V1 generator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedCaseV1 {
    fixture_revision: u32,
    config: GeneratorConfigV1,
    seed: SeedV1,
    transactions: Vec<TransactionV1>,
}

impl GeneratedCaseV1 {
    pub(crate) const fn new(
        fixture_revision: u32,
        config: GeneratorConfigV1,
        seed: SeedV1,
        transactions: Vec<TransactionV1>,
    ) -> Self {
        Self {
            fixture_revision,
            config,
            seed,
            transactions,
        }
    }

    /// Returns the registered synthetic fixture revision.
    #[must_use]
    pub const fn fixture_revision(&self) -> u32 {
        self.fixture_revision
    }

    /// Returns the exact configuration used to generate this case.
    #[must_use]
    pub const fn config(&self) -> GeneratorConfigV1 {
        self.config
    }

    /// Returns the separate deterministic seed used to generate this case.
    #[must_use]
    pub const fn seed(&self) -> SeedV1 {
        self.seed
    }

    /// Returns transactions in generated execution order.
    #[must_use]
    pub fn transactions(&self) -> &[TransactionV1] {
        &self.transactions
    }

    /// Returns the total number of semantic operations in the case.
    #[must_use]
    pub fn operation_count(&self) -> usize {
        self.transactions
            .iter()
            .map(|transaction| transaction.operations.len())
            .sum()
    }
}
