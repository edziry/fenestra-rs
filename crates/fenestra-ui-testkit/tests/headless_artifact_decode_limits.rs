#[path = "headless/artifact_decode_support.rs"]
mod support;

use fenestra_ui_testkit::prototype::{
    HeadlessArtifactCountKindV1 as Count, HeadlessArtifactDecodeErrorKindV1 as Kind,
    HeadlessArtifactLimitKindV1 as Limit,
};

#[test]
fn limit_vocabulary_and_priority_are_closed() {
    assert_eq!(
        Limit::ALL,
        [
            Limit::ArtifactBytes,
            Limit::LineBytes,
            Limit::Lines,
            Limit::HeadlessEvents,
            Limit::HeadlessTraceBytes,
            Limit::SchedulerEvents,
            Limit::SchedulerTraceBytes,
            Limit::ComputedStyles,
            Limit::Geometry,
            Limit::Semantics,
            Limit::HitRegions,
            Limit::SceneRectangles,
            Limit::PathDepth,
        ]
    );
    assert_eq!(
        Count::ALL,
        [
            Count::HeadlessEvents,
            Count::HeadlessTraceBytes,
            Count::SchedulerEvents,
            Count::SchedulerTraceBytes,
            Count::ComputedStyles,
            Count::Geometry,
            Count::Semantics,
            Count::HitRegions,
            Count::SceneRectangles,
        ]
    );
}

#[test]
fn every_declared_storage_limit_is_inclusive_and_rejects_one_over() {
    let canonical = support::canonical_bytes();
    let cases = [
        declared(&canonical, "headless-trace-begin|", 1, "128"),
        declared(&canonical, "headless-trace-begin|", 2, "20480"),
        declared(&canonical, "scheduler-trace-begin|", 1, "256"),
        declared(&canonical, "scheduler-trace-begin|", 2, "24576"),
        declared(&canonical, "projection-begin|", 4, "8"),
        declared(&canonical, "projection-begin|", 5, "8"),
        declared(&canonical, "projection-begin|", 6, "1"),
        declared(&canonical, "projection-begin|", 7, "8"),
        declared(&canonical, "projection-begin|", 8, "8"),
    ];
    let counts = [
        Count::HeadlessEvents,
        Count::HeadlessTraceBytes,
        Count::SchedulerEvents,
        Count::SchedulerTraceBytes,
        Count::ComputedStyles,
        Count::Geometry,
        Count::Semantics,
        Count::HitRegions,
        Count::SceneRectangles,
    ];
    let lines = [15, 15, 72, 72, 115, 115, 115, 115, 115];
    for ((bytes, count), line) in cases.iter().zip(counts).zip(lines) {
        support::assert_decode_error(bytes, Kind::CountMismatch(count), Some(line));
    }

    let over = [
        declared(&canonical, "headless-trace-begin|", 1, "129"),
        declared(&canonical, "headless-trace-begin|", 2, "20481"),
        declared(&canonical, "scheduler-trace-begin|", 1, "257"),
        declared(&canonical, "scheduler-trace-begin|", 2, "24577"),
        declared(&canonical, "projection-begin|", 4, "9"),
        declared(&canonical, "projection-begin|", 5, "9"),
        declared(&canonical, "projection-begin|", 6, "2"),
        declared(&canonical, "projection-begin|", 7, "9"),
        declared(&canonical, "projection-begin|", 8, "9"),
    ];
    let limits = [
        Limit::HeadlessEvents,
        Limit::HeadlessTraceBytes,
        Limit::SchedulerEvents,
        Limit::SchedulerTraceBytes,
        Limit::ComputedStyles,
        Limit::Geometry,
        Limit::Semantics,
        Limit::HitRegions,
        Limit::SceneRectangles,
    ];
    for ((bytes, limit), line) in over.iter().zip(limits).zip(lines) {
        support::assert_decode_error(bytes, Kind::LimitExceeded(limit), Some(line));
    }
}

#[test]
fn actual_trace_counts_are_bounded_before_sequence_or_count_validation() {
    let canonical = support::canonical_bytes();
    let headless = support::lines(&canonical)[15].to_owned();
    for (count, expected) in [
        (128, Kind::CountMismatch(Count::HeadlessEvents)),
        (129, Kind::LimitExceeded(Limit::HeadlessEvents)),
    ] {
        let records = vec![headless.clone(); count];
        let bytes = support::replace_section(
            &canonical,
            "headless-trace-begin|",
            "headless-trace-end",
            "headless-trace-begin|55|8800",
            &records,
        );
        support::assert_decode_error(&bytes, expected, Some(15));
    }

    let scheduler = support::lines(&canonical)[72].to_owned();
    for (count, expected) in [
        (256, Kind::CountMismatch(Count::SchedulerEvents)),
        (257, Kind::LimitExceeded(Limit::SchedulerEvents)),
    ] {
        let records = vec![scheduler.clone(); count];
        let bytes = support::replace_section(
            &canonical,
            "scheduler-trace-begin|",
            "scheduler-trace-end",
            "scheduler-trace-begin|41|3936",
            &records,
        );
        support::assert_decode_error(&bytes, expected, Some(72));
    }
}

