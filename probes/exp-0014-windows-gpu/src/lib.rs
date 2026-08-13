#![forbid(unsafe_code)]

//! Disposable interactive native GPU feasibility probe for Fenestra.

use std::error::Error;
use std::fmt;

mod admission;
mod artifact;
mod cli;
mod evidence;
mod native;
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

/// Closed failures that prevent the native probe from emitting an artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InteractiveProbeErrorKindV1 {
    /// The executable was not built with the required release profile.
    BuildProfile,
    /// The native event loop could not start or complete.
    EventLoop,
    /// The native window could not be created.
    Window,
    /// The registered runtime or scheduler contract failed.
    Runtime,
    /// The bounded evidence artifact could not be formed.
    Artifact,
}

impl fmt::Display for InteractiveProbeErrorKindV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("interactive GPU probe failed")
    }
}

impl Error for InteractiveProbeErrorKindV1 {}

/// Runs one native interactive GPU probe to a terminal evidence artifact.
#[must_use = "interactive probe failures must be handled"]
pub fn run_interactive_probe_v1() -> Result<Vec<u8>, InteractiveProbeErrorKindV1> {
    native::run_interactive_probe_v1()
}
