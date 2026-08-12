use super::{ARTIFACT_LIMITS_V2, ArtifactErrorV2, ArtifactLimitKindV2, SPATIAL_LIMITS};

pub(crate) const fn registered_spatial_limits_v2() -> [usize; 30] {
    SPATIAL_LIMITS
}

pub(crate) fn artifact_limit_probe_v2(
    kind: ArtifactLimitKindV2,
    observed: usize,
) -> Result<(), ArtifactErrorV2> {
    let maximum = match kind {
        ArtifactLimitKindV2::Records => ARTIFACT_LIMITS_V2.records,
        ArtifactLimitKindV2::LineBytes => ARTIFACT_LIMITS_V2.line_bytes,
        ArtifactLimitKindV2::ArtifactBytes => ARTIFACT_LIMITS_V2.artifact_bytes,
    };
    if observed <= maximum {
        Ok(())
    } else {
        Err(ArtifactErrorV2::limit(kind, observed, maximum, None))
    }
}
