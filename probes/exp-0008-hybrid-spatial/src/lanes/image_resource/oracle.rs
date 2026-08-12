use super::faults::preflight_cases;
use super::types::{ImageCaseV2, ImageResultV2, ImageRunV2};

pub(crate) fn literal_image_run_v2(cases: &[ImageCaseV2]) -> ImageResultV2<ImageRunV2> {
    preflight_cases(cases)?;
    Ok(ImageRunV2::literal(
        cases.iter().map(|case| case.expected.clone()).collect(),
    ))
}
