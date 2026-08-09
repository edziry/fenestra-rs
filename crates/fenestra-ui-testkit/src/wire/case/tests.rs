use crate::case::{GeneratorConfigV1, SeedV1};

use super::{CaseDecodeContextV1, decode_case_v1};
use crate::wire::error::{ArtifactDecodeErrorKind, ArtifactLimitKind};

#[test]
fn artifact_bytes_precede_ascii_scanning() {
    let input = vec![0xff; 524_289];

    let error = decode_case_v1(&input, context()).expect_err("artifact bytes should win");

    assert_eq!(
        error.kind(),
        ArtifactDecodeErrorKind::LimitExceeded(ArtifactLimitKind::ArtifactBytes)
    );
    assert_eq!(error.line(), None);
}

#[test]
fn line_count_precedes_record_grammar() {
    let input = vec![b'\n'; 4_097];

    let error = decode_case_v1(&input, context()).expect_err("line count should win");

    assert_eq!(
        error.kind(),
        ArtifactDecodeErrorKind::LimitExceeded(ArtifactLimitKind::Lines)
    );
    assert_eq!(error.line(), Some(4_097));
}

#[test]
fn ascii_validation_precedes_line_length() {
    let mut input = vec![b'x'; 1_025];
    input.push(0xff);
    input.push(b'\n');

    let error = decode_case_v1(&input, context()).expect_err("ASCII should win");

    assert_eq!(error.kind(), ArtifactDecodeErrorKind::InvalidAscii);
    assert_eq!(error.line(), Some(1));
}

fn context() -> CaseDecodeContextV1 {
    CaseDecodeContextV1::new(1, GeneratorConfigV1::new(8, 2, 8), SeedV1::new(0))
}
