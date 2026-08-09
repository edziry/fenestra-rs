use super::support::{canonical_structural_artifact, decode_error, replace_once};
use crate::wire::error::{ArtifactDecodeErrorKind, CountKind};

#[test]
fn original_counts_follow_begin_fields_then_transaction_order() {
    let cases = [
        original_counts("original-begin|9|11|320"),
        original_counts("original-begin|8|11|320"),
        original_counts("original-begin|8|10|320"),
        original_counts("original-begin|8|10|319"),
    ];

    assert_eq!(
        signatures(&cases),
        vec![
            mismatch(CountKind::Transactions, 7),
            mismatch(CountKind::Operations, 7),
            mismatch(CountKind::CaseBytes, 7),
            mismatch(CountKind::OperationsPerTransaction, 8),
        ]
    );
}

#[test]
fn minimized_counts_follow_begin_fields_then_transaction_order() {
    let cases = [
        minimized_counts("minimized-begin|2|3|56"),
        minimized_counts("minimized-begin|1|3|56"),
        minimized_counts("minimized-begin|1|2|56"),
        minimized_counts("minimized-begin|1|2|55"),
    ];

    assert_eq!(
        signatures(&cases),
        vec![
            mismatch(CountKind::Transactions, 30),
            mismatch(CountKind::Operations, 30),
            mismatch(CountKind::CaseBytes, 30),
            mismatch(CountKind::OperationsPerTransaction, 31),
        ]
    );
}

#[test]
fn case_sections_complete_their_counts_before_the_next_section() {
    let original_bytes_and_minimized_transactions = replace_once(
        &replace_once(
            canonical_structural_artifact(),
            "original-begin|8|10|319",
            "original-begin|8|10|320",
        ),
        "minimized-begin|1|2|55",
        "minimized-begin|2|2|55",
    );
    assert_signature(
        &original_bytes_and_minimized_transactions,
        mismatch(CountKind::CaseBytes, 7),
    );

    let original_transaction_and_minimized_transactions = replace_once(
        &replace_once(canonical_structural_artifact(), "tx|0|2", "tx|0|4"),
        "minimized-begin|1|2|55",
        "minimized-begin|2|2|55",
    );
    assert_signature(
        &original_transaction_and_minimized_transactions,
        mismatch(CountKind::OperationsPerTransaction, 8),
    );
}

#[test]
fn trace_counts_check_events_before_bytes() {
    let both = replace_once(
        canonical_structural_artifact(),
        "trace-begin|1|59",
        "trace-begin|2|60",
    );
    assert_signature(&both, mismatch(CountKind::TraceEvents, 36));

    let bytes = replace_once(
        canonical_structural_artifact(),
        "trace-begin|1|59",
        "trace-begin|1|60",
    );
    assert_signature(&bytes, mismatch(CountKind::TraceBytes, 36));
}

#[test]
fn all_counts_precede_id_order_and_fingerprint_legality() {
    let original = with_original_id_and_fingerprint_defects(&replace_once(
        canonical_structural_artifact(),
        "original-begin|8|10|319",
        "original-begin|8|10|320",
    ));
    let minimized = with_original_fingerprint_defect(&replace_minimized_operation(
        &replace_once(
            canonical_structural_artifact(),
            "minimized-begin|1|2|55",
            "minimized-begin|1|2|56",
        ),
        "op|3|move",
    ));
    let trace = with_original_id_and_fingerprint_defects(&replace_once(
        canonical_structural_artifact(),
        "trace-begin|1|59",
        "trace-begin|1|60",
    ));

    assert_eq!(
        signatures(&[original, minimized, trace]),
        vec![
            mismatch(CountKind::CaseBytes, 7),
            mismatch(CountKind::CaseBytes, 30),
            mismatch(CountKind::TraceBytes, 36),
        ]
    );
}

fn original_counts(begin: &str) -> Vec<u8> {
    replace_once(
        &replace_once(
            canonical_structural_artifact(),
            "original-begin|8|10|319",
            begin,
        ),
        "tx|0|2",
        "tx|0|4",
    )
}

fn minimized_counts(begin: &str) -> Vec<u8> {
    let transaction = replace_once(
        canonical_structural_artifact(),
        "minimized-begin|1|2|55\ntx|2|2",
        "minimized-begin|1|2|55\ntx|2|4",
    );
    replace_once(&transaction, "minimized-begin|1|2|55", begin)
}

fn with_original_id_and_fingerprint_defects(input: &[u8]) -> Vec<u8> {
    with_original_fingerprint_defect(&replace_once(input, "tx|1|1", "tx|0|1"))
}

fn with_original_fingerprint_defect(input: &[u8]) -> Vec<u8> {
    replace_once(
        input,
        "failure|original|2|-|state-mismatch",
        "failure|original|2|4|state-mismatch",
    )
}

fn replace_minimized_operation(input: &[u8], replacement: &str) -> Vec<u8> {
    replace_once(
        input,
        "minimized-begin|1|2|56\ntx|2|2\nop|3|insert|root/r:1|9|2\nop|4|move",
        &format!("minimized-begin|1|2|56\ntx|2|2\nop|3|insert|root/r:1|9|2\n{replacement}"),
    )
}

fn signatures(inputs: &[Vec<u8>]) -> Vec<(ArtifactDecodeErrorKind, Option<u32>)> {
    inputs.iter().map(|input| signature(input)).collect()
}

fn assert_signature(input: &[u8], expected: (ArtifactDecodeErrorKind, Option<u32>)) {
    assert_eq!(signature(input), expected);
}

fn signature(input: &[u8]) -> (ArtifactDecodeErrorKind, Option<u32>) {
    let error = decode_error(input);
    (error.kind(), error.line())
}

const fn mismatch(kind: CountKind, line: u32) -> (ArtifactDecodeErrorKind, Option<u32>) {
    (ArtifactDecodeErrorKind::CountMismatch(kind), Some(line))
}
