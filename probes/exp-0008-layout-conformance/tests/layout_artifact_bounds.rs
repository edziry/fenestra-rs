#![forbid(unsafe_code)]

#[path = "layout_artifact/encode.rs"]
mod encode;

use encode::{
    LayoutArtifactErrorKindV1, LayoutArtifactLimitKindV1, LayoutArtifactLimitsV1,
    REGISTERED_LAYOUT_ARTIFACT_LIMITS_V1, encode_layout_artifact_v1,
};

const RECORDS: usize = 512;
const LINE_BYTES: usize = 512;
const ARTIFACT_BYTES: usize = 65_536;

#[test]
fn artifact_limit_vocabulary_and_registered_profile_are_closed() {
    assert_eq!(
        LayoutArtifactLimitKindV1::ALL,
        [
            LayoutArtifactLimitKindV1::Records,
            LayoutArtifactLimitKindV1::LineBytes,
            LayoutArtifactLimitKindV1::ArtifactBytes,
        ]
    );
    assert_eq!(
        LayoutArtifactErrorKindV1::ALL,
        [
            LayoutArtifactErrorKindV1::InvalidRecord,
            LayoutArtifactErrorKindV1::LimitExceeded(LayoutArtifactLimitKindV1::Records),
            LayoutArtifactErrorKindV1::LimitExceeded(LayoutArtifactLimitKindV1::LineBytes),
            LayoutArtifactErrorKindV1::LimitExceeded(LayoutArtifactLimitKindV1::ArtifactBytes),
        ]
    );
    assert_eq!(
        REGISTERED_LAYOUT_ARTIFACT_LIMITS_V1,
        LayoutArtifactLimitsV1::new(RECORDS, LINE_BYTES, ARTIFACT_BYTES)
    );
    assert_eq!(
        REGISTERED_LAYOUT_ARTIFACT_LIMITS_V1.limit(LayoutArtifactLimitKindV1::Records),
        RECORDS
    );
    assert_eq!(
        REGISTERED_LAYOUT_ARTIFACT_LIMITS_V1.limit(LayoutArtifactLimitKindV1::LineBytes),
        LINE_BYTES
    );
    assert_eq!(
        REGISTERED_LAYOUT_ARTIFACT_LIMITS_V1.limit(LayoutArtifactLimitKindV1::ArtifactBytes),
        ARTIFACT_BYTES
    );
}

#[test]
fn record_limit_is_inclusive_and_rejects_one_over() {
    assert!(encode(&repeated_lines(RECORDS, 1)).is_ok());
    assert_limit(
        &repeated_lines(RECORDS + 1, 1),
        LayoutArtifactLimitKindV1::Records,
        RECORDS + 1,
        RECORDS,
    );
}

#[test]
fn line_limit_is_inclusive_and_rejects_one_over() {
    assert!(encode(&repeated_lines(1, LINE_BYTES)).is_ok());
    assert_limit(
        &repeated_lines(1, LINE_BYTES + 1),
        LayoutArtifactLimitKindV1::LineBytes,
        LINE_BYTES + 1,
        LINE_BYTES,
    );
}

#[test]
fn artifact_limit_is_inclusive_and_rejects_one_over() {
    let exact = repeated_lines(128, 511);
    let one_over = artifact_one_over_lines();
    assert_eq!(encoded_bytes(&exact), ARTIFACT_BYTES);
    assert_eq!(
        encode(&exact)
            .expect("exact artifact byte limit should be accepted")
            .as_bytes()
            .len(),
        ARTIFACT_BYTES
    );
    assert_limit(
        &one_over,
        LayoutArtifactLimitKindV1::ArtifactBytes,
        ARTIFACT_BYTES + 1,
        ARTIFACT_BYTES,
    );
}

#[test]
fn simultaneous_crossings_follow_records_line_then_artifact_priority() {
    let mut record_crossing = repeated_lines(RECORDS + 1, LINE_BYTES + 1);
    record_crossing[0].clear();
    assert_limit(
        &record_crossing,
        LayoutArtifactLimitKindV1::Records,
        RECORDS + 1,
        RECORDS,
    );

    let invalid_and_long = format!("{}\n", "x".repeat(LINE_BYTES + 1));
    assert_invalid(&invalid_and_long);

    let earlier_long_then_invalid = vec!["x".repeat(LINE_BYTES + 1), String::new()];
    assert_limit(
        &earlier_long_then_invalid,
        LayoutArtifactLimitKindV1::LineBytes,
        LINE_BYTES + 1,
        LINE_BYTES,
    );

    let mut line_and_artifact = repeated_lines(127, 511);
    line_and_artifact.push("x".repeat(LINE_BYTES + 1));
    assert!(encoded_bytes(&line_and_artifact) > ARTIFACT_BYTES);
    assert_limit(
        &line_and_artifact,
        LayoutArtifactLimitKindV1::LineBytes,
        LINE_BYTES + 1,
        LINE_BYTES,
    );

    let mut invalid_and_artifact = repeated_lines(128, 511);
    invalid_and_artifact.push(String::new());
    assert!(encoded_bytes(&invalid_and_artifact) > ARTIFACT_BYTES);
    assert_invalid_records(&invalid_and_artifact);

    assert_limit(
        &artifact_one_over_lines(),
        LayoutArtifactLimitKindV1::ArtifactBytes,
        ARTIFACT_BYTES + 1,
        ARTIFACT_BYTES,
    );
}

