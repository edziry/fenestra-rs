use fenestra_ui_testkit::prototype::{
    ArtifactVerificationError, ArtifactVerificationErrorKind, FailureArtifactV1, OperationIdV1,
    TransactionIdV1, decode_failure_artifact_v1, verify_failure_artifact_v1,
};

const CANONICAL: &[u8] = include_bytes!("fixtures/canonical_structural_failure_v1.txt");

#[test]
fn replay_trace_and_reduction_failures_use_their_closed_classes() {
    assert_verification_error(
        original_failure_mismatch_bytes(),
        ArtifactVerificationErrorKind::OriginalFailureMismatch,
        Some(TransactionIdV1::new(2)),
        Some(OperationIdV1::new(4)),
    );
    assert_verification_error(
        minimized_failure_mismatch_bytes(),
        ArtifactVerificationErrorKind::MinimizedFailureMismatch,
        Some(TransactionIdV1::new(2)),
        None,
    );
    assert_verification_error(
        fingerprint_mismatch_bytes(),
        ArtifactVerificationErrorKind::FingerprintMismatch,
        Some(TransactionIdV1::new(2)),
        None,
    );
    assert_verification_error(
        trace_mismatch_bytes(),
        ArtifactVerificationErrorKind::TraceMismatch,
        Some(TransactionIdV1::new(2)),
        None,
    );
    assert_verification_error(
        reduction_mismatch_bytes(),
        ArtifactVerificationErrorKind::ReductionMismatch,
        None,
        None,
    );
}

#[test]
fn replay_trace_and_reduction_priority_is_stable() {
    assert_verification_kind(
        minimized_failure_mismatch_on(original_failure_mismatch_bytes()),
        ArtifactVerificationErrorKind::OriginalFailureMismatch,
    );
    assert_verification_kind(
        trace_mismatch_on(minimized_failure_mismatch_bytes()),
        ArtifactVerificationErrorKind::MinimizedFailureMismatch,
    );
    assert_verification_kind(
        trace_mismatch_on(fingerprint_mismatch_bytes()),
        ArtifactVerificationErrorKind::FingerprintMismatch,
    );
    assert_verification_kind(
        reduction_mismatch_on(trace_mismatch_bytes()),
        ArtifactVerificationErrorKind::TraceMismatch,
    );
}

#[test]
fn verification_error_kind_is_closed_in_v1_priority_order() {
    let order = [
        ArtifactVerificationErrorKind::FixtureMismatch,
        ArtifactVerificationErrorKind::ReplayConfigMismatch,
        ArtifactVerificationErrorKind::InvalidSemanticPath,
        ArtifactVerificationErrorKind::InvalidSemanticOperation,
        ArtifactVerificationErrorKind::SeedMismatch,
        ArtifactVerificationErrorKind::OriginalFailureMismatch,
        ArtifactVerificationErrorKind::MinimizedFailureMismatch,
        ArtifactVerificationErrorKind::FingerprintMismatch,
        ArtifactVerificationErrorKind::TraceMismatch,
        ArtifactVerificationErrorKind::FaultFreeReplayFailed,
        ArtifactVerificationErrorKind::ReductionMismatch,
    ];

    for (expected, kind) in order.into_iter().enumerate() {
        assert_eq!(verification_kind_rank(kind), expected);
    }
}

fn original_failure_mismatch_bytes() -> Vec<u8> {
    let bytes = replace_once(
        verified_artifact_bytes(),
        "failure|original|2|-|state-mismatch|fragment:root/r:1|keyed-order|keys:9,7,8|keys:7,8,9",
        "failure|original|2|4|candidate-rejected|global|candidate-outcome|kind:accept|kind:missing-key",
    );
    replace_once(
        bytes,
        "failure|minimized|2|-|state-mismatch|fragment:root/r:1|keyed-order|keys:9,7,8|keys:7,8,9",
        "failure|minimized|2|4|candidate-rejected|global|candidate-outcome|kind:accept|kind:missing-key",
    )
}

fn minimized_failure_mismatch_bytes() -> Vec<u8> {
    minimized_failure_mismatch_on(verified_artifact_bytes())
}

