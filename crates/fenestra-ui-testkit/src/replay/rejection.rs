use fenestra_ui_runtime::prototype::{CapacityKind, TransactionError, TransactionErrorKind};

use crate::case::{OperationIdV1, TransactionIdV1};
use crate::error::{HarnessError, HarnessErrorKind};
use crate::failure::ReplayFailureV1;
use crate::fingerprint::FailureFingerprintV1;
use crate::trace::CandidateRejectionV1;

pub(super) fn candidate_rejection_v1(
    error: TransactionError,
    transaction: TransactionIdV1,
    candidate_ids: &[OperationIdV1],
) -> Result<(CandidateRejectionV1, ReplayFailureV1), HarnessError> {
    let rejection = map_kind(error.kind());
    let operation = match error.operation_index() {
        None => None,
        Some(index) => Some(candidate_ids.get(index).copied().ok_or_else(trace_error)?),
    };
    let fingerprint = FailureFingerprintV1::candidate_rejected(rejection);
    Ok((
        rejection,
        ReplayFailureV1::new(transaction, operation, fingerprint),
    ))
}

const fn map_kind(kind: TransactionErrorKind) -> CandidateRejectionV1 {
    match kind {
        TransactionErrorKind::CapacityExceeded(capacity) => map_capacity(capacity),
        TransactionErrorKind::Headless(_) => CandidateRejectionV1::InvariantViolation,
        TransactionErrorKind::HeadlessUnavailable => CandidateRejectionV1::InvariantViolation,
        TransactionErrorKind::StaleBase => CandidateRejectionV1::StaleBase,
        TransactionErrorKind::MissingNode => CandidateRejectionV1::MissingNode,
        TransactionErrorKind::MissingFragment => CandidateRejectionV1::MissingFragment,
        TransactionErrorKind::MissingKey => CandidateRejectionV1::MissingKey,
        TransactionErrorKind::DuplicateKey => CandidateRejectionV1::DuplicateKey,
        TransactionErrorKind::UnknownProperty => CandidateRejectionV1::UnknownProperty,
        TransactionErrorKind::PropertyTypeMismatch => CandidateRejectionV1::PropertyTypeMismatch,
        TransactionErrorKind::IndexOutOfBounds => CandidateRejectionV1::IndexOutOfBounds,
        TransactionErrorKind::GenerationExhausted => CandidateRejectionV1::GenerationExhausted,
        TransactionErrorKind::InvariantViolation => CandidateRejectionV1::InvariantViolation,
    }
}

const fn map_capacity(capacity: CapacityKind) -> CandidateRejectionV1 {
    match capacity {
        CapacityKind::Operations => CandidateRejectionV1::CapacityOperations,
        CapacityKind::StructuralChanges => CandidateRejectionV1::CapacityStructural,
        CapacityKind::LiveNodes => CandidateRejectionV1::CapacityLiveNodes,
        CapacityKind::LiveFragments => CandidateRejectionV1::CapacityLiveFragments,
        CapacityKind::LivePropertySlots => CandidateRejectionV1::CapacityLiveProperties,
        CapacityKind::RetainedGenerations => CandidateRejectionV1::CapacityRetainedGenerations,
    }
}

fn trace_error() -> HarnessError {
    HarnessError::new(HarnessErrorKind::TraceMismatch)
}
