mod candidates;
mod compare;
mod faults;
mod input;
mod oracle;
mod png_bytes;
mod types;

pub(crate) use candidates::{image_crate_run_v2, png_image_run_v2};
pub(crate) use compare::classify_image_run_v2;
pub(crate) use faults::image_faults_v2;
pub(crate) use input::image_cases_v2;
pub(crate) use oracle::literal_image_run_v2;
pub(crate) use types::{ImageCandidateV2, ImageFaultKindV2, ImageObligationV2, ImageOutcomeV2};

use types::ImageCandidateRegistrationV2;

pub(crate) const fn image_candidate_registry_v2() -> [ImageCandidateRegistrationV2; 2] {
    [
        ImageCandidateRegistrationV2 {
            kind: ImageCandidateV2::Png,
            name: "png",
            version: "0.18.1",
            features: "-",
        },
        ImageCandidateRegistrationV2 {
            kind: ImageCandidateV2::Image,
            name: "image",
            version: "0.25.10",
            features: "png",
        },
    ]
}
