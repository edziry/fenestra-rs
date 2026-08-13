#![forbid(unsafe_code)]

//! Disposable interactive native GPU feasibility probe for Fenestra.

mod admission;
mod artifact;
mod cli;
mod evidence;
mod presentation;
mod scene;

pub use admission::{
    GpuAdapterObservationV1, GpuAdmissionErrorKindV1, GpuBackendV1, GpuDeviceTypeV1, GpuTargetV1,
    admit_adapter_v1,
};
pub use artifact::{
    ARTIFACT_LIMITS_V1, ArtifactAdaptReasonV1, ArtifactAdapterV1, ArtifactEventV1,
    ArtifactPresentV1, ArtifactSurfaceV1, ArtifactTerminalV1, InteractiveArtifactBuilderV1,
    InteractiveArtifactErrorKindV1, InteractiveArtifactLimitsV1, SurfaceAlphaV1, SurfaceFormatV1,
    SurfacePresentModeV1, VerifiedInteractiveArtifactV1, verify_interactive_artifact_v1,
};
pub use cli::{ProbeCliErrorKindV1, ProbeCliV1, parse_probe_cli_v1};
pub use evidence::{
    InteractiveEvidenceErrorKindV1, InteractiveEvidenceV1, InteractiveMilestoneV1,
    InteractiveObservationV1, InteractiveResultV1,
};
pub use presentation::{
    GpuPortReceiptV1, GpuPresentErrorKindV1, GpuPresentErrorV1, GpuPresentPortV1,
    GpuPresentationOutcomeV1, GpuPresentationReceiptV1, GpuSurfaceExtentV1, present_gpu_offer_v1,
};
pub use scene::{
    RegisteredSceneErrorKindV1, RegisteredSceneObservationV1, build_registered_runtime_v1,
    inspect_registered_scene_pair_v1,
};
