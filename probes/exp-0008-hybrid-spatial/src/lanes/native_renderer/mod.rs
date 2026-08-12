mod candidates;
mod compare;
mod faults;
mod input;
mod oracle;
mod types;

pub(crate) use candidates::vello_native_run_v2;
pub(crate) use compare::classify_native_run_v2;
pub(crate) use faults::native_faults_v2;
pub(crate) use input::native_cases_v2;
pub(crate) use oracle::literal_native_run_v2;
pub(crate) use types::{NativeCandidateV2, NativeFaultKindV2, NativeObligationV2, NativeOutcomeV2};

use types::NativeCandidateRegistrationV2;

pub(crate) const fn native_candidate_registry_v2() -> [NativeCandidateRegistrationV2; 1] {
    [NativeCandidateRegistrationV2 {
        kind: NativeCandidateV2::Vello,
        name: "vello",
        version: "0.9.0",
        renderer_features: "wgpu",
        gpu_version: "29.0.3",
        gpu_features: "std,parking_lot,wgsl,vulkan,dx12",
        targets: "x86_64-unknown-linux-gnu:vulkan-wayland,x86_64-pc-windows-msvc:dx12-win32",
    }]
}
