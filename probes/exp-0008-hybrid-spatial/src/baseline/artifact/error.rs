#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArtifactLimitKindV2 {
    Records,
    LineBytes,
    ArtifactBytes,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArtifactLimitsV2 {
    pub(crate) records: usize,
    pub(crate) line_bytes: usize,
    pub(crate) artifact_bytes: usize,
}

pub(crate) const ARTIFACT_LIMITS_V2: ArtifactLimitsV2 = ArtifactLimitsV2 {
    records: 4096,
    line_bytes: 1024,
    artifact_bytes: 1_048_576,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArtifactErrorKindV2 {
    InvalidModel,
    InvalidVersion,
    InvalidOrder,
    InvalidCount,
    InvalidReference,
    DigestMismatch,
    InvalidGrammar,
    LimitExceeded(ArtifactLimitKindV2),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArtifactErrorV2 {
    pub(crate) kind: ArtifactErrorKindV2,
    pub(crate) observed: Option<u128>,
    pub(crate) maximum: Option<u128>,
    pub(crate) record: Option<usize>,
    pub(crate) artifact: Option<Vec<u8>>,
}

impl ArtifactErrorV2 {
    pub(crate) const fn new(kind: ArtifactErrorKindV2) -> Self {
        Self {
            kind,
            observed: None,
            maximum: None,
            record: None,
            artifact: None,
        }
    }

    pub(crate) const fn at(kind: ArtifactErrorKindV2, record: usize) -> Self {
        Self {
            kind,
            observed: None,
            maximum: None,
            record: Some(record),
            artifact: None,
        }
    }

    pub(crate) const fn limit(
        kind: ArtifactLimitKindV2,
        observed: usize,
        maximum: usize,
        record: Option<usize>,
    ) -> Self {
        Self {
            kind: ArtifactErrorKindV2::LimitExceeded(kind),
            observed: Some(observed as u128),
            maximum: Some(maximum as u128),
            record,
            artifact: None,
        }
    }
}
