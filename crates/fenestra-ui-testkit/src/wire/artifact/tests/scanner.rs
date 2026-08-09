use super::support::{assert_error, scan_valid};
use crate::wire::error::{ArtifactDecodeErrorKind, ArtifactLimitKind};

#[test]
fn envelope_boundaries_preserve_original_source_lines() {
    let scanned = scan_valid();

    assert_eq!(scanned.original().begin().number, 7);
    assert_eq!(scanned.original().begin().text, "original-begin|1|0|7");
    assert_eq!(record_lines(scanned.original().records()), vec![8]);
    assert_eq!(record_texts(scanned.original().records()), vec!["tx|0|0"]);
    assert_eq!(scanned.original().end().number, 9);
    assert_eq!(scanned.original().end().text, "original-end");

    assert_eq!(scanned.minimized().begin().number, 13);
    assert_eq!(scanned.minimized().begin().text, "minimized-begin|1|0|7");
    assert_eq!(record_lines(scanned.minimized().records()), vec![14]);
    assert_eq!(record_texts(scanned.minimized().records()), vec!["tx|0|0"]);
    assert_eq!(scanned.minimized().end().number, 15);
    assert_eq!(scanned.minimized().end().text, "minimized-end");

    assert_eq!(scanned.trace().begin().number, 17);
    assert_eq!(scanned.trace().begin().text, "trace-begin|1|38");
    assert_eq!(record_lines(scanned.trace().records()), vec![18]);
    assert_eq!(
        record_texts(scanned.trace().records()),
        vec!["event|0|0|0|0|0|noop|0|-|mismatch"]
    );
    assert_eq!(scanned.trace().end().number, 19);
    assert_eq!(scanned.trace().end().text, "trace-end");
}

#[test]
fn artifact_bytes_precede_ascii_validation() {
    let input = vec![0xff; 524_289];

    assert_error(
        &input,
        ArtifactDecodeErrorKind::LimitExceeded(ArtifactLimitKind::ArtifactBytes),
        None,
    );
}

#[test]
fn ascii_validation_precedes_line_length() {
    let mut input = vec![b'x'; 1_025];
    input.push(0xff);
    input.push(b'\n');

    assert_error(&input, ArtifactDecodeErrorKind::InvalidAscii, Some(1));
}

#[test]
fn line_length_precedes_line_count() {
    let mut input = vec![b'x'; 1_025];
    input.push(b'\n');
    input.extend(vec![b'\n'; 4_096]);

    assert_error(
        &input,
        ArtifactDecodeErrorKind::LimitExceeded(ArtifactLimitKind::LineBytes),
        Some(1),
    );
}

#[test]
fn line_count_precedes_record_grammar() {
    let input = vec![b'\n'; 4_097];

    assert_error(
        &input,
        ArtifactDecodeErrorKind::LimitExceeded(ArtifactLimitKind::Lines),
        Some(4_097),
    );
}

fn record_lines(records: &[crate::wire::scan::ScannedLine<'_>]) -> Vec<u32> {
    records.iter().map(|line| line.number).collect()
}

fn record_texts<'a>(records: &[crate::wire::scan::ScannedLine<'a>]) -> Vec<&'a str> {
    records.iter().map(|line| line.text).collect()
}
