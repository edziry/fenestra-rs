#[path = "headless/artifact_decode_support.rs"]
mod support;

use fenestra_ui_testkit::prototype::{
    HeadlessArtifactDecodeErrorKindV1 as Kind, HeadlessArtifactDecodeErrorV1,
    HeadlessArtifactLimitKindV1 as Limit,
};

const ARTIFACT_BYTES: usize = 65_536;
const LINE_BYTES: usize = 1_024;
const LINES: usize = 512;

#[test]
fn scanner_priority_is_artifact_bytes_ascii_line_bytes_lines_then_final_lf() {
    let bytes_and_ascii = vec![0xff; ARTIFACT_BYTES + 1];
    support::assert_decode_error(&bytes_and_ascii, limit(Limit::ArtifactBytes), None);

    let mut ascii_and_line = vec![b'x'; LINE_BYTES + 1];
    ascii_and_line.push(0xff);
    ascii_and_line.push(b'\n');
    support::assert_decode_error(&ascii_and_line, Kind::InvalidAscii, Some(1));

    let mut line_and_lines = vec![b'x'; LINE_BYTES + 1];
    line_and_lines.push(b'\n');
    line_and_lines.extend(std::iter::repeat_n(b'\n', LINES));
    support::assert_decode_error(&line_and_lines, limit(Limit::LineBytes), Some(1));

    let too_many_lines = vec![b'\n'; LINES + 1];
    support::assert_decode_error(&too_many_lines, limit(Limit::Lines), Some(513));

    let mut no_final_lf = support::replace_once(
        &support::canonical_bytes(),
        "fenestra-headless-spine|1",
        "fenestra-headless-spine|2",
    );
    assert_eq!(no_final_lf.pop(), Some(b'\n'));
    support::assert_decode_error(&no_final_lf, Kind::MissingFinalLineFeed, Some(144));
}

#[test]
fn scanner_limits_are_inclusive_before_grammar_validation() {
    let exact_artifact_bytes = vec![b'\n'; ARTIFACT_BYTES];
    let error = support::decode_error(&exact_artifact_bytes);
    assert_ne!(error.kind(), limit(Limit::ArtifactBytes));

    let mut exact_line = vec![b'x'; LINE_BYTES];
    exact_line.push(b'\n');
    let error = support::decode_error(&exact_line);
    assert_ne!(error.kind(), limit(Limit::LineBytes));

    let exact_lines = vec![b'\n'; LINES];
    let error = support::decode_error(&exact_lines);
    assert_ne!(error.kind(), limit(Limit::Lines));

    let mut over_line = vec![b'x'; LINE_BYTES + 1];
    over_line.push(b'\n');
    support::assert_decode_error(&over_line, limit(Limit::LineBytes), Some(1));
}

#[test]
fn decode_error_is_typed_line_anchored_and_payload_free() {
    let error: HeadlessArtifactDecodeErrorV1 = support::decode_error(&[0xff, b'\n']);
    assert_eq!(error.kind(), Kind::InvalidAscii);
    assert_eq!(error.line(), Some(1));
    assert_eq!(
        format!("{error:?}"),
        "HeadlessArtifactDecodeErrorV1 { kind: InvalidAscii, line: Some(1) }"
    );
    assert_eq!(
        error.to_string(),
        "headless artifact decode failed: InvalidAscii"
    );
    assert!(!format!("{error:?}").contains("ff"));
    assert_eq!(kind_discriminant(error.kind()), 1);
}

const fn limit(limit: Limit) -> Kind {
    Kind::LimitExceeded(limit)
}

const fn kind_discriminant(kind: Kind) -> u8 {
    match kind {
        Kind::LimitExceeded(_) => 0,
        Kind::InvalidAscii => 1,
        Kind::MissingFinalLineFeed => 2,
        Kind::MalformedRecord => 3,
        Kind::UnsupportedVersion(_) => 4,
        Kind::NonCanonicalValue => 5,
        Kind::MissingSection(_) => 6,
        Kind::DuplicateSection(_) => 7,
        Kind::OrderingViolation => 8,
        Kind::CountMismatch(_) => 9,
        Kind::InvalidReference => 10,
        Kind::TrailingData => 11,
    }
}
