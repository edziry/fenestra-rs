mod candidates;
mod faults;
mod input;
mod oracle;
mod types;

pub(crate) use candidates::kurbo_run as kurbo_numeric_run_v2;
pub(crate) use candidates::{
    euclid_run as euclid_numeric_run_v2, fixed_run as fixed_numeric_run_v2,
};
pub(crate) use faults::numeric_faults_v2;
pub(crate) use input::numeric_inputs_v2;
pub(crate) use oracle::literal_numeric_run_v2;
pub(crate) use types::{NumericCandidateV2, NumericFaultKindV2, NumericOutcomeV2};

use types::NumericCandidateRegistrationV2;

pub(crate) const fn numeric_candidate_registry_v2() -> [NumericCandidateRegistrationV2; 3] {
    [
        NumericCandidateRegistrationV2 {
            kind: NumericCandidateV2::Euclid,
            name: "euclid",
            version: "0.22.14",
            features: "std",
            outcome: NumericOutcomeV2::Pass,
            reason: "-",
        },
        NumericCandidateRegistrationV2 {
            kind: NumericCandidateV2::Kurbo,
            name: "kurbo",
            version: "0.13.1",
            features: "std",
            outcome: NumericOutcomeV2::Pass,
            reason: "-",
        },
        NumericCandidateRegistrationV2 {
            kind: NumericCandidateV2::Fixed,
            name: "fixed",
            version: "1.30.0",
            features: "-",
            outcome: NumericOutcomeV2::Pass,
            reason: "-",
        },
    ]
}
