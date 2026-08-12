use super::super::{EvidenceRecordV2, EvidenceSectionV2, SpatialEvidenceV2};
use super::EvidenceMutationV2;

pub(crate) fn mutate_evidence_v2(
    evidence: &SpatialEvidenceV2,
    mutation: EvidenceMutationV2,
) -> SpatialEvidenceV2 {
    let mut changed = evidence.clone();
    match mutation {
        EvidenceMutationV2::ReceiptGeneration => {
            mutate_named(&mut changed, EvidenceSectionV2::Receipt, "generation", 0);
        }
        EvidenceMutationV2::GeometryAndRaster => {
            mutate_named(
                &mut changed,
                EvidenceSectionV2::Geometry,
                "world-aabb-min-x",
                0,
            );
            mutate_bytes(&mut changed, ByteMutation::First);
        }
        EvidenceMutationV2::QueryHitToMiss => {
            mutate_query_result(&mut changed, 1);
        }
        EvidenceMutationV2::RasterFirstByte => mutate_bytes(&mut changed, ByteMutation::First),
        EvidenceMutationV2::SwapAdjacent(section) => mutate_rows(&mut changed, section, Row::Swap),
        EvidenceMutationV2::RemoveRow(section) => mutate_rows(&mut changed, section, Row::Remove),
        EvidenceMutationV2::DuplicateRow(section) => {
            mutate_rows(&mut changed, section, Row::Duplicate);
        }
        EvidenceMutationV2::FieldAt {
            case,
            step,
            section,
            record,
            field,
            byte,
        } => mutate_at(
            &mut changed,
            usize::from(case),
            usize::from(step),
            section,
            record as usize,
            field as usize,
            byte as usize,
        ),
        EvidenceMutationV2::QueryMissToHit => mutate_query_result(&mut changed, 0),
        EvidenceMutationV2::QueryKey => {
            mutate_named(&mut changed, EvidenceSectionV2::Queries, "key", 0);
        }
        EvidenceMutationV2::QueryOwner => {
            mutate_named(&mut changed, EvidenceSectionV2::Queries, "owner", 0);
        }
        EvidenceMutationV2::QueryOrdinal => {
            mutate_named(&mut changed, EvidenceSectionV2::Queries, "item-ordinal", 0);
        }
        EvidenceMutationV2::QueryLocalX => {
            mutate_named(&mut changed, EvidenceSectionV2::Queries, "local-x", 0);
        }
        EvidenceMutationV2::QueryLocalY => {
            mutate_named(&mut changed, EvidenceSectionV2::Queries, "local-y", 0);
        }
        EvidenceMutationV2::RasterWidth => {
            mutate_named(&mut changed, EvidenceSectionV2::Raster, "width", 0);
        }
        EvidenceMutationV2::RasterHeight => {
            mutate_named(&mut changed, EvidenceSectionV2::Raster, "height", 0);
        }
        EvidenceMutationV2::RasterStride => {
            mutate_named(&mut changed, EvidenceSectionV2::Raster, "stride", 0);
        }
        EvidenceMutationV2::RasterByteLength => {
            mutate_named(&mut changed, EvidenceSectionV2::Raster, "bytes", 0);
        }
        EvidenceMutationV2::WidthScalar => changed.width_witness.scalar ^= 1,
        EvidenceMutationV2::WidthDeterminant => changed.width_witness.determinant ^= 1,
        EvidenceMutationV2::WidthStride => changed.width_witness.stride ^= 1,
        EvidenceMutationV2::WidthDimension => changed.width_witness.dimension ^= 1,
        EvidenceMutationV2::WidthKey => changed.width_witness.key ^= 1,
        EvidenceMutationV2::WidthColor => changed.width_witness.color ^= 1,
        EvidenceMutationV2::CaseOrdinal => changed.cases[0].ordinal ^= 1,
        EvidenceMutationV2::CaseName => changed.cases[0].name = "changed-case",
        EvidenceMutationV2::ObservationCase => changed.cases[0].observations[0].case ^= 1,
        EvidenceMutationV2::ObservationStep => changed.cases[0].observations[0].step ^= 1,
        EvidenceMutationV2::ObservationGeneration => {
            let observation = &mut changed.cases[0].observations[0];
            observation.generation = Some(observation.generation.unwrap_or(0) + 1);
        }
        EvidenceMutationV2::ObservationViewportWidth => {
            changed.cases[0].observations[0].viewport.0 ^= 1;
        }
        EvidenceMutationV2::ObservationViewportHeight => {
            changed.cases[0].observations[0].viewport.1 ^= 1;
        }
        EvidenceMutationV2::LiteralMatch => changed.cases[0].result.literal_match = false,
        EvidenceMutationV2::ReferenceMatch => changed.cases[0].result.reference_match = false,
        EvidenceMutationV2::RepeatMatch => changed.cases[0].result.repeat_match = false,
        EvidenceMutationV2::SectionRecordCount => {
            changed.cases[0].observations[0].sections[0].record_count += 1;
        }
        EvidenceMutationV2::SectionByteCount => {
            changed.cases[0].observations[0].sections[0].byte_count += 1;
        }
        EvidenceMutationV2::SectionDigest => {
            changed.cases[0].observations[0].sections[0].digest ^= 1;
        }
        EvidenceMutationV2::SectionTag => {
            changed.cases[0].observations[0].sections[0].name = EvidenceSectionV2::Mapping;
        }
        EvidenceMutationV2::SectionEncoded => {
            let encoded = &mut changed.cases[0].observations[0].sections[0].encoded;
            flip(encoded, 0);
        }
    }
    changed
}

