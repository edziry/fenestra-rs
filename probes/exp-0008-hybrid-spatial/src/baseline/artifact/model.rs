use super::super::EvidenceSectionV2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArtifactKindV2 {
    Baseline,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SpatialEvidenceArtifactV2 {
    pub(crate) kind: ArtifactKindV2,
    pub(crate) candidate_count: u8,
    pub(crate) cases: Vec<ArtifactCaseV2>,
    pub(crate) controls: Vec<ArtifactControlV2>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArtifactCaseV2 {
    pub(crate) ordinal: u8,
    pub(crate) name: String,
    pub(crate) observations: Vec<ArtifactObservationV2>,
    pub(crate) literal_match: bool,
    pub(crate) reference_match: bool,
    pub(crate) repeat_match: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArtifactObservationV2 {
    pub(crate) case: u8,
    pub(crate) step: u8,
    pub(crate) generation: Option<u64>,
    pub(crate) viewport: (u32, u32),
    pub(crate) sections: Vec<ArtifactSectionV2>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArtifactSectionV2 {
    pub(crate) name: EvidenceSectionV2,
    pub(crate) records: u64,
    pub(crate) bytes: u64,
    pub(crate) digest: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArtifactControlV2 {
    pub(crate) family: String,
    pub(crate) registered: u64,
    pub(crate) detected: u64,
}
