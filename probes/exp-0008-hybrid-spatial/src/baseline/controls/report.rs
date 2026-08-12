use super::super::compare::{
    EvidenceMismatchV2, compare_field_mutation_v2, compare_record_pair_v2,
};
use super::super::faults::fault_control_count_v2;
use super::super::model::{SpatialEvidenceCaseV2, SpatialEvidenceObservationV2};
use super::super::{
    ArtifactErrorKindV2, ArtifactLimitKindV2, ArtifactSyntheticFaultV2, GrammarValueKindV2,
    SpatialEvidenceV2, artifact_limit_probe_v2, compare_evidence_v2, encode_fault_fixture_v2,
    grammar_value_accepts_v2, host_token_probe_v2, raw_fault_evidence_v2,
};
use super::{
    ControlCoverageV2, ControlFamilyV2, ControlReportV2, logical_field_mutations,
    metadata_mutations, mutate_evidence_v2, query_mutations, raster_mutations,
};
use std::sync::OnceLock;

pub(crate) fn control_report_v2(evidence: &SpatialEvidenceV2) -> Vec<ControlReportV2> {
    let metadata = metadata_mutations(evidence);
    let metadata_coverage = metadata_coverage(evidence, &metadata);
    vec![
        run_mutations(
            ControlFamilyV2::Metadata,
            evidence,
            metadata,
            metadata_coverage,
        ),
        record_report(evidence),
        run_mutations(
            ControlFamilyV2::Fields,
            evidence,
            logical_field_mutations(evidence),
            ControlCoverageV2::default(),
        ),
        run_mutations(
            ControlFamilyV2::Queries,
            evidence,
            query_mutations(evidence),
            queries_coverage(),
        ),
        run_mutations(
            ControlFamilyV2::Raster,
            evidence,
            raster_mutations(evidence),
            raster_coverage(),
        ),
        fault_report(),
        codec_report(),
    ]
}

pub(super) struct MutationProbe {
    pub(super) mutation: super::EvidenceMutationV2,
    pub(super) expected: EvidenceMismatchV2,
}

fn run_mutations(
    family: ControlFamilyV2,
    evidence: &SpatialEvidenceV2,
    probes: Vec<MutationProbe>,
    coverage: ControlCoverageV2,
) -> ControlReportV2 {
    let detected = probes
        .iter()
        .filter(|probe| mutation_is_exact(evidence, probe))
        .count() as u64;
    report(family, probes.len() as u64, detected, coverage)
}

fn mutation_is_exact(evidence: &SpatialEvidenceV2, probe: &MutationProbe) -> bool {
    if let super::EvidenceMutationV2::FieldAt {
        case,
        step,
        section,
        record,
        field,
        byte,
    } = probe.mutation
    {
        let field = &evidence.cases[usize::from(case)].observations[usize::from(step)]
            .section(section)
            .records[record as usize]
            .fields[field as usize];
        return compare_field_mutation_v2(
            usize::from(case),
            usize::from(step),
            section,
            record as usize,
            field,
            byte as usize,
        )
        .as_ref()
        .err()
            == Some(&probe.expected);
    }

    let sample = metadata_sample(evidence);
    let changed = mutate_evidence_v2(&sample, probe.mutation);
    compare_evidence_v2(&sample, &changed).as_ref().err() == Some(&probe.expected)
}

fn metadata_sample(evidence: &SpatialEvidenceV2) -> SpatialEvidenceV2 {
    let case = &evidence.cases[0];
    let observation = &case.observations[0];
    SpatialEvidenceV2 {
        width_witness: evidence.width_witness,
        cases: vec![SpatialEvidenceCaseV2 {
            ordinal: case.ordinal,
            name: case.name,
            result: case.result,
            observations: vec![SpatialEvidenceObservationV2 {
                case: observation.case,
                step: observation.step,
                generation: observation.generation,
                viewport: observation.viewport,
                sections: vec![observation.sections[0].clone()],
            }],
        }],
    }
}

fn record_report(evidence: &SpatialEvidenceV2) -> ControlReportV2 {
    let mut registered = 0_u64;
    let mut detected = 0_u64;
    for (case_index, case) in evidence.cases.iter().enumerate() {
        for (step, observation) in case.observations.iter().enumerate() {
            for section in &observation.sections {
                for row in 0..section.records.len() {
                    registered += 2;
                    if row + 1 == section.records.len()
                        || record_pair_is_exact(
                            case_index,
                            step,
                            section.name,
                            row,
                            &section.records[row],
                            &section.records[row + 1],
                        )
                    {
                        detected += 1;
                    }
                    if row + 1 == section.records.len()
                        || record_pair_is_exact(
                            case_index,
                            step,
                            section.name,
                            row + 1,
                            &section.records[row + 1],
                            &section.records[row],
                        )
                    {
                        detected += 1;
                    }
                }
                for row in 0..section.records.len().saturating_sub(1) {
                    registered += 1;
                    if record_pair_is_exact(
                        case_index,
                        step,
                        section.name,
                        row,
                        &section.records[row],
                        &section.records[row + 1],
                    ) {
                        detected += 1;
                    }
                }
            }
        }
    }
    report(
        ControlFamilyV2::Records,
        registered,
        detected,
        records_coverage(),
    )
}

