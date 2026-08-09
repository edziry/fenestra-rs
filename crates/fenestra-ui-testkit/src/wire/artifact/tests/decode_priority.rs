use super::support::{
    CANONICAL_TRACE_EVENT, assert_decode_error, canonical_structural_artifact, replace_once,
};
use super::trace_fixture::{TRACE_EVENTS_LIMIT, trace_body_over_bytes_limit, with_trace};
use crate::wire::error::{ArtifactDecodeErrorKind, ArtifactLimitKind, CountKind};

#[test]
fn canonicality_in_later_sections_precedes_earlier_counts_and_fingerprint_legality() {
    let minimized_id = replace_once(
        &replace_once(
            canonical_structural_artifact(),
            "original-begin|8|10|319",
            "original-begin|9|10|319",
        ),
        "minimized-begin|1|2|55\ntx|2|2\nop|3|insert",
        "minimized-begin|1|2|55\ntx|2|2\nop|03|insert",
    );
    assert_decode_error(
        &minimized_id,
        ArtifactDecodeErrorKind::NonCanonicalValue,
        Some(32),
    );

    let trace_sequence = replace_once(
        &with_trace(&[noncanonical_trace_event()]),
        "failure|original|2|-|state-mismatch",
        "failure|original|2|4|state-mismatch",
    );
    assert_decode_error(
        &trace_sequence,
        ArtifactDecodeErrorKind::NonCanonicalValue,
        Some(37),
    );
}

#[test]
fn trace_canonicality_precedes_configured_trace_limits() {
    let mut events = vec![CANONICAL_TRACE_EVENT.to_owned(); TRACE_EVENTS_LIMIT + 1];
    events[0] = noncanonical_trace_event();
    assert_decode_error(
        &with_trace(&events),
        ArtifactDecodeErrorKind::NonCanonicalValue,
        Some(37),
    );

    let mut oversized = trace_body_over_bytes_limit();
    oversized[0] = oversized[0].replacen("event|0|2|10,", "event|00|2|0,", 1);
    assert_decode_error(
        &with_trace(&oversized),
        ArtifactDecodeErrorKind::NonCanonicalValue,
        Some(37),
    );
}

#[test]
fn trace_bytes_limit_precedes_earlier_counts_limits_and_fingerprint_legality() {
    let oversized_trace = with_trace(&trace_body_over_bytes_limit());
    for (before, after) in [
        ("original-begin|8|10|319", "original-begin|9|10|319"),
        ("original-begin|8|10|319", "original-begin|65|10|319"),
        ("tx|0|2", "tx|0|5"),
        (
            "failure|original|2|-|state-mismatch",
            "failure|original|2|4|state-mismatch",
        ),
    ] {
        assert_decode_error(
            &replace_once(&oversized_trace, before, after),
            limit(ArtifactLimitKind::TraceBytes),
            Some(36),
        );
    }
}

#[test]
fn configured_case_limits_are_inclusive_and_use_global_tie_break_order() {
    for (before, after, kind, line) in [
        (
            "original-begin|8|10|319",
            "original-begin|8|10|131073",
            ArtifactLimitKind::CaseBytes,
            7,
        ),
        (
            "original-begin|8|10|319",
            "original-begin|65|10|319",
            ArtifactLimitKind::Transactions,
            7,
        ),
        (
            "original-begin|8|10|319",
            "original-begin|8|257|319",
            ArtifactLimitKind::Operations,
            7,
        ),
        (
            "root/m:1:9/r:1",
            "root/s:0/s:0/s:0/s:0/s:0/s:0/s:0/s:0/s:0/r:1",
            ArtifactLimitKind::PathDepth,
            21,
        ),
    ] {
        assert_decode_error(
            &replace_once(canonical_structural_artifact(), before, after),
            limit(kind),
            Some(line),
        );
    }

    let case_and_trace = replace_once(
        &with_trace(&trace_body_over_bytes_limit()),
        "original-begin|8|10|319",
        "original-begin|8|10|131073",
    );
    assert_decode_error(
        &case_and_trace,
        limit(ArtifactLimitKind::CaseBytes),
        Some(7),
    );

    let transactions_and_operations = replace_once(
        &replace_once(
            canonical_structural_artifact(),
            "original-begin|8|10|319",
            "original-begin|65|257|319",
        ),
        "tx|0|2",
        "tx|0|5",
    );
    assert_decode_error(
        &transactions_and_operations,
        limit(ArtifactLimitKind::Transactions),
        Some(7),
    );
}

#[test]
fn exact_case_limit_declarations_reach_count_validation() {
    for (before, after, count, line) in [
        (
            "original-begin|8|10|319",
            "original-begin|8|10|131072",
            CountKind::CaseBytes,
            7,
        ),
        (
            "original-begin|8|10|319",
            "original-begin|64|10|319",
            CountKind::Transactions,
            7,
        ),
        (
            "original-begin|8|10|319",
            "original-begin|8|256|319",
            CountKind::Operations,
            7,
        ),
        ("tx|0|2", "tx|0|4", CountKind::OperationsPerTransaction, 8),
    ] {
        assert_decode_error(
            &replace_once(canonical_structural_artifact(), before, after),
            ArtifactDecodeErrorKind::CountMismatch(count),
            Some(line),
        );
    }
}

fn noncanonical_trace_event() -> String {
    CANONICAL_TRACE_EVENT.replacen("event|0|", "event|00|", 1)
}

const fn limit(kind: ArtifactLimitKind) -> ArtifactDecodeErrorKind {
    ArtifactDecodeErrorKind::LimitExceeded(kind)
}
