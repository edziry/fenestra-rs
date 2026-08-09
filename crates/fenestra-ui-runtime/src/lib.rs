#![forbid(unsafe_code)]

//! Experimental headless runtime kernel for Fenestra.
//!
//! Runtime behavior remains private until its owning feasibility work provides
//! executable evidence.

mod arena;
mod logical_tree;
mod runtime;

/// Unstable cross-crate surface used only by unpublished feasibility probes.
#[doc(hidden)]
pub mod prototype {
    pub use crate::logical_tree::{LogicalTree, NodeId, TreeError, TreeInvariantError};
    pub use crate::runtime::{
        CapacityKind, CommitReceipt, CommittedRuntimeSnapshot, FragmentId, KeyInsertView,
        KeyMoveView, KeyRemoveView, KeyedMemberIter, ManifestEntry, ManifestIter, MutationIter,
        MutationRecordView, PropertyChangeView, RuntimeCapacity, RuntimeGeneration,
        RuntimeInitializationError, RuntimeInitializationErrorKind, TransactionError,
        TransactionErrorKind, UiRuntime, UiTransaction,
    };
}
