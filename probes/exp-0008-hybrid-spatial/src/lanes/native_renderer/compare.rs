use super::types::{
    NativeClassificationV2, NativeMismatchV2, NativeOutcomeV2, NativeRecordV2, NativeRunV2,
};

pub(crate) fn classify_native_run_v2(
    literal: &NativeRunV2,
    observed: &NativeRunV2,
) -> NativeClassificationV2 {
    let candidate = observed.candidate.expect("registered native candidate run");
    let first_mismatch = first_mismatch(literal, observed);
    let (outcome, reason) = if first_mismatch.is_some() {
        (NativeOutcomeV2::Stop, "mismatch")
    } else if !observed.used_vello_scene {
        (NativeOutcomeV2::Adapt, "painter-reorder")
    } else if !observed.executed_gpu {
        (NativeOutcomeV2::Stop, "target-unavailable")
    } else {
        (NativeOutcomeV2::Pass, "-")
    };
    NativeClassificationV2 {
        candidate,
        outcome,
        reason,
        first_mismatch,
    }
}

fn first_mismatch(expected: &NativeRunV2, observed: &NativeRunV2) -> Option<NativeMismatchV2> {
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

fn record_mismatch(expected: &NativeRecordV2, observed: &NativeRecordV2) -> Option<&'static str> {
    if expected.ordinal != observed.ordinal {
        Some("ordinal")
    } else if expected.width != observed.width {
        Some("width")
    } else if expected.height != observed.height {
        Some("height")
    } else if expected.commands != observed.commands {
        Some("commands")
    } else if expected.shapes != observed.shapes {
        Some("shapes")
    } else if expected.clips != observed.clips {
        Some("clips")
    } else if expected.images != observed.images {
        Some("images")
    } else if expected.painter_digest != observed.painter_digest {
        Some("painter-order")
    } else {
        None
    }
}

const fn mismatch(record: u8, field: &'static str) -> NativeMismatchV2 {
    NativeMismatchV2 { record, field }
}
