#![forbid(unsafe_code)]

//! Disposable interactive native GPU feasibility probe for Fenestra.

mod admission;
mod evidence;

pub use admission::{
    GpuAdapterObservationV1, GpuAdmissionErrorKindV1, GpuBackendV1, GpuDeviceTypeV1, GpuTargetV1,
    admit_adapter_v1,
};
pub use evidence::{
    InteractiveEvidenceErrorKindV1, InteractiveEvidenceV1, InteractiveMilestoneV1,
    InteractiveObservationV1, InteractiveResultV1,
};
