use super::super::decode_failure_artifact_v1;
use super::support::{canonical_structural_artifact, decode_error, replace_once};
use crate::wire::error::ArtifactDecodeErrorKind;

const ORIGINAL_FAILURE: &str =
    "failure|original|2|-|state-mismatch|fragment:root/r:1|keyed-order|keys:9,7,8|keys:7,8,9";
const PRIVATE: &str = "private-trailing-fingerprint-marker";

#[test]
fn individually_legal_fingerprints_may_differ_until_verification() {
    let input = replace_once(
        canonical_structural_artifact(),
        "failure|minimized|2|-|state-mismatch|fragment:root/r:1|keyed-order|keys:9,7,8|keys:7,8,9",
        "failure|minimized|2|-|state-mismatch|fragment:root/r:1|keyed-order|keys:9,7,8|keys:8,7,9",
    );

    assert!(decode_failure_artifact_v1(&input).is_ok());
}

#[test]
fn illegal_fingerprint_precedes_private_trailing_data() {
    let invalid = replace_once(
        canonical_structural_artifact(),
        ORIGINAL_FAILURE,
        "failure|original|2|-|state-mismatch|node:root|fragment-binding|binding:present|binding:absent",
    );
    let input = with_private_trailing(invalid);

    assert_private_error(&input, ArtifactDecodeErrorKind::InvalidFingerprint, 28);
}

#[test]
fn trailing_data_is_checked_last_and_remains_private() {
    let input = with_private_trailing(canonical_structural_artifact().to_vec());

    assert_private_error(&input, ArtifactDecodeErrorKind::TrailingData, 40);
}

fn with_private_trailing(mut input: Vec<u8>) -> Vec<u8> {
    input.extend_from_slice(format!("{PRIVATE}\n").as_bytes());
    input
}

fn assert_private_error(input: &[u8], kind: ArtifactDecodeErrorKind, line: u32) {
    let error = decode_error(input);
    assert_eq!(error.kind(), kind);
    assert_eq!(error.line(), Some(line));
    assert!(!format!("{error:?}").contains(PRIVATE));
    assert!(!error.to_string().contains(PRIVATE));
}
