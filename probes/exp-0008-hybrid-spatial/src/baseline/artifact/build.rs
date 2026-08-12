use super::super::{EvidenceRecordV2, EvidenceSectionV2, SpatialEvidenceV2, control_report_v2};
use super::{
    ArtifactCaseV2, ArtifactControlV2, ArtifactErrorKindV2, ArtifactErrorV2, ArtifactKindV2,
    ArtifactObservationV2, ArtifactSectionV2, CASE_NAMES, CONTROL_FAMILIES, OBSERVATION_COUNTS,
    SpatialEvidenceArtifactV2, token,
};

pub(crate) fn artifact_from_evidence_v2(
    evidence: &SpatialEvidenceV2,
) -> Result<SpatialEvidenceArtifactV2, ArtifactErrorV2> {
    validate_evidence(evidence)?;
    let controls = control_report_v2(evidence);
    if controls.len() != CONTROL_FAMILIES.len()
        || controls
            .iter()
            .any(|control| control.registered == 0 || control.detected != control.registered)
    {
        return Err(ArtifactErrorV2::new(ArtifactErrorKindV2::InvalidModel));
    }

    let cases = evidence
        .cases
        .iter()
        .map(|case| ArtifactCaseV2 {
            ordinal: case.ordinal,
            name: case.name.to_owned(),
            observations: case
                .observations
                .iter()
                .map(|observation| ArtifactObservationV2 {
                    case: observation.case,
                    step: observation.step,
                    generation: observation.generation,
                    viewport: observation.viewport,
                    sections: observation
                        .sections
                        .iter()
                        .map(|section| ArtifactSectionV2 {
                            name: section.name,
                            records: section.record_count,
                            bytes: section.byte_count,
                            digest: section.digest,
                        })
                        .collect(),
                })
                .collect(),
            literal_match: case.result.literal_match,
            reference_match: case.result.reference_match,
            repeat_match: case.result.repeat_match,
        })
        .collect();
    let controls = controls
        .into_iter()
        .map(|control| ArtifactControlV2 {
            family: control.family.token().to_owned(),
            registered: control.registered,
            detected: control.detected,
        })
        .collect();
    Ok(SpatialEvidenceArtifactV2 {
        kind: ArtifactKindV2::Baseline,
        candidate_count: 0,
        cases,
        controls,
    })
}

fn validate_evidence(evidence: &SpatialEvidenceV2) -> Result<(), ArtifactErrorV2> {
    if evidence.cases.len() != CASE_NAMES.len() {
        return invalid_model();
    }
    for (case_index, case) in evidence.cases.iter().enumerate() {
        if usize::from(case.ordinal) != case_index
            || case.name != CASE_NAMES[case_index]
            || !token(case.name)
            || case.observations.len() != OBSERVATION_COUNTS[case_index]
            || !case.result.literal_match
            || !case.result.reference_match
            || !case.result.repeat_match
        {
            return invalid_model();
        }
        for (step, observation) in case.observations.iter().enumerate() {
            if usize::from(observation.case) != case_index
                || usize::from(observation.step) != step
                || observation.sections.len() != EvidenceSectionV2::ALL.len()
            {
                return invalid_model();
            }
            for (section_index, section) in observation.sections.iter().enumerate() {
                if section.name != EvidenceSectionV2::ALL[section_index]
                    || section.record_count != section.records.len() as u64
                    || section.byte_count != section.encoded.len() as u64
                    || encode_records(&section.records) != section.encoded
                    || digest(section.name.token(), &section.encoded) != section.digest
                {
                    return invalid_model();
                }
            }
        }
    }
    Ok(())
}

fn encode_records(records: &[EvidenceRecordV2]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(records.len() as u64).to_le_bytes());
    for record in records {
        for field in &record.fields {
            bytes.extend_from_slice(&field.encoded);
        }
    }
    bytes
}

pub(super) fn digest(section: &str, encoded: &[u8]) -> u64 {
    let mut value = 14_695_981_039_346_656_037_u64;
    for byte in b"spatial-evidence-v2"
        .iter()
        .copied()
        .chain([0])
        .chain(section.bytes())
        .chain([0])
        .chain(encoded.iter().copied())
    {
        value ^= u64::from(byte);
        value = value.wrapping_mul(1_099_511_628_211);
    }
    value
}

fn invalid_model<T>() -> Result<T, ArtifactErrorV2> {
    Err(ArtifactErrorV2::new(ArtifactErrorKindV2::InvalidModel))
}
