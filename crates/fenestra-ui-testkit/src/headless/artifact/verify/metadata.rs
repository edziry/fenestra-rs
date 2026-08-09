use super::super::error::{
    HeadlessArtifactCapacityKindV1 as CapacityKind,
    HeadlessArtifactVerificationErrorKindV1 as Kind, HeadlessArtifactVerificationErrorV1 as Error,
};
use super::super::model::{ArtifactCapacitiesV1, ArtifactMetadataV1, HeadlessArtifactV1};

pub(super) fn verify_registered_v1(artifact: &HeadlessArtifactV1) -> Result<(), Error> {
    if artifact.metadata != ArtifactMetadataV1::REGISTERED {
        return Err(Error::new(Kind::FixtureMismatch));
    }
    let expected = ArtifactCapacitiesV1::REGISTERED;
    for (kind, matches) in [
        (CapacityKind::Ir, artifact.capacities.ir == expected.ir),
        (
            CapacityKind::Style,
            artifact.capacities.style == expected.style,
        ),
        (
            CapacityKind::Runtime,
            artifact.capacities.runtime == expected.runtime,
        ),
        (
            CapacityKind::Projection,
            artifact.capacities.projection == expected.projection,
        ),
        (
            CapacityKind::Scheduler,
            artifact.capacities.scheduler == expected.scheduler,
        ),
        (
            CapacityKind::Renderer,
            artifact.capacities.renderer == expected.renderer,
        ),
        (
            CapacityKind::SchedulerTrace,
            artifact.capacities.scheduler_trace == expected.scheduler_trace,
        ),
        (
            CapacityKind::HeadlessTrace,
            artifact.capacities.headless_trace == expected.headless_trace,
        ),
        (
            CapacityKind::Artifact,
            artifact.capacities.artifact == expected.artifact,
        ),
    ] {
        if !matches {
            return Err(Error::new(Kind::CapacityMismatch(kind)));
        }
    }
    Ok(())
}
