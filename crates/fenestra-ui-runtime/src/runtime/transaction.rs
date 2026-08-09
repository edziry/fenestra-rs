use std::fmt;
use std::sync::{Arc, Weak};

use fenestra_ui_ir::prototype::{
    InvalidationSet, PropertyId, PropertyValue, ValidatedConstruction,
};

use crate::logical_tree::NodeId;

use super::capacity::RuntimeCapacity;
#[cfg(test)]
pub(super) use super::commit_control::CommitTestHook;
use super::commit_control::{CommitCheckpoint, CommitControl};
use super::error::{
    CapacityKind, RuntimeInitializationError, TransactionError, TransactionErrorKind,
};
use super::fragment::FragmentId;
use super::mutation::{MutationIter, MutationRecord};
use super::state::{RuntimeGeneration, RuntimeState};
use super::view::CommittedRuntimeSnapshot;

pub(super) enum Operation {
    SetProperty {
        node: NodeId,
        property: PropertyId,
        value: PropertyValue,
    },
    InsertKeyed {
        fragment: FragmentId,
        key: u64,
        final_index: usize,
    },
    MoveKeyed {
        fragment: FragmentId,
        key: u64,
        final_index: usize,
    },
    UpdateKeyed {
        fragment: FragmentId,
        key: u64,
        property: PropertyId,
        value: PropertyValue,
    },
    RemoveKeyed {
        fragment: FragmentId,
        key: u64,
    },
}

/// Detached bounded mutation plan targeting one exact committed state.
pub struct UiTransaction {
    base: Arc<RuntimeState>,
    operations: Vec<Operation>,
    operation_limit: usize,
    poison: Option<TransactionError>,
}

impl UiTransaction {
    /// Stages one typed direct property update.
    pub fn set_property(
        &mut self,
        node: NodeId,
        property: PropertyId,
        value: PropertyValue,
    ) -> Result<(), TransactionError> {
        self.stage(Operation::SetProperty {
            node,
            property,
            value,
        })
    }

    /// Stages creation of one keyed member at a final local index.
    pub fn insert_keyed(
        &mut self,
        fragment: FragmentId,
        key: u64,
        final_index: usize,
    ) -> Result<(), TransactionError> {
        self.stage(Operation::InsertKeyed {
            fragment,
            key,
            final_index,
        })
    }

    /// Stages a keyed member move to a final local index.
    pub fn move_keyed(
        &mut self,
        fragment: FragmentId,
        key: u64,
        final_index: usize,
    ) -> Result<(), TransactionError> {
        self.stage(Operation::MoveKeyed {
            fragment,
            key,
            final_index,
        })
    }

    /// Stages a typed update on one keyed member root.
    pub fn update_keyed(
        &mut self,
        fragment: FragmentId,
        key: u64,
        property: PropertyId,
        value: PropertyValue,
    ) -> Result<(), TransactionError> {
        self.stage(Operation::UpdateKeyed {
            fragment,
            key,
            property,
            value,
        })
    }

    /// Stages retirement of one keyed member subtree.
    pub fn remove_keyed(&mut self, fragment: FragmentId, key: u64) -> Result<(), TransactionError> {
        self.stage(Operation::RemoveKeyed { fragment, key })
    }

    fn stage(&mut self, operation: Operation) -> Result<(), TransactionError> {
        if let Some(error) = self.poison {
            return Err(error);
        }
        if self.operations.len() >= self.operation_limit {
            let error = TransactionError::new(
                TransactionErrorKind::CapacityExceeded(CapacityKind::Operations),
                Some(self.operations.len()),
            );
            self.poison = Some(error);
            return Err(error);
        }
        self.operations.push(operation);
        Ok(())
    }
}

/// Owner of the latest committed logical runtime generation.
pub struct UiRuntime {
    pub(super) construction: ValidatedConstruction,
    pub(super) capacity: RuntimeCapacity,
    state: Arc<RuntimeState>,
    retired: Vec<Weak<RuntimeState>>,
}

impl UiRuntime {
    /// Materializes generation zero from one exact validated construction.
    pub fn new(
        construction: ValidatedConstruction,
        capacity: RuntimeCapacity,
    ) -> Result<Self, RuntimeInitializationError> {
        let state = RuntimeState::initialize(&construction, capacity)?;
        Ok(Self {
            construction,
            capacity,
            state: Arc::new(state),
            retired: Vec::new(),
        })
    }

