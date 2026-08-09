#[path = "support/mod.rs"]
mod support;

use fenestra_ui_exp_0001_spine::{HeadlessProbeErrorV1, run_headless_probe_v1};
use fenestra_ui_testkit::prototype::{
    HeadlessArtifactEncodeErrorKindV1, HeadlessArtifactVerificationErrorKindV1,
    HeadlessFailureCauseV1, decode_headless_artifact_v1, encode_headless_artifact_v1,
    verify_headless_artifact_v1,
};

#[test]
fn versioned_golden_is_canonical_bounded_ascii() {
    assert!(
        support::GOLDEN
            .iter()
            .all(|byte| *byte == b'\n' || (0x20..=0x7e).contains(byte))
    );
    assert_eq!(support::GOLDEN.len(), 11_227);
    assert_eq!(
        support::GOLDEN
            .iter()
            .filter(|byte| **byte == b'\n')
            .count(),
        144
    );
    assert_eq!(support::GOLDEN.last(), Some(&b'\n'));
    assert!(support::GOLDEN.len() <= 65_536);
    assert!(
        support::GOLDEN
            .split(|byte| *byte == b'\n')
            .all(|line| line.len() <= 1_024)
    );

    let artifact = decode_headless_artifact_v1(support::GOLDEN)
        .expect("the versioned golden should decode exactly");
    assert_eq!(
        encode_headless_artifact_v1(&artifact)
            .expect("the decoded golden should encode canonically"),
        support::GOLDEN
    );
    verify_headless_artifact_v1(&artifact).expect("the versioned golden should verify");
}

#[test]
fn two_fresh_probe_runs_equal_the_versioned_golden() {
    let first = fresh_bytes();
    let second = fresh_bytes();
    assert_eq!(first, support::GOLDEN);
    assert_eq!(second, support::GOLDEN);
    assert_eq!(first, second);
}

#[test]
fn probe_error_vocabulary_is_closed_bounded_and_privacy_safe() {
    let errors = [
        HeadlessProbeErrorV1::Runner(HeadlessFailureCauseV1::Runtime),
        HeadlessProbeErrorV1::Verification {
            kind: HeadlessArtifactVerificationErrorKindV1::HeadlessTraceMismatch,
            index: Some(7),
        },
        HeadlessProbeErrorV1::Encoding(HeadlessArtifactEncodeErrorKindV1::ArtifactBytes),
    ];
    for (rank, error) in errors.into_iter().enumerate() {
        assert_eq!(error_rank(error), rank);
    }
    assert_error_traits::<HeadlessProbeErrorV1>();
    assert_eq!(format!("{:?}", errors[0]), "Runner(Runtime)");
    assert_eq!(
        format!("{:?}", errors[1]),
        "Verification { kind: HeadlessTraceMismatch, index: Some(7) }"
    );
    assert_eq!(format!("{:?}", errors[2]), "Encoding(ArtifactBytes)");
    assert_eq!(
        errors[0].to_string(),
        "headless probe failed: Runner(Runtime)"
    );
    assert_eq!(
        errors[1].to_string(),
        "headless probe failed: Verification { kind: HeadlessTraceMismatch, index: Some(7) }"
    );
    assert_eq!(
        errors[2].to_string(),
        "headless probe failed: Encoding(ArtifactBytes)"
    );
}

fn error_rank(error: HeadlessProbeErrorV1) -> usize {
    match error {
        HeadlessProbeErrorV1::Runner(_) => 0,
        HeadlessProbeErrorV1::Verification { .. } => 1,
        HeadlessProbeErrorV1::Encoding(_) => 2,
    }
}

fn assert_error_traits<T: Copy + Eq + std::error::Error>() {}

fn fresh_bytes() -> Vec<u8> {
    let run: fn() -> Result<Vec<u8>, HeadlessProbeErrorV1> = run_headless_probe_v1;
    run().expect("the fixed headless probe should complete")
}
