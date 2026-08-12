use super::super::reconstruct_literal_v2;
use super::{
    ArtifactErrorKindV2, ArtifactErrorV2, SpatialEvidenceArtifactV2, artifact_from_evidence_v2,
};

pub(crate) fn verify_spatial_evidence_artifact_v2(
    artifact: &SpatialEvidenceArtifactV2,
) -> Result<(), ArtifactErrorV2> {
    let evidence = reconstruct_literal_v2()
        .map_err(|_| ArtifactErrorV2::new(ArtifactErrorKindV2::InvalidModel))?;
    let expected = artifact_from_evidence_v2(&evidence)?;
    if artifact.kind != expected.kind || artifact.candidate_count != expected.candidate_count {
        return invalid(ArtifactErrorKindV2::InvalidModel);
    }
    if artifact.cases.len() != expected.cases.len()
        || artifact.controls.len() != expected.controls.len()
    {
        return invalid(ArtifactErrorKindV2::InvalidCount);
    }
    for (actual_case, expected_case) in artifact.cases.iter().zip(&expected.cases) {
        if actual_case.ordinal != expected_case.ordinal || actual_case.name != expected_case.name {
            return invalid(ArtifactErrorKindV2::InvalidOrder);
        }
        if actual_case.observations.len() != expected_case.observations.len() {
            return invalid(ArtifactErrorKindV2::InvalidCount);
        }
        for (actual, expected) in actual_case
            .observations
            .iter()
            .zip(&expected_case.observations)
        {
            if actual.case != expected.case
                || actual.step != expected.step
                || actual.generation != expected.generation
                || actual.viewport != expected.viewport
            {
                return invalid(ArtifactErrorKindV2::InvalidReference);
            }
            if actual.sections.len() != expected.sections.len() {
                return invalid(ArtifactErrorKindV2::InvalidCount);
            }
            for (actual, expected) in actual.sections.iter().zip(&expected.sections) {
                if actual.name != expected.name {
                    return invalid(ArtifactErrorKindV2::InvalidOrder);
                }
                if actual.records != expected.records || actual.bytes != expected.bytes {
                    return invalid(ArtifactErrorKindV2::InvalidCount);
                }
                if actual.digest != expected.digest {
                    return invalid(ArtifactErrorKindV2::DigestMismatch);
                }
            }
        }
        if actual_case.literal_match != expected_case.literal_match
            || actual_case.reference_match != expected_case.reference_match
            || actual_case.repeat_match != expected_case.repeat_match
        {
            return invalid(ArtifactErrorKindV2::InvalidModel);
        }
    }
    if artifact.controls != expected.controls {
        return invalid(ArtifactErrorKindV2::InvalidCount);
    }
    Ok(())
}

fn invalid<T>(kind: ArtifactErrorKindV2) -> Result<T, ArtifactErrorV2> {
    Err(ArtifactErrorV2::new(kind))
}
