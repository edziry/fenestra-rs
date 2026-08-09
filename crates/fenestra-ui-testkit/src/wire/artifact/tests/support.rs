use super::super::{EnvelopeBoundariesV1, decode_failure_artifact_v1, scan_envelope_v1};
use crate::wire::error::{ArtifactDecodeError, ArtifactDecodeErrorKind};

pub(super) const HEADER: &str = "fenestra-oracle-failure|1";
pub(super) const VERSIONS: &str =
    "versions|fixture|1|generator|1|case|1|state|1|trace|1|fingerprint|1|reducer|1";
pub(super) const FIXTURE: &str = "fixture|runtime-oracle|1|1|5001|1|1";
pub(super) const CANONICAL_TRACE_EVENT: &str =
    "event|0|2|3,4|0|1|commit|1|structure,layout,paint|mismatch";

pub(super) fn canonical_structural_artifact() -> &'static [u8] {
    include_bytes!("../../../../tests/fixtures/canonical_structural_failure_v1.txt")
}

pub(super) fn valid_envelope() -> &'static [u8] {
    concat!(
        "fenestra-oracle-failure|1\n",
        "versions|fixture|1|generator|1|case|1|state|1|trace|1|fingerprint|1|reducer|1\n",
        "fixture|runtime-oracle|1|1|5001|1|1\n",
        "replay|4|64|256|128|1024|1\n",
        "generator|8|2|8\n",
        "seed|0\n",
        "original-begin|1|0|7\n",
        "tx|0|0\n",
        "original-end\n",
        "fault|omit-move|4\n",
        "failure|original|0|-|state-mismatch|global|node-count|count:1|count:2\n",
        "reducer|1|1|budget-exhausted\n",
        "minimized-begin|1|0|7\n",
        "tx|0|0\n",
        "minimized-end\n",
        "failure|minimized|0|-|state-mismatch|global|node-count|count:1|count:2\n",
        "trace-begin|1|38\n",
        "event|0|0|0|0|0|noop|0|-|mismatch\n",
        "trace-end\n",
        "end\n",
    )
    .as_bytes()
}

pub(super) fn scan_valid() -> EnvelopeBoundariesV1<'static> {
    scan_envelope_v1(valid_envelope()).expect("canonical envelope shape should scan")
}

pub(super) fn records(lines: &[&str]) -> Vec<u8> {
    let mut bytes = lines.join("\n").into_bytes();
    bytes.push(b'\n');
    bytes
}

pub(super) fn replace_once(input: &[u8], before: &str, after: &str) -> Vec<u8> {
    let input = std::str::from_utf8(input).expect("test artifact should be ASCII");
    assert_eq!(
        input.match_indices(before).count(),
        1,
        "controlled mutation must have exactly one target"
    );
    input.replacen(before, after, 1).into_bytes()
}

pub(super) fn decode_error(input: &[u8]) -> ArtifactDecodeError {
    match decode_failure_artifact_v1(input) {
        Ok(_) => panic!("invalid artifact should not decode"),
        Err(error) => error,
    }
}

pub(super) fn assert_decode_error(input: &[u8], kind: ArtifactDecodeErrorKind, line: Option<u32>) {
    let error = decode_error(input);
    assert_eq!(error.kind(), kind);
    assert_eq!(error.line(), line);
}

pub(super) fn assert_error(input: &[u8], kind: ArtifactDecodeErrorKind, line: Option<u32>) {
    let result: Result<EnvelopeBoundariesV1<'_>, ArtifactDecodeError> = scan_envelope_v1(input);
    let Err(error) = result else {
        panic!("invalid envelope should not scan");
    };
    assert_eq!(error.kind(), kind);
    assert_eq!(error.line(), line);
}
