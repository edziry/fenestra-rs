use super::super::decode_failure_artifact_v1;
use super::support::{CANONICAL_TRACE_EVENT, assert_decode_error, replace_once};
use super::trace_fixture::with_trace;
use crate::wire::error::ArtifactDecodeErrorKind;

#[test]
fn trace_sequence_is_dense_from_zero() {
    let event = CANONICAL_TRACE_EVENT.replacen("event|0|", "event|1|", 1);
    let input = with_trace(&[event]);

    assert_decode_error(&input, ArtifactDecodeErrorKind::OrderingViolation, Some(37));
}

#[test]
fn trace_sequence_order_precedes_section_references_and_the_fault_target() {
    let event = CANONICAL_TRACE_EVENT.replacen("event|0|", "event|1|", 1);
    let bad_sequence = with_trace(&[event]);
    let bad_failure = replace_once(
        &bad_sequence,
        "failure|original|2|-|state-mismatch|fragment:root/r:1|keyed-order|keys:9,7,8|keys:7,8,9",
        "failure|original|9|-|state-mismatch|fragment:root/r:1|keyed-order|keys:9,7,8|keys:7,8,9",
    );
    let input = replace_once(&bad_failure, "fault|omit-move|4", "fault|omit-move|3");

    assert_decode_error(&input, ArtifactDecodeErrorKind::OrderingViolation, Some(37));
}

#[test]
fn trace_transaction_matches_the_successive_minimized_transaction() {
    let event = CANONICAL_TRACE_EVENT.replacen("event|0|2|", "event|0|3|", 1);

    assert_trace_reference(&event);
}

#[test]
fn trace_operations_match_the_complete_minimized_transaction() {
    let event = CANONICAL_TRACE_EVENT.replacen("|3,4|", "|3,5|", 1);

    assert_trace_reference(&event);
}

#[test]
fn trace_reference_precedes_an_earlier_illegal_fingerprint() {
    let event = CANONICAL_TRACE_EVENT.replacen("event|0|2|", "event|0|3|", 1);
    let trace_reference = with_trace(&[event]);
    let input = replace_once(
        &trace_reference,
        "failure|original|2|-|state-mismatch|fragment:root/r:1|keyed-order|keys:9,7,8|keys:7,8,9",
        "failure|original|2|-|state-mismatch|node:root|fragment-binding|binding:present|binding:absent",
    );

    assert_decode_error(&input, ArtifactDecodeErrorKind::InvalidReference, Some(37));
}

#[test]
fn trace_covers_the_minimized_prefix_through_the_failure_transaction() {
    let event = String::from("event|0|1|2|0|0|noop|0|-|match");
    let one_event = with_trace(&[event]);
    let short_prefix = replace_once(
        &one_event,
        "minimized-begin|1|2|55\ntx|2|2\nop|3|insert|root/r:1|9|2\nop|4|move|root/r:1|9|0",
        "minimized-begin|2|3|86\ntx|1|1\nop|2|set|root|0|i32:480\ntx|2|2\nop|3|insert|root/r:1|9|2\nop|4|move|root/r:1|9|0",
    );
    assert_decode_error(
        &short_prefix,
        ArtifactDecodeErrorKind::InvalidReference,
        Some(38),
    );

    assert_decode_error(
        &with_trace(&[]),
        ArtifactDecodeErrorKind::InvalidReference,
        Some(36),
    );
}

#[test]
fn trace_runtime_shape_is_deferred_to_verification() {
    for event in [
        CANONICAL_TRACE_EVENT.replacen("|0|1|commit|", "|1|2|commit|", 1),
        CANONICAL_TRACE_EVENT.replacen("commit|1|structure,layout,paint", "noop|0|-", 1),
        CANONICAL_TRACE_EVENT.replacen("mismatch", "match", 1),
    ] {
        assert!(decode_failure_artifact_v1(&with_trace(&[event])).is_ok());
    }
}

fn assert_trace_reference(event: &str) {
    let input = with_trace(&[event.to_owned()]);
    assert_decode_error(&input, ArtifactDecodeErrorKind::InvalidReference, Some(37));
}
