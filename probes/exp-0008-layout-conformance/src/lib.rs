#![forbid(unsafe_code)]

//! Disposable EXP-0008 layout conformance probe boundary.

mod candidate;
mod corpus;
mod oracle;

#[cfg(test)]
#[path = "candidate/tests.rs"]
mod candidate_tests;

/// Reserved unstable surface for EXP-0008 conformance work.
#[doc(hidden)]
pub mod prototype {
    pub use crate::candidate::TaffyStackEngineV1;
    pub use crate::corpus::{RegisteredLayoutCaseV1, registered_layout_corpus_v1};
    pub use crate::oracle::{
        LayoutRecordMismatchKindV1, LayoutRecordMismatchV1, compare_layout_records_v1,
    };
}
