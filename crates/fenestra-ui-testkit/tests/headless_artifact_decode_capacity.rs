#[path = "headless/artifact_decode_support.rs"]
mod support;

use fenestra_ui_testkit::prototype::{
    HeadlessArtifactDecodeErrorKindV1 as Kind, HeadlessArtifactLimitKindV1 as Limit,
    encode_headless_artifact_v1,
};

#[test]
fn capacity_rows_reject_storage_below_actual_use() {
    let canonical = support::canonical_bytes();
    let lower = [
        capacity(&canonical, "capacity-headless-trace|", 1, "54"),
        capacity(&canonical, "capacity-headless-trace|", 2, "8799"),
        capacity(&canonical, "capacity-scheduler-trace|", 1, "40"),
        capacity(&canonical, "capacity-scheduler-trace|", 2, "3935"),
        capacity(&canonical, "capacity-projection|", 1, "4"),
        capacity(&canonical, "capacity-projection|", 2, "4"),
        capacity(&canonical, "capacity-projection|", 4, "1"),
        capacity(&canonical, "capacity-projection|", 5, "3"),
        capacity(&canonical, "capacity-ir|", 8, "1"),
        capacity(
            &canonical,
            "capacity-artifact|",
            1,
            &(canonical.len() - 1).to_string(),
        ),
        capacity(
            &canonical,
            "capacity-artifact|",
            2,
            &(max_line_bytes(&canonical) - 1).to_string(),
        ),
        capacity(&canonical, "capacity-artifact|", 3, "143"),
    ];
    let limits = [
        Limit::HeadlessEvents,
        Limit::HeadlessTraceBytes,
        Limit::SchedulerEvents,
        Limit::SchedulerTraceBytes,
        Limit::ComputedStyles,
        Limit::Geometry,
        Limit::HitRegions,
        Limit::SceneRectangles,
        Limit::PathDepth,
        Limit::ArtifactBytes,
        Limit::LineBytes,
        Limit::Lines,
    ];
    let lines = [13, 13, 12, 12, 9, 9, 9, 9, 6, 14, 14, 14];
    for ((bytes, limit), line) in lower.iter().zip(limits).zip(lines) {
        support::assert_decode_error(bytes, Kind::LimitExceeded(limit), Some(line));
    }
}

#[test]
fn capacity_rows_reject_values_above_the_hard_ceiling() {
    let canonical = support::canonical_bytes();
    let above_hard = [
        capacity(&canonical, "capacity-headless-trace|", 1, "129"),
        capacity(&canonical, "capacity-headless-trace|", 2, "20481"),
        capacity(&canonical, "capacity-scheduler-trace|", 1, "257"),
        capacity(&canonical, "capacity-scheduler-trace|", 2, "24577"),
        capacity(&canonical, "capacity-projection|", 1, "9"),
        capacity(&canonical, "capacity-projection|", 2, "9"),
        capacity(&canonical, "capacity-projection|", 3, "2"),
        capacity(&canonical, "capacity-projection|", 4, "9"),
        capacity(&canonical, "capacity-projection|", 5, "9"),
        capacity(&canonical, "capacity-ir|", 8, "4"),
        capacity(&canonical, "capacity-artifact|", 1, "65537"),
        capacity(&canonical, "capacity-artifact|", 2, "1025"),
        capacity(&canonical, "capacity-artifact|", 3, "513"),
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
        Limit::PathDepth,
        Limit::ArtifactBytes,
        Limit::LineBytes,
        Limit::Lines,
    ];
    let lines = [13, 13, 12, 12, 9, 9, 9, 9, 9, 6, 14, 14, 14];
    for ((bytes, limit), line) in above_hard.iter().zip(limits).zip(lines) {
        support::assert_decode_error(bytes, Kind::LimitExceeded(limit), Some(line));
    }
}

#[test]
fn valid_nonregistered_capacity_rows_roundtrip_without_normalization() {
    let canonical = support::canonical_bytes();
    let mut valid = capacity(&canonical, "capacity-headless-trace|", 1, "56");
    valid = capacity(&valid, "capacity-headless-trace|", 2, "9000");
    valid = capacity(&valid, "capacity-scheduler-trace|", 1, "42");
    valid = capacity(&valid, "capacity-scheduler-trace|", 2, "4000");
    valid = capacity(&valid, "capacity-projection|", 1, "6");
    valid = capacity(&valid, "capacity-projection|", 2, "6");
    valid = capacity(&valid, "capacity-projection|", 3, "0");
    valid = capacity(&valid, "capacity-projection|", 4, "3");
    valid = capacity(&valid, "capacity-projection|", 5, "5");
    for (field, value) in [
        (1, "0"),
        (2, "4"),
        (3, "3"),
        (4, "0"),
        (5, "2"),
        (6, "11"),
        (7, "1"),
        (8, "2"),
        (9, "4"),
    ] {
        valid = capacity(&valid, "capacity-ir|", field, value);
    }
    valid = capacity(&valid, "capacity-style|", 1, "1");
    for (field, value) in [(1, "7"), (2, "7"), (3, "7"), (4, "1"), (5, "39"), (6, "2")] {
        valid = capacity(&valid, "capacity-runtime|", field, value);
    }
    for (field, value) in [
        (3, "7"),
        (4, "3"),
        (5, "127"),
        (6, "7"),
        (9, "7"),
        (12, "7"),
    ] {
        valid = capacity(&valid, "capacity-scheduler|", field, value);
    }
    valid = capacity(&valid, "capacity-renderer|", 3, "7");
    valid = capacity(&valid, "capacity-artifact|", 1, "65000");
    valid = capacity(&valid, "capacity-artifact|", 2, "1000");
    valid = capacity(&valid, "capacity-artifact|", 3, "500");
    let decoded = support::decode(&valid);
    assert_eq!(
        encode_headless_artifact_v1(&decoded)
            .expect("valid nonregistered capacities should encode"),
        valid
    );
}

fn capacity(bytes: &[u8], prefix: &str, field: usize, value: &str) -> Vec<u8> {
    support::set_field(bytes, prefix, field, value)
}

fn max_line_bytes(bytes: &[u8]) -> usize {
    support::lines(bytes)
        .iter()
        .map(|line| line.len())
        .max()
        .expect("canonical artifact has lines")
}
