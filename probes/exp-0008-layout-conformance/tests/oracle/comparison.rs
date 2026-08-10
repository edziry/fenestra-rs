use fenestra_ui_exp_0008_layout_conformance::prototype::{
    LayoutRecordMismatchKindV1, LayoutRecordMismatchV1, compare_layout_records_v1,
};
use fenestra_ui_layout::prototype::LayoutRecordV1;

use super::support::record;

#[test]
fn mismatch_kind_vocabulary_is_closed_and_ordered() {
    assert_eq!(
        LayoutRecordMismatchKindV1::ALL,
        [
            LayoutRecordMismatchKindV1::Count,
            LayoutRecordMismatchKindV1::Key,
            LayoutRecordMismatchKindV1::X,
            LayoutRecordMismatchKindV1::Y,
            LayoutRecordMismatchKindV1::Width,
            LayoutRecordMismatchKindV1::Height,
        ]
    );
}

#[test]
fn identical_records_have_no_mismatch() {
    let records = fixture_records();

    assert!(compare_layout_records_v1(&records, &records).is_none());
}

#[test]
fn count_order_and_key_controls_report_the_first_affected_ordinal() {
    let expected = fixture_records();

    let mut shorter = expected.clone();
    shorter.pop();
    assert_mismatch(
        compare_layout_records_v1(&expected, &shorter),
        LayoutRecordMismatchKindV1::Count,
        2,
    );

    let mut longer = expected.clone();
    longer.push(record(3, 100, 100, 1, 1));
    assert_mismatch(
        compare_layout_records_v1(&expected, &longer),
        LayoutRecordMismatchKindV1::Count,
        3,
    );

    let mut reordered = expected.clone();
    reordered.swap(0, 1);
    assert_mismatch(
        compare_layout_records_v1(&expected, &reordered),
        LayoutRecordMismatchKindV1::Key,
        0,
    );

    let mut changed_key = expected.clone();
    let bounds = changed_key[1].bounds();
    changed_key[1] = record(99, bounds.x(), bounds.y(), bounds.width(), bounds.height());
    assert_mismatch(
        compare_layout_records_v1(&expected, &changed_key),
        LayoutRecordMismatchKindV1::Key,
        1,
    );
}

#[test]
fn count_precedes_payload_and_records_are_compared_record_major() {
    let expected = fixture_records();

    let mut shorter_with_changed_key = expected.clone();
    shorter_with_changed_key.pop();
    shorter_with_changed_key[0] = record(99, 0, 0, 100, 80);
    assert_mismatch(
        compare_layout_records_v1(&expected, &shorter_with_changed_key),
        LayoutRecordMismatchKindV1::Count,
        2,
    );

    let mut record_major = expected.clone();
    record_major[0] = record(0, 0, 0, 100, 81);
    record_major[1] = record(99, 7, 11, 30, 20);
    assert_mismatch(
        compare_layout_records_v1(&expected, &record_major),
        LayoutRecordMismatchKindV1::Height,
        0,
    );

    let mut same_record = expected.clone();
    same_record[1] = record(99, 8, 12, 31, 21);
    assert_mismatch(
        compare_layout_records_v1(&expected, &same_record),
        LayoutRecordMismatchKindV1::Key,
        1,
    );

    same_record[1] = record(1, 8, 12, 31, 21);
    assert_mismatch(
        compare_layout_records_v1(&expected, &same_record),
        LayoutRecordMismatchKindV1::X,
        1,
    );

    same_record[1] = record(1, 7, 12, 31, 21);
    assert_mismatch(
        compare_layout_records_v1(&expected, &same_record),
        LayoutRecordMismatchKindV1::Y,
        1,
    );

    same_record[1] = record(1, 7, 11, 31, 21);
    assert_mismatch(
        compare_layout_records_v1(&expected, &same_record),
        LayoutRecordMismatchKindV1::Width,
        1,
    );
}

#[test]
fn coordinate_and_extent_controls_report_the_exact_field_and_ordinal() {
    let expected = fixture_records();

    let mut changed_x = expected.clone();
    let record_one = changed_x[1];
    let bounds = record_one.bounds();
    changed_x[1] = record(
        record_one.key().get(),
        bounds.x() + 1,
        bounds.y(),
        bounds.width(),
        bounds.height(),
    );
    assert_mismatch(
        compare_layout_records_v1(&expected, &changed_x),
        LayoutRecordMismatchKindV1::X,
        1,
    );

    let mut changed_y = expected.clone();
    let record_one = changed_y[1];
    let bounds = record_one.bounds();
    changed_y[1] = record(
        record_one.key().get(),
        bounds.x(),
        bounds.y() + 1,
        bounds.width(),
        bounds.height(),
    );
    assert_mismatch(
        compare_layout_records_v1(&expected, &changed_y),
        LayoutRecordMismatchKindV1::Y,
        1,
    );

    let mut changed_width = expected.clone();
    let record_one = changed_width[1];
    let bounds = record_one.bounds();
    changed_width[1] = record(
        record_one.key().get(),
        bounds.x(),
        bounds.y(),
        bounds.width() + 1,
        bounds.height(),
    );
    assert_mismatch(
        compare_layout_records_v1(&expected, &changed_width),
        LayoutRecordMismatchKindV1::Width,
        1,
    );

    let mut changed_height = expected.clone();
    let record_one = changed_height[1];
    let bounds = record_one.bounds();
    changed_height[1] = record(
        record_one.key().get(),
        bounds.x(),
        bounds.y(),
        bounds.width(),
        bounds.height() + 1,
    );
    assert_mismatch(
        compare_layout_records_v1(&expected, &changed_height),
        LayoutRecordMismatchKindV1::Height,
        1,
    );
}

#[test]
fn mismatch_diagnostics_do_not_expose_record_payloads() {
    let expected = [record(65_005, 41_001, 42_002, 43_003, 44_004)];
    let observed = [record(65_005, 91_001, 42_002, 43_003, 44_004)];
    let mismatch = compare_layout_records_v1(&expected, &observed)
        .expect("the changed x coordinate must be detected");
    let debug = format!("{mismatch:?}");
    let display = mismatch.to_string();

    for payload in ["41001", "42002", "43003", "44004", "65005", "91001"] {
        assert!(!debug.contains(payload), "Debug exposed payload {payload}");
        assert!(
            !display.contains(payload),
            "Display exposed payload {payload}"
        );
    }
}

fn assert_mismatch(
    mismatch: Option<LayoutRecordMismatchV1>,
    expected_kind: LayoutRecordMismatchKindV1,
    expected_ordinal: usize,
) {
    let mismatch = mismatch.expect("one independently mutated field must be detected");
    assert_eq!(mismatch.kind(), expected_kind);
    assert_eq!(mismatch.ordinal(), expected_ordinal);
}

fn fixture_records() -> Vec<LayoutRecordV1> {
    vec![
        record(0, 0, 0, 100, 80),
        record(1, 7, 11, 30, 20),
        record(2, 7, 31, 40, 12),
    ]
}
