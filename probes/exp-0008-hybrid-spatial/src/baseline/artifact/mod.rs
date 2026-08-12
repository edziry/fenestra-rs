mod build;
mod decode;
mod encode;
mod error;
mod grammar;
mod limits;
mod model;
mod schema;
mod synthetic;
mod verify;

pub(crate) use build::artifact_from_evidence_v2;
pub(crate) use decode::decode_spatial_evidence_artifact_v2;
pub(crate) use encode::encode_spatial_evidence_artifact_v2;
pub(crate) use error::{
    ARTIFACT_LIMITS_V2, ArtifactErrorKindV2, ArtifactErrorV2, ArtifactLimitKindV2,
};
pub(crate) use grammar::{GrammarValueKindV2, grammar_value_accepts_v2, host_token_probe_v2};
pub(crate) use limits::{artifact_limit_probe_v2, registered_spatial_limits_v2};
pub(crate) use model::{
    ArtifactCaseV2, ArtifactControlV2, ArtifactKindV2, ArtifactObservationV2, ArtifactSectionV2,
    SpatialEvidenceArtifactV2,
};
pub(crate) use synthetic::{ArtifactSyntheticFaultV2, encode_fault_fixture_v2};
pub(crate) use verify::verify_spatial_evidence_artifact_v2;

use grammar::{token, validate_record_grammar};
use schema::{
    CASE_NAMES, CONTROL_FAMILIES, HEADER, LIMITS, OBSERVATION_COUNTS, PACKAGES, PROFILE,
    SPATIAL_LIMITS,
};
