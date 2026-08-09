use super::support::{
    CANONICAL_TRACE_EVENT, assert_decode_error, canonical_structural_artifact, decode_error,
    replace_once,
};
use super::trace_fixture::{
    TRACE_BYTES_LIMIT, TRACE_EVENTS_LIMIT, trace_body_over_bytes_limit, trace_bytes, with_trace,
};
use crate::wire::error::{ArtifactDecodeErrorKind, ArtifactLimitKind, CountKind, VersionKind};

const ARTIFACT_BYTES_LIMIT: usize = 524_288;
const LINE_BYTES_LIMIT: usize = 1_024;
const LINES_LIMIT: usize = 4_096;

#[test]
fn decode_priority_starts_with_bytes_ascii_line_and_line_count() {
    let artifact_bytes_and_ascii = vec![0xff; ARTIFACT_BYTES_LIMIT + 1];
    assert_decode_error(
        &artifact_bytes_and_ascii,
        limit(ArtifactLimitKind::ArtifactBytes),
        None,
    );

    let mut ascii_and_line = vec![b'x'; LINE_BYTES_LIMIT + 1];
    ascii_and_line.push(0xff);
    ascii_and_line.push(b'\n');
    assert_decode_error(
        &ascii_and_line,
        ArtifactDecodeErrorKind::InvalidAscii,
        Some(1),
    );

    let mut line_and_lines = vec![b'x'; LINE_BYTES_LIMIT + 1];
    line_and_lines.push(b'\n');
    line_and_lines.extend(std::iter::repeat_n(b'\n', LINES_LIMIT));
    assert_decode_error(
        &line_and_lines,
        limit(ArtifactLimitKind::LineBytes),
        Some(1),
    );

    let mut lines_without_final_lf = vec![b'\n'; LINES_LIMIT];
    lines_without_final_lf.push(b'x');
    assert_decode_error(
        &lines_without_final_lf,
        limit(ArtifactLimitKind::Lines),
        Some(4_097),
    );
}

#[test]
fn missing_final_lf_precedes_header_and_version_validation() {
    let mut input = replace_once(
        canonical_structural_artifact(),
        "fenestra-oracle-failure|1",
        "fenestra-oracle-failure|2",
    );
    assert_eq!(input.pop(), Some(b'\n'));

    assert_decode_error(&input, ArtifactDecodeErrorKind::MalformedRecord, Some(39));
}

#[test]
fn envelope_and_declared_versions_require_canonical_u32() {
    for value in ["01", "x", "4294967296"] {
        let header = replace_once(
            canonical_structural_artifact(),
            "fenestra-oracle-failure|1",
            &format!("fenestra-oracle-failure|{value}"),
        );
        assert_decode_error(&header, ArtifactDecodeErrorKind::NonCanonicalValue, Some(1));

        let versions = replace_once(
            canonical_structural_artifact(),
            "versions|fixture|1|generator",
            &format!("versions|fixture|{value}|generator"),
        );
        assert_decode_error(
            &versions,
            ArtifactDecodeErrorKind::NonCanonicalValue,
            Some(2),
        );
    }
}

#[test]
fn any_future_declared_version_precedes_earlier_version_canonicality() {
    let same_record = replace_once(
        &replace_once(
            canonical_structural_artifact(),
            "versions|fixture|1|generator|1",
            "versions|fixture|01|generator|1",
        ),
        "generator|1|case|1",
        "generator|2|case|1",
    );
    assert_decode_error(
        &same_record,
        ArtifactDecodeErrorKind::UnsupportedVersion(VersionKind::Generator),
        Some(2),
    );

    let later_record = replace_once(
        &replace_once(
            canonical_structural_artifact(),
            "fenestra-oracle-failure|1",
            "fenestra-oracle-failure|01",
        ),
        "versions|fixture|1|generator",
        "versions|fixture|2|generator",
    );
    assert_decode_error(
        &later_record,
        ArtifactDecodeErrorKind::UnsupportedVersion(VersionKind::Fixture),
        Some(2),
    );
}

