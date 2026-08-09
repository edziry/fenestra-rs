use super::*;
use crate::trace::CandidateRejectionV1;

#[test]
fn candidate_rejection_constructor_uses_closed_global_shape() {
    let fingerprint = FailureFingerprintV1::candidate_rejected(CandidateRejectionV1::MissingKey);

    assert_eq!(
        fingerprint.kind(),
        FailureFingerprintKindV1::CandidateRejected
    );
    assert_eq!(fingerprint.location(), &FingerprintLocationV1::Global);
    assert_eq!(fingerprint.field(), FingerprintFieldV1::CandidateOutcome);
    assert_eq!(
        fingerprint.expected(),
        &FingerprintSummaryV1::CandidateAccepted
    );
    assert_eq!(
        fingerprint.observed(),
        &FingerprintSummaryV1::CandidateRejected(CandidateRejectionV1::MissingKey)
    );
}

#[test]
fn identity_fingerprint_preserves_expected_before_observed() {
    let path = NodePathV1::root().member(1, 7);
    let fingerprint = FailureFingerprintV1::from_parts(
        FailureFingerprintKindV1::IdentityMismatch,
        FingerprintLocationV1::Node(path.clone()),
        FingerprintFieldV1::IdentityLifecycle,
        FingerprintSummaryV1::LifecycleDistinct,
        FingerprintSummaryV1::LifecycleAliased,
    )
    .expect("legal identity fingerprint should build");

    assert_eq!(
        fingerprint.kind(),
        FailureFingerprintKindV1::IdentityMismatch
    );
    assert_eq!(fingerprint.location(), &FingerprintLocationV1::Node(path));
    assert_eq!(fingerprint.field(), FingerprintFieldV1::IdentityLifecycle);
    assert_eq!(
        fingerprint.expected(),
        &FingerprintSummaryV1::LifecycleDistinct
    );
    assert_eq!(
        fingerprint.observed(),
        &FingerprintSummaryV1::LifecycleAliased
    );
}

#[test]
fn identity_constructor_accepts_node_and_fragment_locations() {
    let node = NodePathV1::root().member(1, 7);
    let fragment = FragmentPathV1::new(node.clone(), 1);

    for location in [
        FingerprintLocationV1::Node(node),
        FingerprintLocationV1::Fragment(fragment),
    ] {
        let fingerprint = FailureFingerprintV1::identity_mismatch(
            location.clone(),
            FingerprintSummaryV1::LifecyclePreserved,
            FingerprintSummaryV1::LifecycleFresh,
        )
        .expect("legal identity mismatch should build");

        assert_eq!(
            fingerprint.kind(),
            FailureFingerprintKindV1::IdentityMismatch
        );
        assert_eq!(fingerprint.location(), &location);
        assert_eq!(fingerprint.field(), FingerprintFieldV1::IdentityLifecycle);
        assert_eq!(
            fingerprint.expected(),
            &FingerprintSummaryV1::LifecyclePreserved
        );
        assert_eq!(
            fingerprint.observed(),
            &FingerprintSummaryV1::LifecycleFresh
        );
    }
}

#[test]
fn fingerprint_parts_reject_equal_or_illegal_combinations() {
    let equal = FailureFingerprintV1::from_parts(
        FailureFingerprintKindV1::StateMismatch,
        FingerprintLocationV1::Global,
        FingerprintFieldV1::NodeCount,
        FingerprintSummaryV1::Count(1),
        FingerprintSummaryV1::Count(1),
    );
    let illegal = FailureFingerprintV1::from_parts(
        FailureFingerprintKindV1::CandidateRejected,
        FingerprintLocationV1::Node(NodePathV1::root()),
        FingerprintFieldV1::CandidateOutcome,
        FingerprintSummaryV1::CandidateAccepted,
        FingerprintSummaryV1::CandidateRejected(CandidateRejectionV1::MissingKey),
    );

    assert!(equal.is_none());
    assert!(illegal.is_none());
}