fn mutate_query_result(evidence: &mut SpatialEvidenceV2, current_tag: u8) {
    for case in &mut evidence.cases {
        for observation in &mut case.observations {
            let section = observation.section_mut(EvidenceSectionV2::Queries);
            for record in &mut section.records {
                if let Some(field) = record
                    .fields
                    .iter_mut()
                    .find(|field| field.name == "result-tag" && field.encoded == [current_tag])
                {
                    field.encoded[0] ^= 1;
                    section.recompute();
                    return;
                }
            }
        }
    }
    mutate_named(evidence, EvidenceSectionV2::Queries, "result-tag", 0);
}

fn mutate_at(
    evidence: &mut SpatialEvidenceV2,
    case: usize,
    step: usize,
    section: EvidenceSectionV2,
    record: usize,
    field: usize,
    byte: usize,
) {
    let section = evidence.cases[case].observations[step].section_mut(section);
    flip(&mut section.records[record].fields[field].encoded, byte);
    section.recompute();
}

fn mutate_named(
    evidence: &mut SpatialEvidenceV2,
    section: EvidenceSectionV2,
    field: &str,
    byte: usize,
) {
    mutate_nth_named(evidence, section, field, 0, byte);
}

fn mutate_nth_named(
    evidence: &mut SpatialEvidenceV2,
    section_name: EvidenceSectionV2,
    field_name: &str,
    mut nth: usize,
    byte: usize,
) {
    for case in &mut evidence.cases {
        for observation in &mut case.observations {
            let section = observation.section_mut(section_name);
            for record in &mut section.records {
                for field in &mut record.fields {
                    if field.name == field_name {
                        if nth == 0 {
                            flip(&mut field.encoded, byte);
                            section.recompute();
                            return;
                        }
                        nth -= 1;
                    }
                }
            }
        }
    }
    mutate_first_field(evidence, section_name);
}

fn mutate_first_field(evidence: &mut SpatialEvidenceV2, section_name: EvidenceSectionV2) {
    for case in &mut evidence.cases {
        for observation in &mut case.observations {
            let section = observation.section_mut(section_name);
            if let Some(field) = section
                .records
                .first_mut()
                .and_then(|record| record.fields.first_mut())
            {
                flip(&mut field.encoded, 0);
                section.recompute();
                return;
            }
        }
    }
}

enum Row {
    Swap,
    Remove,
    Duplicate,
}

fn mutate_rows(evidence: &mut SpatialEvidenceV2, section_name: EvidenceSectionV2, action: Row) {
    for case in &mut evidence.cases {
        for observation in &mut case.observations {
            let section = observation.section_mut(section_name);
            if section.records.is_empty() {
                continue;
            }
            match action {
                Row::Swap if section.records.len() >= 2 => section.records.swap(0, 1),
                Row::Swap => section.records.push(EvidenceRecordV2::new(Vec::new())),
                Row::Remove => {
                    section.records.remove(0);
                }
                Row::Duplicate => section.records.insert(0, section.records[0].clone()),
            }
            section.recompute();
            return;
        }
    }
    if let Some(observation) = evidence
        .cases
        .iter_mut()
        .flat_map(|case| &mut case.observations)
        .next()
    {
        let section = observation.section_mut(section_name);
        section.records.push(EvidenceRecordV2::new(Vec::new()));
        section.recompute();
    }
}

enum ByteMutation {
    First,
}

fn mutate_bytes(evidence: &mut SpatialEvidenceV2, position: ByteMutation) {
    for case in &mut evidence.cases {
        for observation in &mut case.observations {
            let section = observation.section_mut(EvidenceSectionV2::Raster);
            if let Some(field) = section
                .records
                .iter_mut()
                .flat_map(|record| &mut record.fields)
                .find(|field| field.name == "bytes" && field.encoded.len() > 8)
            {
                let index = match position {
                    ByteMutation::First => 8,
                };
                flip(&mut field.encoded, index);
                section.recompute();
                return;
            }
        }
    }
    mutate_named(evidence, EvidenceSectionV2::Raster, "bytes", 0);
}

fn flip(bytes: &mut Vec<u8>, index: usize) {
    if bytes.is_empty() {
        bytes.push(1);
    } else {
        let index = index.min(bytes.len() - 1);
        bytes[index] ^= 1;
    }
}
