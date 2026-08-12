use super::types::{
    ImageClassificationV2, ImageMismatchV2, ImageOutcomeV2, ImageRecordV2, ImageRunV2,
};

pub(crate) fn classify_image_run_v2(
    literal: &ImageRunV2,
    observed: &ImageRunV2,
) -> ImageClassificationV2 {
    let candidate = observed.candidate.expect("registered image candidate run");
    let first_mismatch = first_mismatch(literal, observed);
    let (outcome, reason) = if first_mismatch.is_some() {
        (ImageOutcomeV2::Stop, "mismatch")
    } else if observed.used_orientation_adapter {
        (ImageOutcomeV2::Adapt, "orientation-normalization")
    } else {
        (ImageOutcomeV2::Pass, "-")
    };
    ImageClassificationV2 {
        candidate,
        outcome,
        reason,
        first_mismatch,
    }
}

fn first_mismatch(expected: &ImageRunV2, observed: &ImageRunV2) -> Option<ImageMismatchV2> {
    for (index, expected_record) in expected.records.iter().enumerate() {
        let Some(observed_record) = observed.records.get(index) else {
            return Some(mismatch(expected_record.ordinal, "record"));
        };
        if let Some(field) = record_mismatch(expected_record, observed_record) {
            return Some(mismatch(expected_record.ordinal, field));
        }
    }
    (expected.records.len() != observed.records.len())
        .then(|| mismatch(expected.records.len() as u8, "record"))
}

fn record_mismatch(expected: &ImageRecordV2, observed: &ImageRecordV2) -> Option<&'static str> {
    if expected.ordinal != observed.ordinal {
        Some("ordinal")
    } else if expected.width != observed.width {
        Some("width")
    } else if expected.height != observed.height {
        Some("height")
    } else if expected.stride != observed.stride {
        Some("stride")
    } else if expected.rgba8 != observed.rgba8 {
        Some("rgba8")
    } else if expected.gamma_scaled != observed.gamma_scaled {
        Some("gamma")
    } else if expected.profile_fingerprint != observed.profile_fingerprint {
        Some("profile")
    } else if expected.orientation != observed.orientation {
        Some("orientation")
    } else {
        None
    }
}

const fn mismatch(record: u8, field: &'static str) -> ImageMismatchV2 {
    ImageMismatchV2 { record, field }
}
