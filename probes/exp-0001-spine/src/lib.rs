#![forbid(unsafe_code)]

//! Deterministic artifact seam for the disposable EXP-0001 headless probe.

use std::error::Error;
use std::fmt;

use fenestra_ui_testkit::prototype::{
    HeadlessArtifactEncodeErrorKindV1, HeadlessArtifactV1, HeadlessArtifactVerificationErrorKindV1,
    HeadlessFailureCauseV1, HeadlessRunV1, build_headless_artifact_v1, encode_headless_artifact_v1,
    run_headless_spine_v1, verify_headless_artifact_v1,
};

/// Closed failures from producing the canonical headless probe artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessProbeErrorV1 {
    /// The fixed testkit runner failed before producing evidence.
    Runner(HeadlessFailureCauseV1),
    /// Semantic verification rejected the freshly built artifact.
    Verification {
        /// Closed semantic mismatch class.
        kind: HeadlessArtifactVerificationErrorKindV1,
        /// First differing trace or projection index when available.
        index: Option<usize>,
    },
    /// Canonical bounded encoding failed.
    Encoding(HeadlessArtifactEncodeErrorKindV1),
}

impl fmt::Display for HeadlessProbeErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "headless probe failed: {self:?}")
    }
}

impl Error for HeadlessProbeErrorV1 {}

/// Runs, verifies, and canonically encodes the fixed headless spine.
pub fn run_headless_probe_v1() -> Result<Vec<u8>, HeadlessProbeErrorV1> {
    produce_artifact_v1(
        || run_headless_spine_v1().map_err(|error| error.kind()),
        |artifact| {
            verify_headless_artifact_v1(artifact).map_err(|error| VerificationFailureV1 {
                kind: error.kind(),
                index: error.index(),
            })
        },
        |artifact| encode_headless_artifact_v1(artifact).map_err(|error| error.kind()),
    )
}

fn produce_artifact_v1(
    runner: impl FnOnce() -> Result<HeadlessRunV1, HeadlessFailureCauseV1>,
    verifier: impl FnOnce(&HeadlessArtifactV1) -> Result<(), VerificationFailureV1>,
    encoder: impl FnOnce(&HeadlessArtifactV1) -> Result<Vec<u8>, HeadlessArtifactEncodeErrorKindV1>,
) -> Result<Vec<u8>, HeadlessProbeErrorV1> {
    let run = runner().map_err(HeadlessProbeErrorV1::Runner)?;
    let artifact = build_headless_artifact_v1(&run);
    verifier(&artifact).map_err(|error| HeadlessProbeErrorV1::Verification {
        kind: error.kind,
        index: error.index,
    })?;
    encoder(&artifact).map_err(HeadlessProbeErrorV1::Encoding)
}

#[derive(Clone, Copy)]
struct VerificationFailureV1 {
    kind: HeadlessArtifactVerificationErrorKindV1,
    index: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runner_failure_maps_first_and_skips_later_stages() {
        let error = produce_artifact_v1(
            || Err(HeadlessFailureCauseV1::Trace),
            |_| panic!("verification must not run after runner failure"),
            |_| panic!("encoding must not run after runner failure"),
        )
        .expect_err("injected runner failure should be preserved");

        assert_eq!(
            error,
            HeadlessProbeErrorV1::Runner(HeadlessFailureCauseV1::Trace)
        );
    }

    #[test]
    fn verification_failure_preserves_kind_and_index_before_encoding() {
        let error = produce_artifact_v1(
            fresh_run,
            |_| {
                Err(VerificationFailureV1 {
                    kind: HeadlessArtifactVerificationErrorKindV1::HeadlessTraceMismatch,
                    index: Some(7),
                })
            },
            |_| panic!("encoding must not run after verification failure"),
        )
        .expect_err("injected verification failure should be preserved");

        assert_eq!(
            error,
            HeadlessProbeErrorV1::Verification {
                kind: HeadlessArtifactVerificationErrorKindV1::HeadlessTraceMismatch,
                index: Some(7),
            }
        );
    }

    #[test]
    fn encoding_failure_maps_after_successful_verification() {
        let error = produce_artifact_v1(
            fresh_run,
            |_| Ok(()),
            |_| Err(HeadlessArtifactEncodeErrorKindV1::LineBytes),
        )
        .expect_err("injected encoding failure should be preserved");

        assert_eq!(
            error,
            HeadlessProbeErrorV1::Encoding(HeadlessArtifactEncodeErrorKindV1::LineBytes)
        );
    }

    fn fresh_run() -> Result<HeadlessRunV1, HeadlessFailureCauseV1> {
        run_headless_spine_v1().map_err(|error| error.kind())
    }
}
