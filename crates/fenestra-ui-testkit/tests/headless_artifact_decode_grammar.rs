#[path = "headless/artifact_decode_support.rs"]
mod support;

use fenestra_ui_testkit::prototype::{
    HeadlessArtifactDecodeErrorKindV1 as Kind, HeadlessArtifactSectionKindV1 as Section,
    HeadlessArtifactVersionKindV1 as Version,
};

#[test]
fn section_vocabulary_and_order_are_closed() {
    assert_eq!(
        Section::ALL,
        [
            Section::Header,
            Section::Versions,
            Section::Fixture,
            Section::Environment,
            Section::ProjectionChoices,
            Section::Capacities,
            Section::HeadlessTrace,
            Section::SchedulerTrace,
            Section::Projection,
            Section::ComputedStyles,
            Section::Geometry,
            Section::Semantics,
            Section::HitRegions,
            Section::SceneRectangles,
            Section::Result,
            Section::End,
        ]
    );
}

#[test]
fn every_declared_version_is_closed_and_line_anchored() {
    let canonical = support::canonical_bytes();
    let cases = [
        (
            "fenestra-headless-spine|1",
            "fenestra-headless-spine|2",
            Version::Envelope,
            1,
        ),
        ("fixture|1|schema", "fixture|2|schema", Version::Fixture, 2),
        (
            "schema|1|construction",
            "schema|2|construction",
            Version::Schema,
            2,
        ),
        (
            "construction|1|style",
            "construction|2|style",
            Version::Construction,
            2,
        ),
        ("style|1|trace", "style|2|trace", Version::Style, 2),
        (
            "trace|1|projection",
            "trace|2|projection",
            Version::Trace,
            2,
        ),
        ("projection|1\n", "projection|2\n", Version::Projection, 2),
    ];
    for (from, to, version, line) in cases {
        let bytes = support::replace_once(&canonical, from, to);
        support::assert_decode_error(&bytes, Kind::UnsupportedVersion(version), Some(line));
    }

    let headless_event = support::set_field(&canonical, "h-event|1|0|", 1, "2");
    support::assert_decode_error(
        &headless_event,
        Kind::UnsupportedVersion(Version::Trace),
        Some(16),
    );
    let scheduler_event = support::set_field(&canonical, "s-event|1|0|", 1, "2");
    support::assert_decode_error(
        &scheduler_event,
        Kind::UnsupportedVersion(Version::Trace),
        Some(73),
    );
}

#[test]
fn closed_tags_arities_enums_and_step_auxiliaries_are_malformed() {
    let canonical = support::canonical_bytes();
    for (bytes, line) in [
        (
            support::replace_once(&canonical, "h-event|1|0", "future-event|1|0"),
            16,
        ),
        (
            support::replace_once(
                &canonical,
                "h-event|1|0|8001|0|build|none|observed",
                "h-event|1|0|8001|0|future|none|observed",
            ),
            16,
        ),
        (
            support::replace_line(
                &canonical,
                "h-event|1|0|",
                &format!("{}|extra", support::lines(&canonical)[15]),
            ),
            16,
        ),
        (support::set_field(&canonical, "s-event|1|16|", 6, "-"), 89),
        (support::set_field(&canonical, "s-event|1|25|", 7, "1"), 98),
        (
            support::replace_once(
                &canonical,
                "projection-choices|full|vertical|rebuilt|reverse",
                "projection-choices|partial|vertical|rebuilt|reverse",
            ),
            5,
        ),
        (
            support::set_field(&canonical, "capacity-scheduler-trace|", 3, "95"),
            12,
        ),
        (
            support::set_field(&canonical, "capacity-headless-trace|", 3, "159"),
            13,
        ),
        (
            support::replace_once(&canonical, "result|pass", "result|future"),
            143,
        ),
    ] {
        support::assert_decode_error(&bytes, Kind::MalformedRecord, Some(line));
    }
}

