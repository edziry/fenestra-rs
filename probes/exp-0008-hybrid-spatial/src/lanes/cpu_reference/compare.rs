use super::types::{CpuClassificationV2, CpuMismatchV2, CpuOutcomeV2, CpuRunV2};

pub(crate) fn classify_cpu_run_v2(literal: &CpuRunV2, observed: &CpuRunV2) -> CpuClassificationV2 {
    let candidate = observed.candidate.expect("registered CPU candidate run");
    let first_mismatch = first_mismatch(literal, observed);
    let (outcome, reason) = if first_mismatch.is_none() {
        (CpuOutcomeV2::Pass, "-")
    } else if premultiplication_equivalent(literal, observed) {
        (CpuOutcomeV2::Adapt, "premultiplied-rgba8")
    } else {
        (CpuOutcomeV2::Stop, "mismatch")
    };
    CpuClassificationV2 {
        candidate,
        outcome,
        reason,
        first_mismatch,
    }
}

fn premultiplication_equivalent(expected: &CpuRunV2, observed: &CpuRunV2) -> bool {
    expected.cases.len() == observed.cases.len()
        && expected
            .cases
            .iter()
            .zip(&observed.cases)
            .all(|(expected_case, observed_case)| {
                expected_case.ordinal == observed_case.ordinal
                    && expected_case.width == observed_case.width
                    && expected_case.height == observed_case.height
                    && expected_case.bytes.len() == observed_case.bytes.len()
                    && expected_case
                        .bytes
                        .chunks_exact(4)
                        .zip(observed_case.bytes.chunks_exact(4))
                        .all(|(expected_pixel, observed_pixel)| {
                            canonical_pixel(expected_pixel) == canonical_pixel(observed_pixel)
                        })
            })
}

fn canonical_pixel(pixel: &[u8]) -> [u8; 4] {
    let alpha = pixel[3];
    if alpha == 0 {
        return [0; 4];
    }
    let mut result = [0, 0, 0, alpha];
    for channel in 0..3 {
        let straight =
            ((u16::from(pixel[channel]) * 255 + u16::from(alpha) / 2) / u16::from(alpha)).min(255);
        result[channel] = ((straight * u16::from(alpha) + 127) / 255) as u8;
    }
    result
}

fn first_mismatch(expected: &CpuRunV2, observed: &CpuRunV2) -> Option<CpuMismatchV2> {
    for (case_index, expected_case) in expected.cases.iter().enumerate() {
        let Some(observed_case) = observed.cases.get(case_index) else {
            return Some(mismatch(expected_case.ordinal, 0, 0, 0));
        };
        if expected_case.ordinal != observed_case.ordinal
            || expected_case.width != observed_case.width
            || expected_case.height != observed_case.height
        {
            return Some(mismatch(expected_case.ordinal, 0, 0, 0));
        }
        let length = expected_case.bytes.len().max(observed_case.bytes.len());
        for byte in 0..length {
            let expected_byte = expected_case.bytes.get(byte).copied().unwrap_or(0);
            let observed_byte = observed_case.bytes.get(byte).copied().unwrap_or(0);
            if expected_byte != observed_byte {
                return Some(mismatch(
                    expected_case.ordinal,
                    byte,
                    expected_byte,
                    observed_byte,
                ));
            }
        }
    }
    if observed.cases.len() != expected.cases.len() {
        return Some(mismatch(expected.cases.len() as u8, 0, 0, 0));
    }
    None
}

const fn mismatch(case: u8, byte: usize, expected: u8, observed: u8) -> CpuMismatchV2 {
    CpuMismatchV2 {
        case,
        byte,
        expected,
        observed,
    }
}
