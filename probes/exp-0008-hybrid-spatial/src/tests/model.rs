use crate::baseline::{
    EvidenceSectionV2, compare_evidence_v2, reconstruct_literal_v2, reconstruct_reference_v2,
};

use super::support::artifact::fnv1a64;
use super::support::expected::{CASE_NAMES, OBSERVATION_COUNTS, SECTION_NAMES};

#[test]
fn literal_and_reference_are_four_fresh_complete_equal_reconstructions() {
    let literal_a = reconstruct_literal_v2().expect("first literal reconstruction");
    let literal_b = reconstruct_literal_v2().expect("second literal reconstruction");
    let reference_a = reconstruct_reference_v2().expect("first reference reconstruction");
    let reference_b = reconstruct_reference_v2().expect("second reference reconstruction");

    assert_eq!(literal_a, literal_b);
    assert_eq!(reference_a, reference_b);
    assert_eq!(literal_a, reference_a);
    assert_eq!(literal_b, reference_b);
    compare_evidence_v2(&literal_a, &literal_b).expect("literal repeat");
    compare_evidence_v2(&reference_a, &reference_b).expect("reference repeat");
    compare_evidence_v2(&literal_a, &reference_a).expect("first cross-build pair");
    compare_evidence_v2(&literal_b, &reference_b).expect("second cross-build pair");
}

#[test]
fn normalized_observations_have_the_exact_owned_section_order() {
    let evidence = reconstruct_literal_v2().expect("literal evidence");
    assert_eq!(evidence.cases.len(), CASE_NAMES.len());
    for (ordinal, case) in evidence.cases.iter().enumerate() {
        assert_eq!(case.ordinal as usize, ordinal);
        assert_eq!(case.name, CASE_NAMES[ordinal]);
        assert_eq!(case.observations.len(), OBSERVATION_COUNTS[ordinal]);
        assert!(case.result.literal_match);
        assert!(case.result.reference_match);
        assert!(case.result.repeat_match);
        for (step, observation) in case.observations.iter().enumerate() {
            assert_eq!(observation.case as usize, ordinal);
            assert_eq!(observation.step as usize, step);
            assert_eq!(observation.sections.len(), SECTION_NAMES.len());
            let names = observation
                .sections
                .iter()
                .map(|section| section.name.token())
                .collect::<Vec<_>>();
            assert_eq!(names, SECTION_NAMES);
            for section in &observation.sections {
                assert_eq!(section.byte_count as usize, section.encoded.len());
                assert_eq!(
                    section.digest,
                    fnv1a64(section.name.token(), &section.encoded)
                );
                assert!(section.record_count <= u64::from(u32::MAX));
            }
        }
    }
}

#[test]
fn receipt_mapping_and_runtime_generations_remain_distinct() {
    let evidence = reconstruct_reference_v2().expect("reference evidence");
    for case in &evidence.cases[..12] {
        for observation in &case.observations {
            assert_eq!(observation.generation, None);
            assert_eq!(
                observation.section(EvidenceSectionV2::Mapping).record_count,
                0
            );
        }
    }
    let runtime = &evidence.cases[12];
    assert_eq!(
        runtime
            .observations
            .iter()
            .map(|item| item.generation)
            .collect::<Vec<_>>(),
        [
            Some(0),
            Some(1),
            Some(2),
            Some(3),
            Some(4),
            Some(5),
            Some(6),
            Some(7),
            Some(8),
        ]
    );
    assert!(
        runtime
            .observations
            .iter()
            .all(|item| item.section(EvidenceSectionV2::Mapping).record_count > 0)
    );
    assert_eq!(evidence.cases[13].observations[0].generation, Some(8));
}

#[test]
fn normalization_retains_declared_integer_widths_and_complete_raster_bytes() {
    let evidence = reconstruct_literal_v2().expect("literal evidence");
    let widths = evidence.width_witness;
    let _: i64 = widths.scalar;
    let _: i128 = widths.determinant;
    let _: u64 = widths.stride;
    let _: u32 = widths.dimension;
    let _: u32 = widths.key;
    let _: u8 = widths.color;
    assert!(
        evidence
            .cases
            .iter()
            .flat_map(|case| &case.observations)
            .all(|observation| {
                let raster = observation.section(EvidenceSectionV2::Raster);
                raster.record_count == 1 && !raster.encoded.is_empty()
            })
    );
}