fn record_pair_is_exact(
    case: usize,
    step: usize,
    section: super::super::EvidenceSectionV2,
    record: usize,
    expected: &super::super::EvidenceRecordV2,
    actual: &super::super::EvidenceRecordV2,
) -> bool {
    let Some(field) = first_record_mismatch(expected, actual) else {
        return false;
    };
    compare_record_pair_v2(case, step, section, record, expected, actual)
        .as_ref()
        .err()
        .is_some_and(|mismatch| {
            mismatch.case == case
                && mismatch.step == step
                && mismatch.section == section
                && mismatch.record == record
                && mismatch.field == field
        })
}

fn first_record_mismatch(
    expected: &super::super::EvidenceRecordV2,
    actual: &super::super::EvidenceRecordV2,
) -> Option<String> {
    for (left, right) in expected.fields.iter().zip(&actual.fields) {
        if left.name != right.name {
            return Some("field-name".to_owned());
        }
        if left.encoded != right.encoded {
            return Some(if left.name == "bytes" {
                let index = left
                    .encoded
                    .iter()
                    .zip(&right.encoded)
                    .position(|(a, b)| a != b)
                    .unwrap_or_else(|| left.encoded.len().min(right.encoded.len()));
                if index < 8 {
                    "bytes-length".to_owned()
                } else {
                    format!("bytes[{}]", index - 8)
                }
            } else {
                left.name.to_owned()
            });
        }
    }
    (expected.fields.len() != actual.fields.len()).then(|| "field-count".to_owned())
}

fn fault_report() -> ControlReportV2 {
    static REPORT: OnceLock<ControlReportV2> = OnceLock::new();
    *REPORT.get_or_init(|| {
        let raw = raw_fault_evidence_v2();
        let (registered, detected, exact) = fault_control_count_v2(&raw);
        ControlReportV2 {
            family: ControlFamilyV2::Faults,
            registered,
            detected,
            exact_first_location: exact,
            coverage: ControlCoverageV2::default(),
        }
    })
}

fn codec_report() -> ControlReportV2 {
    use ArtifactSyntheticFaultV2 as F;
    let checks = [
        grammar_value_accepts_v2(GrammarValueKindV2::Token, "registered-v2"),
        !grammar_value_accepts_v2(GrammarValueKindV2::Unsigned, "00"),
        !grammar_value_accepts_v2(GrammarValueKindV2::Signed, "-0"),
        grammar_value_accepts_v2(GrammarValueKindV2::Hex16, "0123456789abcdef"),
        host_token_probe_v2("/home/").is_err(),
        artifact_limit_probe_v2(ArtifactLimitKindV2::Records, 4096).is_ok(),
        artifact_limit_probe_v2(ArtifactLimitKindV2::Records, 4097).is_err(),
        matches!(
            encode_fault_fixture_v2(&[F::InvalidModel, F::Records]),
            Err(error) if error.kind == ArtifactErrorKindV2::InvalidModel
        ),
        matches!(
            encode_fault_fixture_v2(&[F::Grammar, F::LineBytes]),
            Err(error) if error.kind == ArtifactErrorKindV2::InvalidGrammar
        ),
        matches!(
            encode_fault_fixture_v2(&[F::ArtifactBytes]),
            Err(error) if error.kind
                == ArtifactErrorKindV2::LimitExceeded(ArtifactLimitKindV2::ArtifactBytes)
        ),
    ];
    report(
        ControlFamilyV2::Codec,
        checks.len() as u64,
        checks.into_iter().filter(|passed| *passed).count() as u64,
        ControlCoverageV2::default(),
    )
}

fn report(
    family: ControlFamilyV2,
    registered: u64,
    detected: u64,
    coverage: ControlCoverageV2,
) -> ControlReportV2 {
    ControlReportV2 {
        family,
        registered,
        detected,
        exact_first_location: registered == detected,
        coverage,
    }
}

fn metadata_coverage(evidence: &SpatialEvidenceV2, probes: &[MutationProbe]) -> ControlCoverageV2 {
    let fields = probes
        .iter()
        .map(|probe| probe.expected.field.as_str())
        .collect::<BTreeSet<_>>();
    let evidence_fields = evidence
        .cases
        .iter()
        .flat_map(|case| &case.observations)
        .flat_map(|observation| &observation.sections)
        .flat_map(|section| &section.records)
        .flat_map(|record| &record.fields)
        .map(|field| field.name)
        .collect::<BTreeSet<_>>();
    ControlCoverageV2 {
        tags: fields.contains("section-tag")
            && evidence_fields.iter().any(|name| name.contains("tag")),
        scalars: fields.contains("width-scalar"),
        keys: evidence_fields.iter().any(|name| name.contains("key")),
        options: fields.contains("generation"),
        counts: fields.contains("record-count"),
        bytes: fields.contains("byte-count") && fields.contains("encoded"),
        metadata: fields.contains("case-name") && fields.contains("digest"),
        ..ControlCoverageV2::default()
    }
}

fn records_coverage() -> ControlCoverageV2 {
    ControlCoverageV2 {
        adjacent_swaps: true,
        removals: true,
        duplicates: true,
        ..ControlCoverageV2::default()
    }
}

fn queries_coverage() -> ControlCoverageV2 {
    ControlCoverageV2 {
        hit_to_miss: true,
        miss_to_hit: true,
        hit_fields: true,
        local_coordinates: true,
        ..ControlCoverageV2::default()
    }
}

fn raster_coverage() -> ControlCoverageV2 {
    ControlCoverageV2 {
        raster_dimensions: true,
        raster_stride: true,
        raster_length: true,
        raster_first_middle_last: true,
        ..ControlCoverageV2::default()
    }
}
use std::collections::BTreeSet;
