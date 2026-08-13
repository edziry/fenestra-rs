#![forbid(unsafe_code)]

//! Disposable interactive native GPU feasibility probe for Fenestra.

mod admission;
mod artifact;
mod evidence;
mod scene;

pub use admission::{
    GpuAdapterObservationV1, GpuAdmissionErrorKindV1, GpuBackendV1, GpuDeviceTypeV1, GpuTargetV1,
    admit_adapter_v1,
};
pub use artifact::{
    ARTIFACT_LIMITS_V1, InteractiveArtifactErrorKindV1, InteractiveArtifactLimitsV1,
    VerifiedInteractiveArtifactV1, verify_interactive_artifact_v1,
};
pub use evidence::{
    InteractiveEvidenceErrorKindV1, InteractiveEvidenceV1, InteractiveMilestoneV1,
    InteractiveObservationV1, InteractiveResultV1,
};
pub use scene::{
    RegisteredSceneErrorKindV1, RegisteredSceneObservationV1, inspect_registered_scene_pair_v1,
};
