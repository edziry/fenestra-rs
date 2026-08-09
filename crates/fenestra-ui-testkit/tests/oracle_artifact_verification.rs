use fenestra_ui_testkit::prototype::{
    ArtifactVerificationError, ArtifactVerificationErrorKind, FailureArtifactV1, OperationIdV1,
    TransactionIdV1, decode_failure_artifact_v1, verify_failure_artifact_v1,
};

const CANONICAL: &[u8] = include_bytes!("fixtures/canonical_structural_failure_v1.txt");

#[test]
fn verification_inputs_are_structurally_decodable() {
    for bytes in [
        verified_artifact_bytes(),
        fixture_mismatch_bytes(),
        replay_config_mismatch_bytes(),
        invalid_semantic_path_bytes(),
        invalid_semantic_operation_bytes(),
        seed_mismatch_bytes(),
    ] {
        let _: FailureArtifactV1 = decode_failure_artifact_v1(&bytes)
            .expect("verification mutation should remain structurally valid");
    }
}

#[test]
fn known_failure_artifact_verifies_all_v1_phases() {
    let artifact = decode_artifact(&verified_artifact_bytes());

    verify_failure_artifact_v1(&artifact).expect("known failure artifact should verify");
}

#[test]
fn verification_reports_the_first_closed_provenance_or_semantic_error() {
    let transaction = Some(TransactionIdV1::new(0));
    let operation = Some(OperationIdV1::new(0));
    assert_verification_error(
        fixture_mismatch_bytes(),
        ArtifactVerificationErrorKind::FixtureMismatch,
        None,
        None,
    );
    assert_verification_error(
        replay_config_mismatch_bytes(),
        ArtifactVerificationErrorKind::ReplayConfigMismatch,
        None,
        None,
    );
    assert_verification_error(
        invalid_semantic_path_bytes(),
        ArtifactVerificationErrorKind::InvalidSemanticPath,
        transaction,
        operation,
    );
    assert_verification_error(
        invalid_semantic_operation_bytes(),
        ArtifactVerificationErrorKind::InvalidSemanticOperation,
        transaction,
        operation,
    );
    assert_verification_error(
        seed_mismatch_bytes(),
        ArtifactVerificationErrorKind::SeedMismatch,
        None,
        None,
    );
}

#[test]
fn verification_priority_precedes_later_validly_decoded_failures() {
    assert_verification_kind(
        fixture_and_replay_mismatch_bytes(),
        ArtifactVerificationErrorKind::FixtureMismatch,
    );
    assert_verification_kind(
        replay_and_path_mismatch_bytes(),
        ArtifactVerificationErrorKind::ReplayConfigMismatch,
    );
    assert_verification_kind(
        path_and_operation_mismatch_bytes(),
        ArtifactVerificationErrorKind::InvalidSemanticPath,
    );
    assert_verification_kind(
        operation_and_seed_mismatch_bytes(),
        ArtifactVerificationErrorKind::InvalidSemanticOperation,
    );
}

#[test]
fn verification_errors_are_bounded_and_privacy_safe() {
    let error = verification_error(&fixture_mismatch_bytes());
    assert_error_trait(&error);
    assert_eq!(
        format!("{error:?}"),
        "ArtifactVerificationError { kind: FixtureMismatch, transaction: None, operation: None }"
    );
    assert_eq!(
        error.to_string(),
        "runtime oracle artifact verification failed: FixtureMismatch"
    );
}

fn verified_artifact_bytes() -> Vec<u8> {
    replace_once(
        CANONICAL.to_vec(),
        "reducer|4096|4096|budget-exhausted",
        "reducer|4096|35|fixed-point",
    )
}

fn fixture_mismatch_bytes() -> Vec<u8> {
    replace_once(
        verified_artifact_bytes(),
        "fixture|runtime-oracle|1|1|5001|1|1",
        "fixture|runtime-oracle|1|1|5002|1|1",
    )
}

fn replay_config_mismatch_bytes() -> Vec<u8> {
    replace_once(
        verified_artifact_bytes(),
        "replay|4|64|256|128|1024|1",
        "replay|3|64|256|128|1024|1",
    )
}

fn invalid_semantic_path_bytes() -> Vec<u8> {
    invalid_semantic_path_on(verified_artifact_bytes())
}

fn invalid_semantic_path_on(bytes: Vec<u8>) -> Vec<u8> {
    let bytes = replace_once(bytes, "original-begin|8|10|319", "original-begin|8|10|323");
    replace_once(
        bytes,
        "op|0|set|root|0|i32:320",
        "op|0|set|root/s:9|0|i32:320",
    )
}

fn invalid_semantic_operation_bytes() -> Vec<u8> {
    invalid_semantic_operation_on(verified_artifact_bytes())
}

fn invalid_semantic_operation_on(bytes: Vec<u8>) -> Vec<u8> {
    replace_once(bytes, "op|0|set|root|0|i32:320", "op|0|set|root|9|i32:320")
}

fn seed_mismatch_bytes() -> Vec<u8> {
    seed_mismatch_on(verified_artifact_bytes())
}

fn seed_mismatch_on(bytes: Vec<u8>) -> Vec<u8> {
    replace_once(bytes, "generator|8|2|8", "generator|9|2|8")
}

fn fixture_and_replay_mismatch_bytes() -> Vec<u8> {
    replace_once(
        fixture_mismatch_bytes(),
        "replay|4|64|256|128|1024|1",
        "replay|3|64|256|128|1024|1",
    )
}

fn replay_and_path_mismatch_bytes() -> Vec<u8> {
    invalid_semantic_path_on(replay_config_mismatch_bytes())
}

fn path_and_operation_mismatch_bytes() -> Vec<u8> {
    replace_once(
        invalid_semantic_path_bytes(),
        "op|1|set|root|0|i32:480",
        "op|1|set|root|9|i32:480",
    )
}

fn operation_and_seed_mismatch_bytes() -> Vec<u8> {
    seed_mismatch_on(invalid_semantic_operation_bytes())
}

fn verification_error(bytes: &[u8]) -> ArtifactVerificationError {
    let artifact = decode_artifact(bytes);
    verify_failure_artifact_v1(&artifact).expect_err("invalid artifact should not verify")
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

fn decode_artifact(bytes: &[u8]) -> FailureArtifactV1 {
    decode_failure_artifact_v1(bytes).expect("verification input should decode structurally")
}

fn assert_error_trait(_: &dyn std::error::Error) {}

fn replace_once(bytes: Vec<u8>, before: &str, after: &str) -> Vec<u8> {
    let input = String::from_utf8(bytes).expect("test artifact should be ASCII");
    assert_eq!(input.matches(before).count(), 1);
    input.replacen(before, after, 1).into_bytes()
}
