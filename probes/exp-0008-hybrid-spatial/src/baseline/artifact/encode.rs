use std::fmt::Write;

use super::{
    ARTIFACT_LIMITS_V2, ArtifactErrorKindV2, ArtifactErrorV2, ArtifactKindV2, ArtifactLimitKindV2,
    CASE_NAMES, CONTROL_FAMILIES, HEADER, LIMITS, OBSERVATION_COUNTS, PACKAGES, PROFILE,
    SpatialEvidenceArtifactV2, validate_record_grammar, verify_spatial_evidence_artifact_v2,
};

pub(crate) fn encode_spatial_evidence_artifact_v2(
    artifact: &SpatialEvidenceArtifactV2,
) -> Result<Vec<u8>, ArtifactErrorV2> {
    validate_artifact(artifact)?;
    verify_spatial_evidence_artifact_v2(artifact)?;
    let lines = render_lines(artifact);
    encode_lines(lines)
}

fn validate_artifact(artifact: &SpatialEvidenceArtifactV2) -> Result<(), ArtifactErrorV2> {
    if artifact.kind != ArtifactKindV2::Baseline
        || artifact.candidate_count != 0
        || artifact.cases.len() != CASE_NAMES.len()
        || artifact.controls.len() != CONTROL_FAMILIES.len()
    {
        return invalid_model();
    }
    for (index, case) in artifact.cases.iter().enumerate() {
        if usize::from(case.ordinal) != index
            || case.name != CASE_NAMES[index]
            || case.observations.len() != OBSERVATION_COUNTS[index]
            || !case.literal_match
            || !case.reference_match
            || !case.repeat_match
        {
            return invalid_model();
        }
        for (step, observation) in case.observations.iter().enumerate() {
            if usize::from(observation.case) != index
                || usize::from(observation.step) != step
                || observation.sections.len() != 10
            {
                return invalid_model();
            }
            for (section_index, section) in observation.sections.iter().enumerate() {
                if section.name.tag() as usize != section_index {
                    return invalid_model();
                }
            }
        }
    }
    for (index, control) in artifact.controls.iter().enumerate() {
        if control.family != CONTROL_FAMILIES[index]
            || control.registered == 0
            || control.registered != control.detected
        {
            return invalid_model();
        }
    }
    Ok(())
}

fn render_lines(artifact: &SpatialEvidenceArtifactV2) -> Vec<String> {
    let mut lines = vec![
        HEADER.to_owned(),
        PACKAGES.to_owned(),
        PROFILE.to_owned(),
        LIMITS.to_owned(),
    ];
    for case in &artifact.cases {
        lines.push(format!(
            "case|ordinal={}|name={}|observations={}",
            case.ordinal,
            case.name,
            case.observations.len()
        ));
        for observation in &case.observations {
            let generation = observation
                .generation
                .map_or_else(|| "-".to_owned(), |value| value.to_string());
            lines.push(format!(
                "observation|case={}|step={}|generation={generation}|viewport={}x{}",
                observation.case, observation.step, observation.viewport.0, observation.viewport.1
            ));
            for section in &observation.sections {
                lines.push(format!(
                    "section|case={}|step={}|name={}|records={}|bytes={}|digest={:016x}",
                    observation.case,
                    observation.step,
                    section.name.token(),
                    section.records,
                    section.bytes,
                    section.digest
                ));
            }
        }
        lines.push(format!(
            "case-result|case={}|literal=match|reference=match|repeat=match",
            case.ordinal
        ));
    }
    for control in &artifact.controls {
        lines.push(format!(
            "control|family={}|registered={}|detected={}",
            control.family, control.registered, control.detected
        ));
    }
    lines.push("result|literal=pass|reference=pass|candidate-count=0".to_owned());
    lines.push("end|spatial-v2".to_owned());
    lines
}

pub(super) fn encode_lines(lines: Vec<String>) -> Result<Vec<u8>, ArtifactErrorV2> {
    if lines.len() > ARTIFACT_LIMITS_V2.records {
        return Err(ArtifactErrorV2::limit(
            ArtifactLimitKindV2::Records,
            lines.len(),
            ARTIFACT_LIMITS_V2.records,
            None,
        ));
    }
    for (record, line) in lines.iter().enumerate() {
        validate_record_grammar(line, record)?;
    }
    for (record, line) in lines.iter().enumerate() {
        if line.len() > ARTIFACT_LIMITS_V2.line_bytes {
            return Err(ArtifactErrorV2::limit(
                ArtifactLimitKindV2::LineBytes,
                line.len(),
                ARTIFACT_LIMITS_V2.line_bytes,
                Some(record),
            ));
        }
    }
    let byte_count = lines.iter().try_fold(0usize, |total, line| {
        total.checked_add(line.len().checked_add(1)?)
    });
    let Some(byte_count) = byte_count else {
        return Err(ArtifactErrorV2::new(ArtifactErrorKindV2::LimitExceeded(
            ArtifactLimitKindV2::ArtifactBytes,
        )));
    };
    if byte_count > ARTIFACT_LIMITS_V2.artifact_bytes {
        return Err(ArtifactErrorV2::limit(
            ArtifactLimitKindV2::ArtifactBytes,
            byte_count,
            ARTIFACT_LIMITS_V2.artifact_bytes,
            None,
        ));
    }
    let mut bytes = String::with_capacity(byte_count);
    for line in lines {
        let _ = writeln!(bytes, "{line}");
    }
    Ok(bytes.into_bytes())
}

fn invalid_model<T>() -> Result<T, ArtifactErrorV2> {
    Err(ArtifactErrorV2::new(ArtifactErrorKindV2::InvalidModel))
}
