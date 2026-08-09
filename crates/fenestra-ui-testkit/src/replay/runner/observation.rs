use crate::case::TransactionIdV1;
use crate::error::{HarnessError, HarnessErrorKind};
use crate::fingerprint::{FailureFingerprintKindV1, FailureFingerprintV1};
use crate::observe::{ObservationOutcomeV1, ObservedSnapshotV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::replay) enum ObservationPhaseV1 {
    Initial,
    Before(TransactionIdV1),
    AfterCommit(TransactionIdV1),
}

pub(super) fn require_complete_observation_v1(
    outcome: ObservationOutcomeV1,
) -> Result<ObservedSnapshotV1, HarnessError> {
    match outcome {
        ObservationOutcomeV1::Complete(observed) => Ok(observed),
        ObservationOutcomeV1::Mismatch(fingerprint) => Err(observation_error_v1(&fingerprint)),
    }
}

pub(super) fn observation_error_v1(fingerprint: &FailureFingerprintV1) -> HarnessError {
    let kind = match fingerprint.kind() {
        FailureFingerprintKindV1::CandidateRejected => {
            HarnessErrorKind::UnexpectedCandidateRejection
        }
        FailureFingerprintKindV1::StateMismatch => HarnessErrorKind::StateMismatch,
        FailureFingerprintKindV1::IdentityMismatch => HarnessErrorKind::IdentityMismatch,
    };
    HarnessError::new(kind)
}
