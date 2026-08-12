use crate::baseline::{
    ControlFamilyV2, EvidenceMutationV2, EvidenceSectionV2, compare_evidence_v2, control_report_v2,
    mutate_evidence_v2, reconstruct_literal_v2,
};

use super::support::expected::CONTROL_FAMILIES;

#[test]
fn all_seven_control_families_are_complete_and_detected() {
    let evidence = reconstruct_literal_v2().expect("literal evidence");
    let report = control_report_v2(&evidence);
    assert_eq!(report.len(), CONTROL_FAMILIES.len());
    let minimum = [12, 3, 10, 7, 7, 74, 10];
    for ((control, family), minimum) in report.iter().zip(CONTROL_FAMILIES).zip(minimum) {
        assert_eq!(control.family.token(), family);
        assert!(
            control.registered >= minimum,
            "insufficient {family} controls"
        );
        assert_eq!(
            control.detected, control.registered,
            "{family} controls must all report the exact first mismatch"
        );
        assert!(control.exact_first_location);
    }
}

#[test]
fn metadata_record_field_query_and_raster_control_shapes_are_closed() {
    let evidence = reconstruct_literal_v2().expect("literal evidence");
    let report = control_report_v2(&evidence);
    let metadata = report
        .iter()
        .find(|item| item.family == ControlFamilyV2::Metadata)
        .unwrap();
    assert!(metadata.coverage.tags);
    assert!(metadata.coverage.scalars);
    assert!(metadata.coverage.keys);
    assert!(metadata.coverage.options);
    assert!(metadata.coverage.counts);
    assert!(metadata.coverage.bytes);
    assert!(metadata.coverage.metadata);

    let records = report
        .iter()
        .find(|item| item.family == ControlFamilyV2::Records)
        .unwrap();
    assert!(records.coverage.adjacent_swaps);
    assert!(records.coverage.removals);
    assert!(records.coverage.duplicates);

    let queries = report
        .iter()
        .find(|item| item.family == ControlFamilyV2::Queries)
        .unwrap();
    assert!(queries.coverage.hit_to_miss);
    assert!(queries.coverage.miss_to_hit);
    assert!(queries.coverage.hit_fields);
    assert!(queries.coverage.local_coordinates);

    let raster = report
        .iter()
        .find(|item| item.family == ControlFamilyV2::Raster)
        .unwrap();
    assert!(raster.coverage.raster_dimensions);
    assert!(raster.coverage.raster_stride);
    assert!(raster.coverage.raster_length);
    assert!(raster.coverage.raster_first_middle_last);
}

#[test]
fn recursive_comparison_reports_the_first_section_record_and_field() {
    let expected = reconstruct_literal_v2().expect("literal evidence");
    let probes = [
        (
            EvidenceMutationV2::ReceiptGeneration,
            EvidenceSectionV2::Receipt,
            0,
            "generation",
        ),
        (
            EvidenceMutationV2::GeometryAndRaster,
            EvidenceSectionV2::Geometry,
            0,
            "world-aabb-min-x",
        ),
        (
            EvidenceMutationV2::QueryHitToMiss,
            EvidenceSectionV2::Queries,
            2,
            "result-tag",
        ),
        (
            EvidenceMutationV2::RasterFirstByte,
            EvidenceSectionV2::Raster,
            0,
            "bytes[0]",
        ),
    ];
    for (mutation, section, record, field) in probes {
        let actual = mutate_evidence_v2(&expected, mutation);
        let mismatch = compare_evidence_v2(&expected, &actual).expect_err("mutation must differ");
        assert_eq!(mismatch.section, section, "{mutation:?}");
        assert_eq!(mismatch.record, record, "{mutation:?}");
        assert_eq!(mismatch.field, field, "{mutation:?}");
    }
}

#[test]
fn ordered_section_row_mutations_cannot_hide() {
    let expected = reconstruct_literal_v2().expect("literal evidence");
    for section in EvidenceSectionV2::ALL {
        for mutation in [
            EvidenceMutationV2::SwapAdjacent(section),
            EvidenceMutationV2::RemoveRow(section),
            EvidenceMutationV2::DuplicateRow(section),
        ] {
            let actual = mutate_evidence_v2(&expected, mutation);
            let mismatch = compare_evidence_v2(&expected, &actual).expect_err("row mutation");
            assert_eq!(mismatch.section, section);
        }
    }
}
