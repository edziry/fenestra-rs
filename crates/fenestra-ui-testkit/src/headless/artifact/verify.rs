mod metadata;
mod projection;
mod trace;

#[cfg(test)]
mod tests;

use super::error::{
    HeadlessArtifactVerificationErrorKindV1 as Kind, HeadlessArtifactVerificationErrorV1 as Error,
};
use super::model::{HeadlessArtifactV1, build_headless_artifact_v1};
use crate::headless::runner::{HeadlessRunV1, run_headless_spine_v1};
use crate::headless::trace::HeadlessFailureCauseV1;

/// Replays the fixed spine and verifies decoded evidence field by field.
pub fn verify_headless_artifact_v1(artifact: &HeadlessArtifactV1) -> Result<(), Error> {
    verify_with_fresh_v1(artifact, || {
        run_headless_spine_v1().map_err(|error| error.kind())
    })
}

fn verify_with_fresh_v1(
    artifact: &HeadlessArtifactV1,
    fresh: impl FnOnce() -> Result<HeadlessRunV1, HeadlessFailureCauseV1>,
) -> Result<(), Error> {
    metadata::verify_registered_v1(artifact)?;
    let run = fresh().map_err(|cause| Error::new(Kind::ReplayFailed(cause)))?;
    let expected = build_headless_artifact_v1(&run);

    if artifact.result != expected.result {
        return Err(Error::new(Kind::ResultMismatch));
    }
    if artifact.final_generation != expected.final_generation {
        return Err(Error::new(Kind::FinalGenerationMismatch));
    }
    if artifact.projection.surface != expected.projection.surface {
        return Err(Error::new(Kind::SurfaceMismatch));
    }
    trace::verify_traces_v1(artifact, &expected)?;
    projection::verify_projection_v1(artifact, &expected)
}
