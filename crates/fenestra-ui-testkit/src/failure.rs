use crate::case::{OperationIdV1, TransactionIdV1};
use crate::fingerprint::FailureFingerprintV1;

/// First deterministic candidate failure retained by a terminal logical trace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayFailureV1 {
    transaction: TransactionIdV1,
    operation: Option<OperationIdV1>,
    fingerprint: FailureFingerprintV1,
}

impl ReplayFailureV1 {
    pub(crate) const fn new(
        transaction: TransactionIdV1,
        operation: Option<OperationIdV1>,
        fingerprint: FailureFingerprintV1,
    ) -> Self {
        Self {
            transaction,
            operation,
            fingerprint,
        }
    }

    /// Returns the transaction whose candidate first failed verification.
    #[must_use]
    pub const fn transaction(&self) -> TransactionIdV1 {
        self.transaction
    }

    /// Returns the authored operation correlated to a rejection, when present.
    #[must_use]
    pub const fn operation(&self) -> Option<OperationIdV1> {
        self.operation
    }

    /// Returns the physical-identity-free semantic failure fingerprint.
    #[must_use]
    pub const fn fingerprint(&self) -> &FailureFingerprintV1 {
        &self.fingerprint
    }
}