#[test]
fn unsupported_version_precedes_later_numeric_and_path_canonicality() {
    let input = replace_once(
        &replace_once(
            &replace_once(
                canonical_structural_artifact(),
                "fenestra-oracle-failure|1",
                "fenestra-oracle-failure|2",
            ),
            "seed|0",
            "seed|01",
        ),
        "root/m:1:9/r:1",
        "root/m:01:9/r:1",
    );

    assert_decode_error(
        &input,
        ArtifactDecodeErrorKind::UnsupportedVersion(VersionKind::Envelope),
        Some(1),
    );
}

#[test]
fn numeric_canonicality_precedes_path_canonicality_and_configured_limits() {
    let input = replace_once(
        &replace_once(
            &with_trace(&trace_body_over_bytes_limit()),
            "seed|0",
            "seed|01",
        ),
        "root/m:1:9/r:1",
        "root/m:01:9/r:1",
    );

    assert_decode_error(&input, ArtifactDecodeErrorKind::NonCanonicalValue, Some(6));
}

#[test]
fn path_segments_require_canonical_numbers() {
    let input = replace_once(
        canonical_structural_artifact(),
        "root/m:1:9/r:1",
        "root/m:01:9/r:1",
    );

    assert_decode_error(&input, ArtifactDecodeErrorKind::NonCanonicalValue, Some(21));
}

#[test]
fn trace_bytes_limit_is_inclusive_and_anchored_to_its_begin_line() {
    let events = trace_body_over_bytes_limit();
    assert_eq!(trace_bytes(&events), TRACE_BYTES_LIMIT + 1);
    assert_eq!(events.len(), TRACE_EVENTS_LIMIT);

    assert_decode_error(
        &with_trace(&events),
        limit(ArtifactLimitKind::TraceBytes),
        Some(36),
    );
}

#[test]
fn trace_bytes_limit_checks_declared_and_actual_bytes_independently() {
    let declared_only = replace_once(
        canonical_structural_artifact(),
        "trace-begin|1|59",
        "trace-begin|1|65537",
    );
    assert_decode_error(
        &declared_only,
        limit(ArtifactLimitKind::TraceBytes),
        Some(36),
    );

    let events = trace_body_over_bytes_limit();
    let actual_only = replace_once(
        &with_trace(&events),
        "trace-begin|64|65537",
        "trace-begin|1|59",
    );
    assert_decode_error(&actual_only, limit(ArtifactLimitKind::TraceBytes), Some(36));

    let inclusive = replace_once(
        canonical_structural_artifact(),
        "trace-begin|1|59",
        "trace-begin|1|65536",
    );
    assert_decode_error(
        &inclusive,
        ArtifactDecodeErrorKind::CountMismatch(CountKind::TraceBytes),
        Some(36),
    );
}

#[test]
fn trace_event_limit_is_inclusive_and_anchored_to_its_begin_line() {
    let events = vec![CANONICAL_TRACE_EVENT.to_owned(); TRACE_EVENTS_LIMIT + 1];

    assert_decode_error(
        &with_trace(&events),
        limit(ArtifactLimitKind::TraceEvents),
        Some(36),
    );
}

#[test]
fn trace_event_limit_checks_declared_and_actual_events_independently() {
    let declared_only = replace_once(
        canonical_structural_artifact(),
        "trace-begin|1|59",
        "trace-begin|65|59",
    );
    assert_decode_error(
        &declared_only,
        limit(ArtifactLimitKind::TraceEvents),
        Some(36),
    );

    let events = vec![CANONICAL_TRACE_EVENT.to_owned(); TRACE_EVENTS_LIMIT + 1];
    let actual_only = replace_once(
        &with_trace(&events),
        "trace-begin|65|3835",
        "trace-begin|1|59",
    );
    assert_decode_error(
        &actual_only,
        limit(ArtifactLimitKind::TraceEvents),
        Some(36),
    );

    let inclusive = replace_once(
        canonical_structural_artifact(),
        "trace-begin|1|59",
        "trace-begin|64|59",
    );
    assert_decode_error(
        &inclusive,
        ArtifactDecodeErrorKind::CountMismatch(CountKind::TraceEvents),
        Some(36),
    );
}

