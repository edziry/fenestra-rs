use super::{ARTIFACT_BYTES, LINE_BYTES, LINES, LineSinkV1, OutputMeasurementV1, accounted_bytes};
use crate::headless::artifact::error::{
    HeadlessArtifactEncodeErrorKindV1, HeadlessArtifactEncodeErrorV1,
};

#[test]
fn output_limits_are_inclusive_and_reject_one_over() {
    assert_eq!(
        measurement(ARTIFACT_BYTES, LINE_BYTES, LINES).validate(),
        Ok(())
    );
    assert_kind(
        measurement(ARTIFACT_BYTES + 1, LINE_BYTES, LINES).validate(),
        HeadlessArtifactEncodeErrorKindV1::ArtifactBytes,
    );
    assert_kind(
        measurement(ARTIFACT_BYTES, LINE_BYTES + 1, LINES).validate(),
        HeadlessArtifactEncodeErrorKindV1::LineBytes,
    );
    assert_kind(
        measurement(ARTIFACT_BYTES, LINE_BYTES, LINES + 1).validate(),
        HeadlessArtifactEncodeErrorKindV1::Lines,
    );
}

#[test]
fn simultaneous_output_failures_follow_wire_priority() {
    assert_kind(
        measurement(ARTIFACT_BYTES + 1, LINE_BYTES + 1, LINES + 1).validate(),
        HeadlessArtifactEncodeErrorKindV1::ArtifactBytes,
    );
    assert_kind(
        measurement(ARTIFACT_BYTES, LINE_BYTES + 1, LINES + 1).validate(),
        HeadlessArtifactEncodeErrorKindV1::LineBytes,
    );
    assert_kind(
        measurement(ARTIFACT_BYTES, LINE_BYTES, LINES + 1).validate(),
        HeadlessArtifactEncodeErrorKindV1::Lines,
    );
}

#[test]
fn checked_accounting_and_measurement_preserve_the_accepted_prefix() {
    assert_eq!(accounted_bytes(128, 160), Ok(20_480));
    assert_kind(
        accounted_bytes(usize::MAX, 160),
        HeadlessArtifactEncodeErrorKindV1::ArtifactBytes,
    );

    let mut byte_overflow = measurement(usize::MAX, 7, 2);
    let accepted = byte_overflow;
    assert_kind(
        byte_overflow.push_line("x"),
        HeadlessArtifactEncodeErrorKindV1::ArtifactBytes,
    );
    assert_eq!(byte_overflow, accepted);

    let mut line_overflow = measurement(0, 0, usize::MAX);
    let accepted = line_overflow;
    assert_kind(
        line_overflow.push_line(""),
        HeadlessArtifactEncodeErrorKindV1::ArtifactBytes,
    );
    assert_eq!(line_overflow, accepted);
}

#[test]
fn encode_errors_disclose_only_the_closed_kind() {
    for kind in [
        HeadlessArtifactEncodeErrorKindV1::ArtifactBytes,
        HeadlessArtifactEncodeErrorKindV1::LineBytes,
        HeadlessArtifactEncodeErrorKindV1::Lines,
    ] {
        let error = HeadlessArtifactEncodeErrorV1::new(kind);
        assert_eq!(error.kind(), kind);
        assert_eq!(
            format!("{error:?}"),
            format!("HeadlessArtifactEncodeErrorV1 {{ kind: {kind:?} }}")
        );
        assert_eq!(
            error.to_string(),
            format!("headless artifact encode failed: {kind:?}")
        );
    }
}

const fn measurement(
    artifact_bytes: usize,
    line_bytes: usize,
    lines: usize,
) -> OutputMeasurementV1 {
    OutputMeasurementV1 {
        artifact_bytes,
        line_bytes,
        lines,
    }
}

fn assert_kind<T>(
    result: Result<T, HeadlessArtifactEncodeErrorV1>,
    expected: HeadlessArtifactEncodeErrorKindV1,
) {
    assert_eq!(
        result.err().map(HeadlessArtifactEncodeErrorV1::kind),
        Some(expected)
    );
}
