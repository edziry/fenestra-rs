//! Structured semantic failure identities for deterministic oracle replay.

#[cfg(test)]
mod compare;
mod types;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use compare::compare_normalized_state_v1;
pub use types::{
    FailureFingerprintKindV1, FailureFingerprintV1, FingerprintFieldV1, FingerprintLocationV1,
    FingerprintSummaryV1,
};
