use std::collections::BTreeSet;

use super::super::compare::EvidenceMismatchV2;
use super::super::{EvidenceSectionV2, SpatialEvidenceV2};
use super::{EvidenceMutationV2, MutationProbe};

pub(super) fn metadata_mutations(evidence: &SpatialEvidenceV2) -> Vec<MutationProbe> {
    let mutations = [
        (EvidenceMutationV2::WidthScalar, "width-scalar"),
        (EvidenceMutationV2::WidthDeterminant, "width-determinant"),
        (EvidenceMutationV2::WidthStride, "width-stride"),
        (EvidenceMutationV2::WidthDimension, "width-dimension"),
        (EvidenceMutationV2::WidthKey, "width-key"),
        (EvidenceMutationV2::WidthColor, "width-color"),
        (EvidenceMutationV2::CaseOrdinal, "case-ordinal"),
        (EvidenceMutationV2::CaseName, "case-name"),
        (EvidenceMutationV2::ObservationCase, "case"),
        (EvidenceMutationV2::ObservationStep, "step"),
        (EvidenceMutationV2::ObservationGeneration, "generation"),
        (
            EvidenceMutationV2::ObservationViewportWidth,
            "viewport-width",
        ),
        (
            EvidenceMutationV2::ObservationViewportHeight,
            "viewport-height",
        ),
        (EvidenceMutationV2::LiteralMatch, "literal-match"),
        (EvidenceMutationV2::ReferenceMatch, "reference-match"),
        (EvidenceMutationV2::RepeatMatch, "repeat-match"),
        (EvidenceMutationV2::SectionRecordCount, "record-count"),
        (EvidenceMutationV2::SectionByteCount, "byte-count"),
        (EvidenceMutationV2::SectionDigest, "digest"),
        (EvidenceMutationV2::SectionTag, "section-tag"),
        (EvidenceMutationV2::SectionEncoded, "encoded"),
    ];
    let mut probes = mutations
        .into_iter()
        .map(|(mutation, field)| {
            let record = if mutation == EvidenceMutationV2::SectionRecordCount {
                evidence.cases[0].observations[0].sections[0].records.len()
            } else {
                0
            };
            MutationProbe {
                mutation,
                expected: EvidenceMismatchV2::new(0, 0, EvidenceSectionV2::Receipt, record, field),
            }
        })
        .collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    for probe in unique_field_mutations(evidence) {
        if metadata_field(probe.expected.field.as_str())
            && seen.insert(probe.expected.field.clone())
        {
            probes.push(probe);
        }
    }
    probes
}

pub(super) fn logical_field_mutations(evidence: &SpatialEvidenceV2) -> Vec<MutationProbe> {
    unique_field_mutations(evidence)
}

fn unique_field_mutations(evidence: &SpatialEvidenceV2) -> Vec<MutationProbe> {
    let mut probes = Vec::new();
    let mut seen = BTreeSet::new();
    for (case_index, case) in evidence.cases.iter().enumerate() {
        for (step, observation) in case.observations.iter().enumerate() {
            for section in &observation.sections {
                for (record, value) in section.records.iter().enumerate() {
                    for (field, value) in value.fields.iter().enumerate() {
                        if section.name == EvidenceSectionV2::Raster && value.name == "bytes" {
                            continue;
                        }
                        if !seen.insert((section.name.tag(), value.name)) {
                            continue;
                        }
                        probes.push(MutationProbe {
                            mutation: EvidenceMutationV2::FieldAt {
                                case: case_index as u8,
                                step: step as u8,
                                section: section.name,
                                record: record as u32,
                                field: field as u32,
                                byte: 0,
                            },
                            expected: EvidenceMismatchV2::new(
                                case_index,
                                step,
                                section.name,
                                record,
                                if value.name == "bytes" {
                                    "bytes-length"
                                } else {
                                    value.name
                                },
                            ),
                        });
                    }
                }
            }
        }
    }
    probes
}

fn metadata_field(field: &str) -> bool {
    field.contains("tag")
        || field.contains("key")
        || field.contains("count")
        || field.contains("generation")
        || field.contains("metadata")
        || field.contains("invalidation")
}

