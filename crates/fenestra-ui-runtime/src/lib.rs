#![forbid(unsafe_code)]

//! Experimental headless runtime kernel for Fenestra.
//!
//! Runtime behavior remains private until its owning feasibility work provides
//! executable evidence.

mod arena;
mod logical_tree;

/// Unstable cross-crate surface used only by unpublished feasibility probes.
#[doc(hidden)]
pub mod prototype {
    pub use crate::logical_tree::{LogicalTree, NodeId, TreeError, TreeInvariantError};
}