    /// Returns an immutable handle to the current committed state.
    #[must_use]
    pub fn committed(&self) -> CommittedRuntimeSnapshot {
        CommittedRuntimeSnapshot {
            state: Arc::clone(&self.state),
        }
    }

    /// Begins a detached transaction against the exact current state.
    #[must_use]
    pub fn begin_transaction(&self) -> UiTransaction {
        UiTransaction {
            base: Arc::clone(&self.state),
            operations: Vec::new(),
            operation_limit: self.capacity.operations(),
            poison: None,
        }
    }

    /// Atomically commits all staged operations or preserves the prior state.
    pub fn commit(
        &mut self,
        transaction: UiTransaction,
    ) -> Result<CommitReceipt, TransactionError> {
        self.commit_inner(transaction, CommitControl::NONE)
    }

    fn commit_inner(
        &mut self,
        transaction: UiTransaction,
        control: CommitControl,
    ) -> Result<CommitReceipt, TransactionError> {
        if let Some(error) = transaction.poison {
            return Err(error);
        }
        if !Arc::ptr_eq(&self.state, &transaction.base) {
            return Err(TransactionError::new(TransactionErrorKind::StaleBase, None));
        }

        let mut draft = transaction.base.fork_for_transaction();
        control.panic_if(CommitCheckpoint::Draft);
        let mut records = self.apply_operations(&mut draft, transaction.operations)?;
        control.panic_if(CommitCheckpoint::Apply);
        control.before_validation(&mut draft);
        draft
            .validate(&self.construction, self.capacity)
            .map_err(|()| TransactionError::new(TransactionErrorKind::InvariantViolation, None))?;
        control.panic_if(CommitCheckpoint::Validation);
        records.retain(MutationRecord::is_effective);
        let invalidation = records.iter().fold(InvalidationSet::NONE, |set, record| {
            set.union(record.invalidation())
        });
        if records.is_empty() {
            return Ok(CommitReceipt {
                generation: self.state.generation,
                records,
                invalidation,
                _retired_generation: None,
            });
        }

        self.retired.retain(|state| state.strong_count() != 0);
        let retained = self.retired.len().checked_add(1).ok_or_else(|| {
            TransactionError::new(
                TransactionErrorKind::CapacityExceeded(CapacityKind::RetainedGenerations),
                None,
            )
        })?;
        if retained > self.capacity.retained_generations() {
            return Err(TransactionError::new(
                TransactionErrorKind::CapacityExceeded(CapacityKind::RetainedGenerations),
                None,
            ));
        }
        let generation = self.state.generation.next().ok_or_else(|| {
            TransactionError::new(TransactionErrorKind::GenerationExhausted, None)
        })?;
        draft.generation = generation;
        let prepared = Arc::new(draft);
        self.retired.reserve(1);
        control.panic_if(CommitCheckpoint::Preparation);

        let previous = std::mem::replace(&mut self.state, prepared);
        self.retired.push(Arc::downgrade(&previous));
        Ok(CommitReceipt {
            generation,
            records,
            invalidation,
            _retired_generation: Some(previous),
        })
    }

    #[cfg(test)]
    pub(super) fn commit_with_test_hook(
        &mut self,
        transaction: UiTransaction,
        hook: CommitTestHook,
    ) -> Result<CommitReceipt, TransactionError> {
        self.commit_inner(transaction, hook.control())
    }

    #[cfg(test)]
    pub(super) fn set_generation_for_test(&mut self, value: u64) {
        Arc::get_mut(&mut self.state)
            .expect("test generation mutation requires no retained current snapshot")
            .set_generation_for_test(value);
    }
}

/// Immutable result of one successful commit attempt.
pub struct CommitReceipt {
    generation: RuntimeGeneration,
    records: Vec<MutationRecord>,
    invalidation: InvalidationSet,
    _retired_generation: Option<Arc<RuntimeState>>,
}

impl CommitReceipt {
    /// Returns the generation observed after the commit attempt.
    #[must_use]
    pub const fn generation(&self) -> RuntimeGeneration {
        self.generation
    }

    /// Returns whether no state publication occurred.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Iterates the ordered typed mutation log.
    pub fn mutations(&self) -> MutationIter<'_> {
        MutationIter::new(&self.records)
    }

    /// Returns the deterministic union of retained mutation causes.
    #[must_use]
    pub const fn invalidation(&self) -> InvalidationSet {
        self.invalidation
    }
}

impl fmt::Debug for CommitReceipt {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CommitReceipt")
            .field("generation", &self.generation)
            .field("mutation_count", &self.records.len())
            .field("invalidation", &self.invalidation)
            .finish()
    }
}