#[test]
fn empty_artifact_is_rejected() {
    assert_invalid_records(&[]);
}

#[test]
fn empty_record_is_rejected() {
    assert_invalid("");
}

#[test]
fn embedded_lf_is_rejected() {
    assert_invalid("private\npayload");
}

#[test]
fn embedded_cr_is_rejected() {
    assert_invalid("private\rpayload");
}

#[test]
fn control_byte_is_rejected() {
    assert_invalid("private\x1fpayload");
}

#[test]
fn delete_byte_is_rejected() {
    assert_invalid("private\x7fpayload");
}

#[test]
fn non_ascii_record_is_rejected() {
    assert_invalid("private-payload-\u{e9}");
}

#[test]
fn canonical_line_encoding_is_printable_deterministic_and_has_one_final_lf() {
    let lines = vec![
        "layout-artifact|1".to_owned(),
        "case|column-basic|pass".to_owned(),
        "result|records=2".to_owned(),
    ];
    let first = encode(&lines).expect("canonical lines should encode");
    let second = encode(&lines).expect("the same canonical lines should encode again");

    assert_eq!(first.as_bytes(), second.as_bytes());
    assert_eq!(
        first.as_str(),
        "layout-artifact|1\ncase|column-basic|pass\nresult|records=2\n"
    );
    assert!(first.as_bytes().ends_with(b"\n"));
    assert!(!first.as_bytes().ends_with(b"\n\n"));
    assert_eq!(
        first
            .as_bytes()
            .iter()
            .filter(|byte| **byte == b'\n')
            .count(),
        lines.len()
    );
    assert!(
        first
            .as_bytes()
            .iter()
            .all(|byte| *byte == b'\n' || (0x20..=0x7e).contains(byte))
    );
}

#[test]
fn limit_errors_expose_only_kind_observed_and_maximum() {
    let mut private_line = "private-payload|".to_owned();
    private_line.push_str(&"x".repeat(LINE_BYTES + 1 - private_line.len()));
    let error = encode(&[private_line])
        .expect_err("one-over private line should fail without retaining its payload");

    assert_eq!(
        error.kind(),
        LayoutArtifactErrorKindV1::LimitExceeded(LayoutArtifactLimitKindV1::LineBytes)
    );
    assert_eq!(error.observed(), Some(LINE_BYTES + 1));
    assert_eq!(error.maximum(), Some(LINE_BYTES));
    assert_eq!(
        error.to_string(),
        "layout-artifact-limit-exceeded(line-bytes;observed=513;maximum=512)"
    );
    assert_eq!(
        format!("{error:?}"),
        "LayoutArtifactErrorV1(layout-artifact-limit-exceeded(line-bytes;observed=513;maximum=512))"
    );
    assert!(!error.to_string().contains("private-payload"));
    assert!(!format!("{error:?}").contains("private-payload"));
}

fn encode(lines: &[String]) -> Result<encode::LayoutArtifactV1, encode::LayoutArtifactErrorV1> {
    encode_layout_artifact_v1(lines, REGISTERED_LAYOUT_ARTIFACT_LIMITS_V1)
}

fn assert_limit(
    lines: &[String],
    kind: LayoutArtifactLimitKindV1,
    observed: usize,
    maximum: usize,
) {
    let error = encode(lines).expect_err("one-over artifact fixture should fail");
    assert_eq!(error.kind(), LayoutArtifactErrorKindV1::LimitExceeded(kind));
    assert_eq!(error.observed(), Some(observed));
    assert_eq!(error.maximum(), Some(maximum));
}

fn assert_invalid(record: &str) {
    assert_invalid_records(&[record.to_owned()]);
}

fn assert_invalid_records(records: &[String]) {
    let error = encode(records).expect_err("invalid artifact record should fail");
    assert_eq!(error.kind(), LayoutArtifactErrorKindV1::InvalidRecord);
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
    assert_eq!(error.to_string(), "layout-artifact-invalid-record");
    assert_eq!(
        format!("{error:?}"),
        "LayoutArtifactErrorV1(layout-artifact-invalid-record)"
    );
    for record in records.iter().filter(|record| !record.is_empty()) {
        assert!(!error.to_string().contains(record));
        assert!(!format!("{error:?}").contains(record));
    }
}

fn repeated_lines(records: usize, line_bytes: usize) -> Vec<String> {
    vec!["x".repeat(line_bytes); records]
}

fn artifact_one_over_lines() -> Vec<String> {
    let mut lines = repeated_lines(127, 511);
    lines.push("x".repeat(512));
    lines
}

fn encoded_bytes(lines: &[String]) -> usize {
    lines.iter().map(|line| line.len() + 1).sum()
}
