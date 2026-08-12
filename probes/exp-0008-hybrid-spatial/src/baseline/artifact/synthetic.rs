use super::{ARTIFACT_LIMITS_V2, ArtifactErrorKindV2, ArtifactErrorV2, ArtifactLimitKindV2};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArtifactSyntheticFaultV2 {
    InvalidModel,
    Records,
    Grammar,
    LineBytes,
    ArtifactBytes,
    GrammarAt(usize),
    LineAt(usize),
}

pub(crate) fn encode_fault_fixture_v2(
    faults: &[ArtifactSyntheticFaultV2],
) -> Result<Vec<u8>, ArtifactErrorV2> {
    if faults.contains(&ArtifactSyntheticFaultV2::InvalidModel) {
        return Err(ArtifactErrorV2::new(ArtifactErrorKindV2::InvalidModel));
    }
    if faults.contains(&ArtifactSyntheticFaultV2::Records) {
        return Err(ArtifactErrorV2::limit(
            ArtifactLimitKindV2::Records,
            ARTIFACT_LIMITS_V2.records + 1,
            ARTIFACT_LIMITS_V2.records,
            None,
        ));
    }
    if let Some(record) = first_record(faults, FaultClass::Grammar) {
        return Err(ArtifactErrorV2::at(
            ArtifactErrorKindV2::InvalidGrammar,
            record,
        ));
    }
    if let Some(record) = first_record(faults, FaultClass::Line) {
        return Err(ArtifactErrorV2::limit(
            ArtifactLimitKindV2::LineBytes,
            ARTIFACT_LIMITS_V2.line_bytes + 1,
            ARTIFACT_LIMITS_V2.line_bytes,
            Some(record),
        ));
    }
    if faults.contains(&ArtifactSyntheticFaultV2::ArtifactBytes) {
        return Err(ArtifactErrorV2::limit(
            ArtifactLimitKindV2::ArtifactBytes,
            ARTIFACT_LIMITS_V2.artifact_bytes + 1,
            ARTIFACT_LIMITS_V2.artifact_bytes,
            None,
        ));
    }
    Ok(b"end|spatial-v2\n".to_vec())
}

#[derive(Clone, Copy)]
enum FaultClass {
    Grammar,
    Line,
}

fn first_record(faults: &[ArtifactSyntheticFaultV2], class: FaultClass) -> Option<usize> {
    faults
        .iter()
        .filter_map(|fault| match (class, *fault) {
            (FaultClass::Grammar, ArtifactSyntheticFaultV2::Grammar) => Some(0),
            (FaultClass::Grammar, ArtifactSyntheticFaultV2::GrammarAt(record)) => Some(record),
            (FaultClass::Line, ArtifactSyntheticFaultV2::LineBytes) => Some(0),
            (FaultClass::Line, ArtifactSyntheticFaultV2::LineAt(record)) => Some(record),
            _ => None,
        })
        .min()
}
