use super::support::{
    CANONICAL_TRACE_EVENT, canonical_structural_artifact, decode_error, replace_once,
};
use super::trace_fixture::with_trace;
use crate::wire::error::ArtifactDecodeErrorKind;

#[test]
fn property_value_enum_is_closed_before_earlier_numeric_canonicality() {
    const PRIVATE: &str = "pvt";
    let input = with_early_noncanonical(&replace_once(
        canonical_structural_artifact(),
        "i32:320",
        "pvt:320",
    ));

    assert_private_malformed(&input, 9, PRIVATE);
}

#[test]
fn fingerprint_field_is_closed_before_earlier_numeric_canonicality() {
    const PRIVATE: &str = "private-field";
    let input = with_early_noncanonical(&replace_once(
        canonical_structural_artifact(),
        "failure|original|2|-|state-mismatch|fragment:root/r:1|keyed-order|",
        &format!("failure|original|2|-|state-mismatch|fragment:root/r:1|{PRIVATE}|"),
    ));

    assert_private_malformed(&input, 28, PRIVATE);
}

#[test]
fn fingerprint_summary_is_closed_before_earlier_numeric_canonicality() {
    const PRIVATE: &str = "private-summary";
    let input = with_early_noncanonical(&replace_once(
        canonical_structural_artifact(),
        "failure|original|2|-|state-mismatch|fragment:root/r:1|keyed-order|keys:9,7,8|keys:7,8,9",
        &format!(
            "failure|original|2|-|state-mismatch|fragment:root/r:1|keyed-order|{PRIVATE}|keys:7,8,9"
        ),
    ));

    assert_private_malformed(&input, 28, PRIVATE);
}

#[test]
fn fingerprint_rejection_is_closed_before_earlier_numeric_canonicality() {
    const PRIVATE: &str = "private-rejection";
    let failure = format!(
        "failure|original|2|4|candidate-rejected|global|candidate-outcome|kind:accept|kind:{PRIVATE}"
    );
    let input = with_early_noncanonical(&replace_once(
        canonical_structural_artifact(),
        "failure|original|2|-|state-mismatch|fragment:root/r:1|keyed-order|keys:9,7,8|keys:7,8,9",
        &failure,
    ));

    assert_private_malformed(&input, 28, PRIVATE);
}

#[test]
fn trace_outcome_is_closed_before_earlier_numeric_canonicality() {
    const PRIVATE: &str = "private-outcome";
    let event = CANONICAL_TRACE_EVENT.replacen("commit", PRIVATE, 1);
    let input = with_early_noncanonical(&with_trace(&[event]));

    assert_private_malformed(&input, 37, PRIVATE);
}

#[test]
fn trace_invalidation_is_closed_before_earlier_numeric_canonicality() {
    const PRIVATE: &str = "private-invalidation";
    let event = CANONICAL_TRACE_EVENT.replacen("structure,layout,paint", PRIVATE, 1);
    let input = with_early_noncanonical(&with_trace(&[event]));

    assert_private_malformed(&input, 37, PRIVATE);
}

#[test]
fn trace_comparison_is_closed_before_earlier_numeric_canonicality() {
    const PRIVATE: &str = "private-comparison";
    let event = CANONICAL_TRACE_EVENT.replacen("mismatch", PRIVATE, 1);
    let input = with_early_noncanonical(&with_trace(&[event]));

    assert_private_malformed(&input, 37, PRIVATE);
}

#[test]
fn empty_or_trailing_fingerprint_list_is_noncanonical_not_malformed() {
    for summary in ["children:", "children:s:root,"] {
        let failure = format!(
            "failure|original|2|-|state-mismatch|node:root|child-order|{summary}|children:s:root"
        );
        let input = replace_once(
            canonical_structural_artifact(),
            "failure|original|2|-|state-mismatch|fragment:root/r:1|keyed-order|keys:9,7,8|keys:7,8,9",
            &failure,
        );
        let error = decode_error(&input);

        assert_eq!(error.kind(), ArtifactDecodeErrorKind::NonCanonicalValue);
        assert_eq!(error.line(), Some(28));
    }
}

#[test]
fn empty_invalidation_list_is_noncanonical_not_malformed() {
    let event = CANONICAL_TRACE_EVENT.replacen("structure,layout,paint", "", 1);
    let error = decode_error(&with_trace(&[event]));

    assert_eq!(error.kind(), ArtifactDecodeErrorKind::NonCanonicalValue);
    assert_eq!(error.line(), Some(37));
}

#[test]
fn an_explicit_empty_transaction_is_malformed_before_numeric_canonicality() {
    let input = with_early_noncanonical(&replace_once(
        canonical_structural_artifact(),
        "original-end\n",
        "tx|10|0\noriginal-end\n",
    ));
    let error = decode_error(&input);

    assert_eq!(error.kind(), ArtifactDecodeErrorKind::MalformedRecord);
    assert_eq!(error.line(), Some(26));
}

fn with_early_noncanonical(input: &[u8]) -> Vec<u8> {
    replace_once(input, "seed|0", "seed|01")
}

fn assert_private_malformed(input: &[u8], line: u32, private: &str) {
    let error = decode_error(input);
    assert_eq!(error.kind(), ArtifactDecodeErrorKind::MalformedRecord);
    assert_eq!(error.line(), Some(line));
    assert!(!format!("{error:?}").contains(private));
    assert!(!error.to_string().contains(private));
}
