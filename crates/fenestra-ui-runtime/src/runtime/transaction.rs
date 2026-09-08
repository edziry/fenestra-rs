use std::convert::Infallible;
use std::fmt;
use std::sync::{Arc, Weak};

use fenestra_ui_ir::prototype::{
    InvalidationSet, PropertyId, PropertyValue, ValidatedConstruction,
};
use fenestra_ui_spatial::prototype::SpatialViewportV2;

use crate::logical_tree::NodeId;

use super::capacity::RuntimeCapacity;
use super::commit_control::CommitControl;
#[cfg(test)]
pub(super) use super::commit_control::CommitTestHook;
use super::error::{CapacityKind, TransactionError, TransactionErrorKind};
use super::fragment::FragmentId;
use super::headless::{HeadlessRuntimeConfig, HeadlessSurface};
use super::mutation::{MutationIter, MutationRecord};
use super::spatial::SpatialRuntimeConfig;
use super::state::{RuntimeGeneration, RuntimeState};
use super::view::CommittedRuntimeSnapshot;

mod commit;
mod construct;

pub use commit::CommitWithError;

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
    ResizeHeadless {
        surface: HeadlessSurface,
    },
    ResizeSpatial {
        viewport: SpatialViewportV2,
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
    pub(super) fn operation_count(&self) -> usize {
        self.operations.len()
    }

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

    /// Stages one provisional headless surface extent change.
    pub fn resize_headless(&mut self, surface: HeadlessSurface) -> Result<(), TransactionError> {
        self.stage(Operation::ResizeHeadless { surface })
    }

    /// Stages one spatial viewport extent change.
    pub fn resize_spatial(&mut self, viewport: SpatialViewportV2) -> Result<(), TransactionError> {
        self.stage(Operation::ResizeSpatial { viewport })
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
    pub(super) headless: Option<HeadlessRuntimeConfig>,
    pub(super) spatial: Option<SpatialRuntimeConfig>,
    state: Arc<RuntimeState>,
    retired: Vec<Weak<RuntimeState>>,
}

impl UiRuntime {
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

    /// Resolves an immutable candidate without publishing or reserving identities.
    ///
    /// Uses the same validation, projection work, and publication guards as
    /// `commit`. A no-op shares the current snapshot; an effective transaction
    /// yields a separate snapshot with the next generation. Its generation and
    /// newly created identities are provisional and reserve no future publication.
    /// Reconstruct a transaction against the current base to commit afterward.
    ///
    /// Leaves accepted state, identity allocators, and retention bookkeeping
    /// unchanged. Effective previews are independent of retired-generation
    /// accounting; a no-op retains the current state exactly as `committed()`.
    /// Projection implementations may still update their own caches.
    pub fn preview(
        &self,
        transaction: UiTransaction,
    ) -> Result<CommittedRuntimeSnapshot, TransactionError> {
        self.preview_inner(transaction)
    }

    /// Atomically commits all staged operations or preserves the prior state.
    pub fn commit(
        &mut self,
        transaction: UiTransaction,
    ) -> Result<CommitReceipt, TransactionError> {
        self.commit_inner(transaction, CommitControl::NONE, |_| {
            Ok::<(), Infallible>(())
        })
        .map(|(receipt, ())| receipt)
        .map_err(CommitWithError::into_runtime)
    }

    /// Prepares an owned sidecar from the exact candidate before publication.
    ///
    /// The callback runs once after runtime validation and publication guards.
    /// A no-op transaction supplies the current snapshot without advancing its
    /// generation. Otherwise the supplied snapshot has the next generation and
    /// its completed projections, but becomes committed only when this returns
    /// successfully. Rejection or unwind preserves the previous runtime state.
    /// Callbacks must keep external changes provisional until success.
    pub fn commit_with<T, E>(
        &mut self,
        transaction: UiTransaction,
        prepare: impl FnOnce(&CommittedRuntimeSnapshot) -> Result<T, E>,
    ) -> Result<(CommitReceipt, T), CommitWithError<E>> {
        self.commit_inner(transaction, CommitControl::NONE, prepare)
    }

    #[cfg(test)]
    pub(super) fn commit_with_test_hook(
        &mut self,
        transaction: UiTransaction,
        hook: CommitTestHook,
    ) -> Result<CommitReceipt, TransactionError> {
        self.commit_inner(transaction, hook.control(), |_| Ok::<(), Infallible>(()))
            .map(|(receipt, ())| receipt)
            .map_err(CommitWithError::into_runtime)
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
