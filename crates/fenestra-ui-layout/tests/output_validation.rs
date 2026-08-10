mod output_support;

use fenestra_ui_layout::prototype::{
    LayoutEngineErrorKindV1, LayoutEngineErrorV1, LayoutErrorKindV1, LayoutErrorLocationV1,
    LayoutExtentV1, LayoutOutputErrorKindV1, LayoutOutputFieldV1,
};

use output_support::{
    FakeOutputEngine, assert_output_error, output_record, record, rect, run, valid_records,
};

#[test]
fn valid_output_is_returned_after_exactly_one_engine_call() {
    let expected = valid_records();
    let engine = FakeOutputEngine::with_records(expected.clone());

    let output = run(&engine).expect("valid output should cross the boundary");

    assert_eq!(output.records(), expected);
}

#[test]
fn record_count_is_the_first_global_output_phase() {
    assert_output_error(
        vec![record(713, rect(-991, 0, 0, 0))],
        LayoutOutputErrorKindV1::RecordCountMismatch,
        LayoutErrorLocationV1::Output,
    );
    assert_output_error(
        vec![
            record(713, rect(-991, 0, 0, 0)),
            record(1, rect(0, 0, 0, 0)),
            record(2, rect(0, 0, 0, 0)),
        ],
        LayoutOutputErrorKindV1::RecordCountMismatch,
        LayoutErrorLocationV1::Output,
    );
}

#[test]
fn every_key_is_checked_before_any_rectangle_field() {
    assert_output_error(
        vec![record(1, rect(0, 0, 0, 0)), record(0, rect(0, 0, 0, 0))],
        LayoutOutputErrorKindV1::KeyMismatch,
        output_record(0),
    );
    assert_output_error(
        vec![record(0, rect(-991, 0, 0, 0)), record(0, rect(0, 0, 0, 0))],
        LayoutOutputErrorKindV1::KeyMismatch,
        output_record(1),
    );
    assert_output_error(
        vec![
            record(0, rect(0, 0, 0, 0)),
            record(u32::MAX, rect(0, 0, 0, 0)),
        ],
        LayoutOutputErrorKindV1::KeyMismatch,
        output_record(1),
    );
}

#[test]
fn negative_fields_report_the_exact_field_and_record() {
    let cases = [
        (
            rect(-11, 0, 0, 0),
            LayoutOutputErrorKindV1::Negative(LayoutOutputFieldV1::X),
        ),
        (
            rect(0, -12, 0, 0),
            LayoutOutputErrorKindV1::Negative(LayoutOutputFieldV1::Y),
        ),
        (
            rect(0, 0, -13, 0),
            LayoutOutputErrorKindV1::Negative(LayoutOutputFieldV1::Width),
        ),
        (
            rect(0, 0, 0, -14),
            LayoutOutputErrorKindV1::Negative(LayoutOutputFieldV1::Height),
        ),
    ];

    for (bounds, expected) in cases {
        assert_output_error(
            vec![record(0, rect(0, 0, 0, 0)), record(1, bounds)],
            expected,
            output_record(1),
        );
    }
}

#[test]
fn negative_validation_uses_record_then_field_order() {
    assert_output_error(
        vec![
            record(0, rect(-11, -12, -13, -14)),
            record(1, rect(0, 0, 0, 0)),
        ],
        LayoutOutputErrorKindV1::Negative(LayoutOutputFieldV1::X),
        output_record(0),
    );
    assert_output_error(
        vec![record(0, rect(0, 0, 0, -14)), record(1, rect(-11, 0, 0, 0))],
        LayoutOutputErrorKindV1::Negative(LayoutOutputFieldV1::Height),
        output_record(0),
    );
}

#[test]
fn all_negative_fields_are_checked_before_any_far_edge() {
    assert_output_error(
        vec![
            record(0, rect(i32::MAX, 0, 1, 0)),
            record(1, rect(-11, 0, 0, 0)),
        ],
        LayoutOutputErrorKindV1::Negative(LayoutOutputFieldV1::X),
        output_record(1),
    );
}

#[test]
fn far_edges_report_the_exact_extent_and_record() {
    let cases = [
        (
            vec![
                record(0, rect(i32::MAX, 0, 1, 0)),
                record(1, rect(0, 0, 0, 0)),
            ],
            LayoutOutputErrorKindV1::FarEdgeArithmetic(LayoutExtentV1::Width),
            output_record(0),
        ),
        (
            vec![
                record(0, rect(0, i32::MAX, 0, 1)),
                record(1, rect(0, 0, 0, 0)),
            ],
            LayoutOutputErrorKindV1::FarEdgeArithmetic(LayoutExtentV1::Height),
            output_record(0),
        ),
        (
            vec![
                record(0, rect(i32::MAX, i32::MAX, 1, 1)),
                record(1, rect(0, 0, 0, 0)),
            ],
            LayoutOutputErrorKindV1::FarEdgeArithmetic(LayoutExtentV1::Width),
            output_record(0),
        ),
        (
            vec![
                record(0, rect(0, i32::MAX, 0, 1)),
                record(1, rect(i32::MAX, 0, 1, 0)),
            ],
            LayoutOutputErrorKindV1::FarEdgeArithmetic(LayoutExtentV1::Height),
            output_record(0),
        ),
    ];

    for (records, expected_kind, expected_location) in cases {
        assert_output_error(records, expected_kind, expected_location);
    }
}

#[test]
fn full_i32_far_edge_is_inclusive() {
    let records = vec![
        record(0, rect(i32::MAX - 1, i32::MAX - 1, 1, 1)),
        record(1, rect(0, 0, 0, 0)),
    ];
    let engine = FakeOutputEngine::with_records(records.clone());

    let output = run(&engine).expect("an exact i32 far edge should be accepted");

    assert_eq!(output.records(), records);
}

#[test]
fn candidate_output_edge_profile_does_not_leak_into_core_validation() {
    let records = vec![
        record(0, rect(524_287, 0, 1, 0)),
        record(1, rect(524_288, 0, 1, 0)),
    ];
    let engine = FakeOutputEngine::with_records(records.clone());

    let output = run(&engine).expect("candidate-only edge limits must not constrain core output");

    assert_eq!(output.records(), records);
}

#[test]
fn engine_failures_are_forwarded_after_exactly_one_call() {
    let expected =
        LayoutEngineErrorV1::new(LayoutEngineErrorKindV1::RejectedInput, output_record(1));
    let engine = FakeOutputEngine::with_error(expected);

    let error = run(&engine).expect_err("engine failure should cross the boundary");

    assert_eq!(
        error.kind(),
        LayoutErrorKindV1::Engine(LayoutEngineErrorKindV1::RejectedInput)
    );
    assert_eq!(error.location(), output_record(1));
}

#[test]
fn output_errors_are_privacy_safe_and_use_output_ordinals() {
    let error = assert_output_error(
        vec![
            record(0, rect(-991, 0, 0, 0)),
            record(713, rect(0, 0, 0, 0)),
        ],
        LayoutOutputErrorKindV1::KeyMismatch,
        output_record(1),
    );
    let rendered = format!("{error:?} {error}");

    for forbidden in [
        "713",
        "-991",
        "LayoutOutputV1",
        "LayoutRecordV1",
        "LayoutRectV1",
    ] {
        assert!(
            !rendered.contains(forbidden),
            "output error leaked {forbidden}: {rendered}"
        );
    }
}