#[test]
fn trace_byte_markers_are_fixed_protocol_accounting_not_wire_text_lengths() {
    assert_eq!(55 * 160, 8_800);
    assert_eq!(41 * 96, 3_936);
    let canonical = support::canonical_bytes();
    let cases: [(Vec<u8>, Count, u32); 4] = [
        (
            declared(&canonical, "headless-trace-begin|", 2, "8799"),
            Count::HeadlessTraceBytes,
            15,
        ),
        (
            declared(&canonical, "headless-trace-begin|", 2, "8801"),
            Count::HeadlessTraceBytes,
            15,
        ),
        (
            declared(&canonical, "scheduler-trace-begin|", 2, "3935"),
            Count::SchedulerTraceBytes,
            72,
        ),
        (
            declared(&canonical, "scheduler-trace-begin|", 2, "3937"),
            Count::SchedulerTraceBytes,
            72,
        ),
    ];
    for (bytes, count, line) in &cases {
        support::assert_decode_error(bytes, Kind::CountMismatch(*count), Some(*line));
    }
}

#[test]
fn actual_projection_counts_are_bounded_before_allocation_and_mismatch() {
    let canonical = support::canonical_bytes();
    let cases = [
        projection_case(
            &canonical,
            "computed-begin",
            "computed-end",
            "computed|root|84|80|rgba8:090909ff|true|ignore",
            8,
            Limit::ComputedStyles,
            Count::ComputedStyles,
        ),
        projection_case(
            &canonical,
            "geometry-begin",
            "geometry-end",
            "geometry|root|0|0|84|80|0|0|84|70",
            8,
            Limit::Geometry,
            Count::Geometry,
        ),
        projection_case(
            &canonical,
            "semantic-begin",
            "semantic-end",
            "semantic|root/s:0/s:0|control|1|activate",
            1,
            Limit::Semantics,
            Count::Semantics,
        ),
        projection_case(
            &canonical,
            "hit-begin",
            "hit-end",
            "hit|root/s:0/m:1:10|0|10|40|12",
            8,
            Limit::HitRegions,
            Count::HitRegions,
        ),
        projection_case(
            &canonical,
            "scene-begin",
            "scene-end",
            "scene|root|0|0|84|70|rgba8:090909ff",
            8,
            Limit::SceneRectangles,
            Count::SceneRectangles,
        ),
    ];
    for case in &cases {
        support::assert_decode_error(&case.0, Kind::CountMismatch(case.3), Some(115));
        support::assert_decode_error(&case.1, Kind::LimitExceeded(case.2), Some(115));
    }
}

#[test]
fn path_depth_is_inclusive_and_preflighted_before_node_path_allocation() {
    let canonical = support::canonical_bytes();
    let exact = support::replace_once(
        &canonical,
        "computed|root|84|80",
        "computed|root/s:0/s:0/s:0|84|80",
    );
    assert_ne!(
        support::decode_error(&exact).kind(),
        Kind::LimitExceeded(Limit::PathDepth)
    );

    let over = support::replace_once(
        &canonical,
        "computed|root|84|80",
        "computed|root/s:0/s:0/s:0/s:0|84|80",
    );
    support::assert_decode_error(&over, Kind::LimitExceeded(Limit::PathDepth), Some(117));
}

#[test]
fn every_limit_phase_precedes_counts_and_counts_precede_references() {
    let canonical = support::canonical_bytes();
    let count_mismatch = support::set_field(&canonical, "headless-trace-begin|", 1, "56");
    let path_limit = support::replace_once(
        &count_mismatch,
        "computed|root|84|80",
        "computed|root/s:0/s:0/s:0/s:0|84|80",
    );
    support::assert_decode_error(
        &path_limit,
        Kind::LimitExceeded(Limit::PathDepth),
        Some(117),
    );

    let earlier_limit = support::set_field(&path_limit, "capacity-headless-trace|", 1, "129");
    support::assert_decode_error(
        &earlier_limit,
        Kind::LimitExceeded(Limit::HeadlessEvents),
        Some(13),
    );

    let invalid_reference = support::set_field(&count_mismatch, "h-event|1|0|", 2, "1");
    support::assert_decode_error(
        &invalid_reference,
        Kind::CountMismatch(Count::HeadlessEvents),
        Some(15),
    );

    let two_counts = support::set_field(&count_mismatch, "headless-trace-begin|", 2, "8801");
    support::assert_decode_error(
        &two_counts,
        Kind::CountMismatch(Count::HeadlessEvents),
        Some(15),
    );

    let artifact_over_headless = support::set_field(
        &support::set_field(&canonical, "capacity-headless-trace|", 1, "129"),
        "capacity-artifact|",
        1,
        "65537",
    );
    support::assert_decode_error(
        &artifact_over_headless,
        Kind::LimitExceeded(Limit::ArtifactBytes),
        Some(14),
    );

    let headless_over_geometry = support::set_field(
        &support::set_field(&canonical, "capacity-projection|", 2, "9"),
        "capacity-headless-trace|",
        1,
        "129",
    );
    support::assert_decode_error(
        &headless_over_geometry,
        Kind::LimitExceeded(Limit::HeadlessEvents),
        Some(13),
    );
}

fn declared(bytes: &[u8], prefix: &str, field: usize, value: &str) -> Vec<u8> {
    support::set_field(bytes, prefix, field, value)
}

fn projection_case(
    canonical: &[u8],
    begin: &str,
    end: &str,
    record: &str,
    exact_count: usize,
    limit: Limit,
    count: Count,
) -> (Vec<u8>, Vec<u8>, Limit, Count) {
    let exact = support::replace_section(
        canonical,
        begin,
        end,
        begin,
        &vec![record.to_owned(); exact_count],
    );
    let over = support::replace_section(
        canonical,
        begin,
        end,
        begin,
        &vec![record.to_owned(); exact_count + 1],
    );
    (exact, over, limit, count)
}