#[test]
fn reduction_evaluation_limit_is_inclusive_and_anchored_to_reducer() {
    for input in [
        with_reduction_limits(4_097, 4_096),
        with_reduction_limits(4_096, 4_097),
    ] {
        assert_decode_error(
            &input,
            limit(ArtifactLimitKind::ReductionEvaluations),
            Some(29),
        );
    }
}

#[test]
fn configured_limit_ties_follow_artifact_limit_order_not_section_order() {
    let event_and_reduction = replace_once(
        &with_trace(&vec![
            CANONICAL_TRACE_EVENT.to_owned();
            TRACE_EVENTS_LIMIT + 1
        ]),
        "reducer|4096|4096|budget-exhausted",
        "reducer|4097|4097|budget-exhausted",
    );
    assert_decode_error(
        &event_and_reduction,
        limit(ArtifactLimitKind::TraceEvents),
        Some(36),
    );

    let mut byte_and_event_body = trace_body_over_bytes_limit();
    byte_and_event_body.push(CANONICAL_TRACE_EVENT.to_owned());
    let all_trace_limits = replace_once(
        &with_trace(&byte_and_event_body),
        "reducer|4096|4096|budget-exhausted",
        "reducer|4097|4097|budget-exhausted",
    );
    assert_decode_error(
        &all_trace_limits,
        limit(ArtifactLimitKind::TraceBytes),
        Some(36),
    );
}

#[test]
fn malformed_private_marker_is_absent_from_debug_and_display() {
    const PRIVATE_MARKER: &str = "private-marker-do-not-leak-7f3a";
    let input = replace_once(
        canonical_structural_artifact(),
        "fault|omit-move|4",
        &format!("fault|{PRIVATE_MARKER}|4"),
    );
    let error = decode_error(&input);

    assert_eq!(error.kind(), ArtifactDecodeErrorKind::MalformedRecord);
    assert_eq!(error.line(), Some(27));
    let debug = format!("{error:?}");
    let display = error.to_string();
    assert_eq!(
        debug,
        "ArtifactDecodeError { kind: MalformedRecord, line: Some(27) }"
    );
    assert_eq!(
        display,
        "runtime oracle artifact decode failed: MalformedRecord"
    );
    assert!(!debug.contains(PRIVATE_MARKER));
    assert!(!display.contains(PRIVATE_MARKER));
}

#[test]
fn closed_grammar_precedes_earlier_numeric_canonicality() {
    const PRIVATE_MARKER: &str = "private-marker-do-not-leak-7f3a";
    let input = replace_once(
        &replace_once(canonical_structural_artifact(), "seed|0", "seed|01"),
        "fault|omit-move|4",
        &format!("fault|{PRIVATE_MARKER}|4"),
    );
    let error = decode_error(&input);

    assert_eq!(error.kind(), ArtifactDecodeErrorKind::MalformedRecord);
    assert_eq!(error.line(), Some(27));
    assert!(!format!("{error:?}").contains(PRIVATE_MARKER));
    assert!(!error.to_string().contains(PRIVATE_MARKER));
}

fn with_reduction_limits(maximum: u32, used: u32) -> Vec<u8> {
    replace_once(
        canonical_structural_artifact(),
        "reducer|4096|4096|budget-exhausted",
        &format!("reducer|{maximum}|{used}|budget-exhausted"),
    )
}

const fn limit(kind: ArtifactLimitKind) -> ArtifactDecodeErrorKind {
    ArtifactDecodeErrorKind::LimitExceeded(kind)
}