#[test]
fn every_integer_uses_one_canonical_decimal_spelling() {
    let canonical = support::canonical_bytes();
    for bytes in [
        support::replace_once(
            &canonical,
            "fenestra-headless-spine|1",
            "fenestra-headless-spine|01",
        ),
        support::replace_once(
            &canonical,
            "fenestra-headless-spine|1",
            "fenestra-headless-spine|+1",
        ),
    ] {
        support::assert_decode_error(&bytes, Kind::NonCanonicalValue, Some(1));
    }

    let overflow = support::replace_once(
        &canonical,
        "fixture|headless-spine|1|1|8001",
        "fixture|headless-spine|1|1|18446744073709551616",
    );
    support::assert_decode_error(&overflow, Kind::NonCanonicalValue, Some(3));

    let negative_zero = support::replace_once(
        &canonical,
        "geometry|root|0|0|84|80",
        "geometry|root|-0|0|84|80",
    );
    support::assert_decode_error(&negative_zero, Kind::NonCanonicalValue, Some(124));

    let static_path = support::replace_once(
        &canonical,
        "computed|root/s:0|80|50",
        "computed|root/s:00|80|50",
    );
    support::assert_decode_error(&static_path, Kind::NonCanonicalValue, Some(118));
    let member_path = support::replace_once(
        &canonical,
        "computed|root/s:0/m:1:10|40|12",
        "computed|root/s:0/m:01:010|40|12",
    );
    support::assert_decode_error(&member_path, Kind::NonCanonicalValue, Some(120));

    for (bytes, line) in [
        (support::set_field(&canonical, "capacity-ir|", 1, "01"), 6),
        (support::set_field(&canonical, "h-event|1|0|", 4, "00"), 16),
        (
            support::set_field(&canonical, "s-event|1|40|", 29, "00:1"),
            113,
        ),
    ] {
        support::assert_decode_error(&bytes, Kind::NonCanonicalValue, Some(line));
    }
}

#[test]
fn missing_duplicate_ordered_and_trailing_sections_are_distinct() {
    let canonical = support::canonical_bytes();
    let missing = support::remove_line(&canonical, "computed-begin");
    support::assert_decode_error(
        &missing,
        Kind::MissingSection(Section::ComputedStyles),
        Some(116),
    );

    let duplicate = support::duplicate_line(&canonical, "computed-begin");
    support::assert_decode_error(
        &duplicate,
        Kind::DuplicateSection(Section::ComputedStyles),
        Some(117),
    );

    let ordered = support::replace_once(
        &support::remove_line(&canonical, "geometry-begin"),
        "computed-begin\n",
        "geometry-begin\ncomputed-begin\n",
    );
    support::assert_decode_error(&ordered, Kind::OrderingViolation, Some(116));

    let mut trailing = canonical;
    trailing.extend_from_slice(b"future|private-payload\n");
    support::assert_decode_error(&trailing, Kind::TrailingData, Some(145));
}

#[test]
fn grammar_versions_and_canonical_values_follow_global_priority() {
    let canonical = support::canonical_bytes();
    let grammar_over_version = support::replace_once(
        &support::replace_once(
            &canonical,
            "fenestra-headless-spine|1",
            "fenestra-headless-spine|2",
        ),
        "h-event|1|0",
        "future-event|1|0",
    );
    support::assert_decode_error(&grammar_over_version, Kind::MalformedRecord, Some(16));

    let version_over_canonical = support::replace_once(
        &support::replace_once(
            &canonical,
            "fenestra-headless-spine|1",
            "fenestra-headless-spine|01",
        ),
        "trace|1|projection",
        "trace|2|projection",
    );
    support::assert_decode_error(
        &version_over_canonical,
        Kind::UnsupportedVersion(Version::Trace),
        Some(2),
    );

    let canonical_over_limit = support::replace_once(
        &support::set_field(&canonical, "capacity-headless-trace|", 1, "129"),
        "fenestra-headless-spine|1",
        "fenestra-headless-spine|01",
    );
    support::assert_decode_error(&canonical_over_limit, Kind::NonCanonicalValue, Some(1));
}
