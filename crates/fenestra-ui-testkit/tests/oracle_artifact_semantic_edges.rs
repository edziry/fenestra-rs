use fenestra_ui_testkit::prototype::{
    ArtifactVerificationError, ArtifactVerificationErrorKind, FailureArtifactV1, OperationIdV1,
    TransactionIdV1, decode_failure_artifact_v1, verify_failure_artifact_v1,
};

const CANONICAL: &[u8] = include_bytes!("fixtures/canonical_structural_failure_v1.txt");
const ORIGINAL_FAILURE: &str =
    "failure|original|2|-|state-mismatch|fragment:root/r:1|keyed-order|keys:9,7,8|keys:7,8,9";

#[test]
fn fingerprint_locations_and_path_summaries_must_resolve_in_the_fixture() {
    let invalid_fingerprints = [
        "failure|original|2|-|state-mismatch|fragment:root/r:9|keyed-order|keys:9,7,8|keys:7,8,9",
        "failure|original|2|-|state-mismatch|node:root|parent|node:root/s:9|none",
        "failure|original|2|-|state-mismatch|node:root|child-order|nodes:root/s:9|nodes:root/s:0",
        "failure|original|2|-|state-mismatch|node:root|child-order|children:s:root/s:0|children:r:root/r:9",
    ];

    for fingerprint in invalid_fingerprints {
        assert_verification_error(
            replace_once(CANONICAL, ORIGINAL_FAILURE, fingerprint),
            ArtifactVerificationErrorKind::InvalidSemanticPath,
            Some(TransactionIdV1::new(2)),
            None,
        );
    }
}

#[test]
fn a_later_minimized_path_error_precedes_an_original_operation_error() {
    let bytes = replace_case_record(
        CANONICAL,
        "original",
        "op|0|set|root|0|i32:320",
        "op|0|set|root|9|i32:320",
    );
    let bytes = replace_case_record(
        &bytes,
        "minimized",
        "op|3|insert|root/r:1|9|2",
        "op|3|insert|root/r:9|9|2",
    );

    assert_verification_error(
        bytes,
        ArtifactVerificationErrorKind::InvalidSemanticPath,
        Some(TransactionIdV1::new(2)),
        Some(OperationIdV1::new(3)),
    );
}

#[test]
fn targets_created_in_the_current_transaction_are_not_base_snapshot_targets() {
    let created_node = replace_original_operations(
        "op|0|insert|root/r:1|99|2",
        "op|1|set|root/m:1:99|0|i32:480",
    );
    assert_verification_error(
        created_node,
        ArtifactVerificationErrorKind::InvalidSemanticOperation,
        Some(TransactionIdV1::new(0)),
        Some(OperationIdV1::new(1)),
    );

    let created_fragment = replace_original_operations(
        "op|0|insert|root/r:1|99|2",
        "op|1|insert|root/m:1:99/r:1|2|1",
    );
    assert_verification_error(
        created_fragment,
        ArtifactVerificationErrorKind::InvalidSemanticOperation,
        Some(TransactionIdV1::new(0)),
        Some(OperationIdV1::new(1)),
    );
}

#[test]
fn a_retired_and_reinserted_owner_is_a_new_incarnation() {
    let bytes = replace_original_operations("op|0|remove|root/r:1|7", "op|1|insert|root/r:1|7|0");
    let bytes = replace_case_record(&bytes, "original", "tx|0|2", "tx|0|3");
    let bytes = replace_once(&bytes, "tx|1|1\n", "");
    let bytes = refresh_case_declaration(&bytes, "original");
    let bytes = replace_case_record(
        &bytes,
        "original",
        "op|2|set|root|0|i32:480",
        "op|2|update|root/m:1:7/r:1|1|0|i32:30",
    );

    assert_verification_error(
        bytes,
        ArtifactVerificationErrorKind::InvalidSemanticOperation,
        Some(TransactionIdV1::new(0)),
        Some(OperationIdV1::new(2)),
    );
}

#[test]
fn operations_may_use_current_keys_when_the_fragment_existed_in_the_base_snapshot() {
    for second in [
        "op|1|move|root/r:1|99|0",
        "op|1|update|root/r:1|99|0|i32:480",
        "op|1|remove|root/r:1|99",
    ] {
        let bytes = replace_original_operations("op|0|insert|root/r:1|99|2", second);
        assert_verification_error(
            bytes,
            ArtifactVerificationErrorKind::SeedMismatch,
            None,
            None,
        );
    }
}

fn replace_original_operations(first: &str, second: &str) -> Vec<u8> {
    let bytes = replace_case_record(CANONICAL, "original", "op|0|set|root|0|i32:320", first);
    replace_case_record(&bytes, "original", "op|1|set|root|0|i32:480", second)
}

fn assert_verification_error(
    bytes: Vec<u8>,
    kind: ArtifactVerificationErrorKind,
    transaction: Option<TransactionIdV1>,
    operation: Option<OperationIdV1>,
) {
    let artifact: FailureArtifactV1 = decode_failure_artifact_v1(&bytes)
        .expect("semantic verification mutation should decode structurally");
    let error: ArtifactVerificationError = verify_failure_artifact_v1(&artifact)
        .expect_err("semantic verification mutation should not verify");
    assert_eq!(error.kind(), kind);
    assert_eq!(error.transaction(), transaction);
    assert_eq!(error.operation(), operation);
}

fn replace_case_record(input: &[u8], section: &str, before: &str, after: &str) -> Vec<u8> {
    let input = std::str::from_utf8(input).expect("test artifact should be ASCII");
    let begin = format!("{section}-begin|");
    let end = format!("{section}-end");
    let mut within_section = false;
    let mut replacements = 0_usize;
    let lines = input
        .lines()
        .map(|line| {
            if line.starts_with(&begin) {
                within_section = true;
            } else if line == end {
                within_section = false;
            }
            if within_section && line == before {
                replacements += 1;
                after.to_owned()
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(replacements, 1);
    let mut replaced = lines.join("\n").into_bytes();
    replaced.push(b'\n');
    refresh_case_declaration(&replaced, section)
}

fn refresh_case_declaration(input: &[u8], section: &str) -> Vec<u8> {
    let input = std::str::from_utf8(input).expect("test artifact should be ASCII");
    let begin = format!("{section}-begin|");
    let end = format!("{section}-end");
    let mut lines = input.lines().map(str::to_owned).collect::<Vec<_>>();
    let begin_index = lines
        .iter()
        .position(|line| line.starts_with(&begin))
        .expect("case declaration should exist");
    let end_index = lines
        .iter()
        .position(|line| line == &end)
        .expect("case terminator should exist");
    let records = &lines[begin_index + 1..end_index];
    let transactions = records
        .iter()
        .filter(|line| line.starts_with("tx|"))
        .count();
    let operations = records
        .iter()
        .filter(|line| line.starts_with("op|"))
        .count();
    let bytes = records.iter().map(|line| line.len() + 1).sum::<usize>();
    lines[begin_index] = format!("{section}-begin|{transactions}|{operations}|{bytes}");
    let mut output = lines.join("\n").into_bytes();
    output.push(b'\n');
    output
}

fn replace_once(input: &[u8], before: &str, after: &str) -> Vec<u8> {
    let input = std::str::from_utf8(input).expect("test artifact should be ASCII");
    assert_eq!(input.matches(before).count(), 1);
    input.replacen(before, after, 1).into_bytes()
}
