use super::super::{
    EvidenceFieldV2, EvidenceRecordV2, EvidenceSectionV2, NormalizedSectionV2,
    SpatialEvidenceObservationV2, SpatialEvidenceV2,
};
use super::EvidenceMismatchV2;

type Compared = Result<(), EvidenceMismatchV2>;

pub(crate) fn compare_evidence_v2(
    expected: &SpatialEvidenceV2,
    actual: &SpatialEvidenceV2,
) -> Compared {
    compare_widths(expected, actual)?;
    if expected.cases.len() != actual.cases.len() {
        return mismatch(0, 0, EvidenceSectionV2::Receipt, 0, "case-count");
    }
    for (case_index, (expected_case, actual_case)) in
        expected.cases.iter().zip(&actual.cases).enumerate()
    {
        if expected_case.ordinal != actual_case.ordinal {
            return mismatch(case_index, 0, EvidenceSectionV2::Receipt, 0, "case-ordinal");
        }
        if expected_case.name != actual_case.name {
            return mismatch(case_index, 0, EvidenceSectionV2::Receipt, 0, "case-name");
        }
        if expected_case.observations.len() != actual_case.observations.len() {
            return mismatch(
                case_index,
                0,
                EvidenceSectionV2::Receipt,
                0,
                "observation-count",
            );
        }
        for (step, (expected_observation, actual_observation)) in expected_case
            .observations
            .iter()
            .zip(&actual_case.observations)
            .enumerate()
        {
            compare_observation(case_index, step, expected_observation, actual_observation)?;
        }
        if expected_case.result.literal_match != actual_case.result.literal_match {
            return mismatch(
                case_index,
                0,
                EvidenceSectionV2::Receipt,
                0,
                "literal-match",
            );
        }
        if expected_case.result.reference_match != actual_case.result.reference_match {
            return mismatch(
                case_index,
                0,
                EvidenceSectionV2::Receipt,
                0,
                "reference-match",
            );
        }
        if expected_case.result.repeat_match != actual_case.result.repeat_match {
            return mismatch(case_index, 0, EvidenceSectionV2::Receipt, 0, "repeat-match");
        }
    }
    Ok(())
}

fn compare_widths(expected: &SpatialEvidenceV2, actual: &SpatialEvidenceV2) -> Compared {
    let left = expected.width_witness;
    let right = actual.width_witness;
    for (matches, field) in [
        (left.scalar == right.scalar, "width-scalar"),
        (left.determinant == right.determinant, "width-determinant"),
        (left.stride == right.stride, "width-stride"),
        (left.dimension == right.dimension, "width-dimension"),
        (left.key == right.key, "width-key"),
        (left.color == right.color, "width-color"),
    ] {
        if !matches {
            return mismatch(0, 0, EvidenceSectionV2::Receipt, 0, field);
        }
    }
    Ok(())
}

fn compare_observation(
    case: usize,
    step: usize,
    expected: &SpatialEvidenceObservationV2,
    actual: &SpatialEvidenceObservationV2,
) -> Compared {
    for (matches, field) in [
        (expected.case == actual.case, "case"),
        (expected.step == actual.step, "step"),
        (expected.generation == actual.generation, "generation"),
        (expected.viewport.0 == actual.viewport.0, "viewport-width"),
        (expected.viewport.1 == actual.viewport.1, "viewport-height"),
    ] {
        if !matches {
            return mismatch(case, step, EvidenceSectionV2::Receipt, 0, field);
        }
    }
    if expected.sections.len() != actual.sections.len() {
        return mismatch(case, step, EvidenceSectionV2::Receipt, 0, "section-count");
    }
    for (section_index, (left, right)) in expected.sections.iter().zip(&actual.sections).enumerate()
    {
        let expected_name = EvidenceSectionV2::ALL
            .get(section_index)
            .copied()
            .unwrap_or(left.name);
        if left.name != right.name || left.name != expected_name {
            return mismatch(case, step, expected_name, 0, "section-tag");
        }
        compare_section(case, step, left, right)?;
    }
    Ok(())
}

fn compare_section(
    case: usize,
    step: usize,
    expected: &NormalizedSectionV2,
    actual: &NormalizedSectionV2,
) -> Compared {
    for (record, (left, right)) in expected.records.iter().zip(&actual.records).enumerate() {
        compare_record(case, step, expected.name, record, left, right)?;
    }
    if expected.records.len() != actual.records.len()
        || expected.record_count != actual.record_count
    {
        return mismatch(
            case,
            step,
            expected.name,
            expected.records.len().min(actual.records.len()),
            "record-count",
        );
    }
    for (matches, field) in [
        (expected.byte_count == actual.byte_count, "byte-count"),
        (expected.encoded == actual.encoded, "encoded"),
        (expected.digest == actual.digest, "digest"),
    ] {
        if !matches {
            return mismatch(case, step, expected.name, 0, field);
        }
    }
    Ok(())
}

fn compare_record(
    case: usize,
    step: usize,
    section: EvidenceSectionV2,
    record: usize,
    expected: &EvidenceRecordV2,
    actual: &EvidenceRecordV2,
) -> Compared {
    for (left, right) in expected.fields.iter().zip(&actual.fields) {
        compare_field(case, step, section, record, left, right)?;
    }
    if expected.fields.len() != actual.fields.len() {
        return mismatch(case, step, section, record, "field-count");
    }
    Ok(())
}

pub(crate) fn compare_field_mutation_v2(
    case: usize,
    step: usize,
    section: EvidenceSectionV2,
    record: usize,
    expected: &EvidenceFieldV2,
    byte: usize,
) -> Compared {
    let mut actual = expected.clone();
    if actual.encoded.is_empty() {
        actual.encoded.push(1);
    } else {
        let byte = byte.min(actual.encoded.len() - 1);
        actual.encoded[byte] ^= 1;
    }
    compare_field(case, step, section, record, expected, &actual)
}

pub(crate) fn compare_record_pair_v2(
    case: usize,
    step: usize,
    section: EvidenceSectionV2,
    record: usize,
    expected: &EvidenceRecordV2,
    actual: &EvidenceRecordV2,
) -> Compared {
    compare_record(case, step, section, record, expected, actual)
}

fn compare_field(
    case: usize,
    step: usize,
    section: EvidenceSectionV2,
    record: usize,
    expected: &EvidenceFieldV2,
    actual: &EvidenceFieldV2,
) -> Compared {
    if expected.name != actual.name {
        return mismatch(case, step, section, record, "field-name");
    }
    if expected.encoded != actual.encoded {
        let field = byte_field_location(expected, actual);
        return mismatch(case, step, section, record, field);
    }
    Ok(())
}

fn byte_field_location(expected: &EvidenceFieldV2, actual: &EvidenceFieldV2) -> String {
    if expected.name != "bytes" {
        return expected.name.to_owned();
    }
    let differing = expected
        .encoded
        .iter()
        .zip(&actual.encoded)
        .position(|(left, right)| left != right)
        .unwrap_or_else(|| expected.encoded.len().min(actual.encoded.len()));
    if differing < 8 {
        "bytes-length".to_owned()
    } else {
        format!("bytes[{}]", differing - 8)
    }
}

fn mismatch<T>(
    case: usize,
    step: usize,
    section: EvidenceSectionV2,
    record: usize,
    field: impl Into<String>,
) -> Result<T, EvidenceMismatchV2> {
    Err(EvidenceMismatchV2::new(case, step, section, record, field))
}
