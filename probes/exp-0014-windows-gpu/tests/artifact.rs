use fenestra_ui_exp_0014_windows_gpu::{
    ARTIFACT_LIMITS_V1, InteractiveArtifactErrorKindV1, InteractiveResultV1,
    verify_interactive_artifact_v1,
};

mod support;

use support::valid_pass_artifact;

#[test]
fn exact_complete_windows_artifact_verifies() {
    let bytes = valid_pass_artifact();
    let verified = verify_interactive_artifact_v1(&bytes).expect("valid pass artifact");

    assert_eq!(verified.result(), InteractiveResultV1::Pass);
    assert_eq!(verified.record_count(), 16);
    assert_eq!(verified.byte_count(), bytes.len());
    assert_eq!(verified.last_generation(), Some(2));
}

#[test]
fn artifact_bounds_are_exact_and_checked_before_grammar() {
    assert_eq!(ARTIFACT_LIMITS_V1.records(), 256);
    assert_eq!(ARTIFACT_LIMITS_V1.line_bytes(), 512);
    assert_eq!(ARTIFACT_LIMITS_V1.artifact_bytes(), 65_536);

    let mut too_many = Vec::new();
    for _ in 0..257 {
        too_many.extend_from_slice(b"x\n");
    }
    assert_eq!(
        verify_interactive_artifact_v1(&too_many).expect_err("record bound"),
        InteractiveArtifactErrorKindV1::Bounds
    );

    let mut long_line = vec![b'x'; 513];
    long_line.push(b'\n');
    assert_eq!(
        verify_interactive_artifact_v1(&long_line).expect_err("line bound"),
        InteractiveArtifactErrorKindV1::Bounds
    );
}

#[test]
fn encoding_and_privacy_failures_are_closed() {
    let mut non_ascii = valid_pass_artifact();
    non_ascii[0] = 0xff;
    assert_eq!(
        verify_interactive_artifact_v1(&non_ascii).expect_err("ASCII only"),
        InteractiveArtifactErrorKindV1::Encoding
    );

    let private = String::from_utf8(valid_pass_artifact())
        .expect("fixture ASCII")
        .replace("|profile=release", "|home=/users/alice|profile=release");
    assert_eq!(
        verify_interactive_artifact_v1(private.as_bytes()).expect_err("private key"),
        InteractiveArtifactErrorKindV1::Redaction
    );
}

#[test]
fn target_backend_coherence_is_verified() {
    let incoherent = String::from_utf8(valid_pass_artifact())
        .expect("fixture ASCII")
        .replace("adapter|backend=dx12", "adapter|backend=vulkan");
    assert_eq!(
        verify_interactive_artifact_v1(incoherent.as_bytes()).expect_err("backend mismatch"),
        InteractiveArtifactErrorKindV1::Coherence
    );
}

#[test]
fn artifact_replays_generation_and_terminal_rules() {
    let stale = String::from_utf8(valid_pass_artifact())
        .expect("fixture ASCII")
        .replace(
            "mutation-present|generation=1",
            "mutation-present|generation=0",
        );
    assert_eq!(
        verify_interactive_artifact_v1(stale.as_bytes()).expect_err("stale mutation"),
        InteractiveArtifactErrorKindV1::Protocol
    );

    let incomplete = String::from_utf8(valid_pass_artifact())
        .expect("fixture ASCII")
        .replace("event|milestone=close\n", "");
    assert_eq!(
        verify_interactive_artifact_v1(incomplete.as_bytes()).expect_err("incomplete pass"),
        InteractiveArtifactErrorKindV1::Terminal
    );
}