fn minimized_failure_mismatch_on(bytes: Vec<u8>) -> Vec<u8> {
    replace_once(
        bytes,
        "op|4|move|root/r:1|9|0\nminimized-end",
        "op|4|move|root/r:1|9|2\nminimized-end",
    )
}

fn fingerprint_mismatch_bytes() -> Vec<u8> {
    let bytes = replace_once(
        verified_artifact_bytes(),
        "op|4|move|root/r:1|9|0\nminimized-end",
        "op|4|move|root/r:1|9|1\nminimized-end",
    );
    replace_once(
        bytes,
        "failure|minimized|2|-|state-mismatch|fragment:root/r:1|keyed-order|keys:9,7,8|keys:7,8,9",
        "failure|minimized|2|-|state-mismatch|fragment:root/r:1|keyed-order|keys:7,9,8|keys:7,8,9",
    )
}

fn trace_mismatch_bytes() -> Vec<u8> {
    trace_mismatch_on(verified_artifact_bytes())
}

fn trace_mismatch_on(bytes: Vec<u8>) -> Vec<u8> {
    replace_once(
        bytes,
        "event|0|2|3,4|0|1|commit|1|structure,layout,paint|mismatch",
        "event|0|2|3,4|0|1|commit|2|structure,layout,paint|mismatch",
    )
}

fn reduction_mismatch_bytes() -> Vec<u8> {
    reduction_mismatch_on(verified_artifact_bytes())
}

fn reduction_mismatch_on(bytes: Vec<u8>) -> Vec<u8> {
    replace_once(
        bytes,
        "reducer|4096|35|fixed-point",
        "reducer|4096|34|fixed-point",
    )
}

fn verified_artifact_bytes() -> Vec<u8> {
    replace_once(
        CANONICAL.to_vec(),
        "reducer|4096|4096|budget-exhausted",
        "reducer|4096|35|fixed-point",
    )
}

fn assert_verification_error(
    bytes: Vec<u8>,
    kind: ArtifactVerificationErrorKind,
    transaction: Option<TransactionIdV1>,
    operation: Option<OperationIdV1>,
) {
    let error = verification_error(&bytes);
    assert_eq!(error.kind(), kind);
    assert_eq!(error.transaction(), transaction);
    assert_eq!(error.operation(), operation);
}

fn assert_verification_kind(bytes: Vec<u8>, kind: ArtifactVerificationErrorKind) {
    assert_eq!(verification_error(&bytes).kind(), kind);
}

fn verification_error(bytes: &[u8]) -> ArtifactVerificationError {
    let artifact: FailureArtifactV1 = decode_failure_artifact_v1(bytes)
        .expect("verification mutation should remain structurally valid");
    verify_failure_artifact_v1(&artifact).expect_err("invalid artifact should not verify")
}

fn verification_kind_rank(kind: ArtifactVerificationErrorKind) -> usize {
    match kind {
        ArtifactVerificationErrorKind::FixtureMismatch => 0,
        ArtifactVerificationErrorKind::ReplayConfigMismatch => 1,
        ArtifactVerificationErrorKind::InvalidSemanticPath => 2,
        ArtifactVerificationErrorKind::InvalidSemanticOperation => 3,
        ArtifactVerificationErrorKind::SeedMismatch => 4,
        ArtifactVerificationErrorKind::OriginalFailureMismatch => 5,
        ArtifactVerificationErrorKind::MinimizedFailureMismatch => 6,
        ArtifactVerificationErrorKind::FingerprintMismatch => 7,
        ArtifactVerificationErrorKind::TraceMismatch => 8,
        ArtifactVerificationErrorKind::FaultFreeReplayFailed => 9,
        ArtifactVerificationErrorKind::ReductionMismatch => 10,
    }
}

fn replace_once(bytes: Vec<u8>, before: &str, after: &str) -> Vec<u8> {
    let input = String::from_utf8(bytes).expect("test artifact should be ASCII");
    assert_eq!(input.matches(before).count(), 1);
    input.replacen(before, after, 1).into_bytes()
}
