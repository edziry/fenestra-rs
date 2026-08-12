mod candidates;
mod compare;
mod faults;
mod input;
mod oracle;
mod types;

pub(crate) use candidates::{raqote_cpu_run_v2, tiny_skia_cpu_run_v2};
pub(crate) use compare::classify_cpu_run_v2;
pub(crate) use faults::cpu_faults_v2;
pub(crate) use input::cpu_cases_v2;
pub(crate) use oracle::literal_cpu_run_v2;
pub(crate) use types::{CpuCandidateV2, CpuFaultKindV2, CpuObligationV2, CpuOutcomeV2};

use types::CpuCandidateRegistrationV2;

pub(crate) const fn cpu_candidate_registry_v2() -> [CpuCandidateRegistrationV2; 2] {
    [
        CpuCandidateRegistrationV2 {
            kind: CpuCandidateV2::TinySkia,
            name: "tiny-skia",
            version: "0.12.0",
            features: "std",
        },
        CpuCandidateRegistrationV2 {
            kind: CpuCandidateV2::Raqote,
            name: "raqote",
            version: "0.8.5",
            features: "-",
        },
    ]
}