pub(super) fn query_mutations(evidence: &SpatialEvidenceV2) -> Vec<MutationProbe> {
    [
        (EvidenceMutationV2::QueryHitToMiss, "result-tag"),
        (EvidenceMutationV2::QueryMissToHit, "result-tag"),
        (EvidenceMutationV2::QueryKey, "key"),
        (EvidenceMutationV2::QueryOwner, "owner"),
        (EvidenceMutationV2::QueryOrdinal, "item-ordinal"),
        (EvidenceMutationV2::QueryLocalX, "local-x"),
        (EvidenceMutationV2::QueryLocalY, "local-y"),
    ]
    .into_iter()
    .filter_map(|(mutation, field)| {
        located_named_probe(evidence, mutation, EvidenceSectionV2::Queries, field)
    })
    .collect()
}

pub(super) fn raster_mutations(evidence: &SpatialEvidenceV2) -> Vec<MutationProbe> {
    let mut probes = [
        (EvidenceMutationV2::RasterWidth, "width"),
        (EvidenceMutationV2::RasterHeight, "height"),
        (EvidenceMutationV2::RasterStride, "stride"),
        (EvidenceMutationV2::RasterByteLength, "bytes"),
    ]
    .into_iter()
    .filter_map(|(mutation, field)| {
        located_named_probe(evidence, mutation, EvidenceSectionV2::Raster, field)
    })
    .collect::<Vec<_>>();
    if let Some((case, step, record, field, payload)) = raster_bytes_location(evidence) {
        for (index, byte) in [
            (0, 8),
            (payload / 2, 8 + payload / 2),
            (payload - 1, 8 + payload - 1),
        ] {
            probes.push(MutationProbe {
                mutation: EvidenceMutationV2::FieldAt {
                    case: case as u8,
                    step: step as u8,
                    section: EvidenceSectionV2::Raster,
                    record: record as u32,
                    field: field as u32,
                    byte: byte as u32,
                },
                expected: EvidenceMismatchV2::new(
                    case,
                    step,
                    EvidenceSectionV2::Raster,
                    record,
                    format!("bytes[{index}]"),
                ),
            });
        }
    }
    probes
}

fn located_named_probe(
    evidence: &SpatialEvidenceV2,
    mutation: EvidenceMutationV2,
    section: EvidenceSectionV2,
    name: &str,
) -> Option<MutationProbe> {
    let required_tag = match mutation {
        EvidenceMutationV2::QueryHitToMiss => Some(1),
        EvidenceMutationV2::QueryMissToHit => Some(0),
        _ => None,
    };
    for (case_index, case) in evidence.cases.iter().enumerate() {
        for (step, observation) in case.observations.iter().enumerate() {
            for (record, value) in observation.section(section).records.iter().enumerate() {
                if let Some((field_index, _)) =
                    value.fields.iter().enumerate().find(|(_, field)| {
                        field.name == name && required_tag.is_none_or(|tag| field.encoded == [tag])
                    })
                {
                    let field = if mutation == EvidenceMutationV2::RasterByteLength {
                        "bytes-length"
                    } else {
                        name
                    };
                    return Some(MutationProbe {
                        mutation: EvidenceMutationV2::FieldAt {
                            case: case_index as u8,
                            step: step as u8,
                            section,
                            record: record as u32,
                            field: field_index as u32,
                            byte: 0,
                        },
                        expected: EvidenceMismatchV2::new(case_index, step, section, record, field),
                    });
                }
            }
        }
    }
    None
}

fn raster_bytes_location(
    evidence: &SpatialEvidenceV2,
) -> Option<(usize, usize, usize, usize, usize)> {
    for (case_index, case) in evidence.cases.iter().enumerate() {
        for (step, observation) in case.observations.iter().enumerate() {
            for (record, value) in observation
                .section(EvidenceSectionV2::Raster)
                .records
                .iter()
                .enumerate()
            {
                if let Some((field_index, field)) = value
                    .fields
                    .iter()
                    .enumerate()
                    .find(|(_, field)| field.name == "bytes" && field.encoded.len() > 8)
                {
                    return Some((
                        case_index,
                        step,
                        record,
                        field_index,
                        field.encoded.len() - 8,
                    ));
                }
            }
        }
    }
    None
}
