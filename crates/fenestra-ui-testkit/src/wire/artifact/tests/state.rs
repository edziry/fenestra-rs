use super::support::{FIXTURE, HEADER, VERSIONS, assert_error, records};
use crate::wire::error::{ArtifactDecodeErrorKind, SectionKind, VersionKind};

#[test]
fn envelope_header_rejects_a_future_version() {
    assert_error(
        b"fenestra-oracle-failure|2\n",
        ArtifactDecodeErrorKind::UnsupportedVersion(VersionKind::Envelope),
        Some(1),
    );
}

#[test]
fn first_future_declared_version_wins() {
    let cases = [
        (
            "versions|fixture|2|generator|2|case|2|state|2|trace|2|fingerprint|2|reducer|2",
            VersionKind::Fixture,
        ),
        (
            "versions|fixture|1|generator|2|case|2|state|2|trace|2|fingerprint|2|reducer|2",
            VersionKind::Generator,
        ),
        (
            "versions|fixture|1|generator|1|case|2|state|2|trace|2|fingerprint|2|reducer|2",
            VersionKind::Case,
        ),
        (
            "versions|fixture|1|generator|1|case|1|state|2|trace|2|fingerprint|2|reducer|2",
            VersionKind::State,
        ),
        (
            "versions|fixture|1|generator|1|case|1|state|1|trace|2|fingerprint|2|reducer|2",
            VersionKind::Trace,
        ),
        (
            "versions|fixture|1|generator|1|case|1|state|1|trace|1|fingerprint|2|reducer|2",
            VersionKind::Fingerprint,
        ),
        (
            "versions|fixture|1|generator|1|case|1|state|1|trace|1|fingerprint|1|reducer|2",
            VersionKind::Reducer,
        ),
    ];

    for (versions, kind) in cases {
        assert_error(
            &records(&[HEADER, versions]),
            ArtifactDecodeErrorKind::UnsupportedVersion(kind),
            Some(2),
        );
    }
}

#[test]
fn eof_reports_the_first_missing_section_without_a_line() {
    assert_error(
        &records(&[HEADER, VERSIONS]),
        ArtifactDecodeErrorKind::MissingSection(SectionKind::Fixture),
        None,
    );
}

#[test]
fn a_known_future_marker_reports_the_missing_expected_section() {
    assert_error(
        &records(&[HEADER, VERSIONS, "replay|4|64|256|128|1024|1"]),
        ArtifactDecodeErrorKind::MissingSection(SectionKind::Fixture),
        Some(3),
    );
}

#[test]
fn a_consumed_singleton_reports_its_duplicate() {
    assert_error(
        &records(&[HEADER, VERSIONS, FIXTURE, FIXTURE]),
        ArtifactDecodeErrorKind::DuplicateSection(SectionKind::Fixture),
        Some(4),
    );
}

#[test]
fn a_case_record_outside_its_section_is_an_ordering_violation() {
    assert_error(
        &records(&[HEADER, VERSIONS, FIXTURE, "tx|0|1"]),
        ArtifactDecodeErrorKind::OrderingViolation,
        Some(4),
    );
}

#[test]
fn a_section_end_before_its_begin_reports_the_missing_section() {
    assert_error(
        &records(&[
            HEADER,
            VERSIONS,
            FIXTURE,
            "replay|4|64|256|128|1024|1",
            "generator|8|2|8",
            "seed|0",
            "original-end",
        ]),
        ArtifactDecodeErrorKind::MissingSection(SectionKind::Original),
        Some(7),
    );
}

#[test]
fn a_repeated_section_begin_reports_the_duplicate_section() {
    assert_error(
        &records(&[
            HEADER,
            VERSIONS,
            FIXTURE,
            "replay|4|64|256|128|1024|1",
            "generator|8|2|8",
            "seed|0",
            "original-begin|1|0|7",
            "original-begin|1|0|7",
        ]),
        ArtifactDecodeErrorKind::DuplicateSection(SectionKind::Original),
        Some(8),
    );
}

#[test]
fn unknown_tags_bad_arity_and_bad_enum_words_are_malformed() {
    let cases = [
        records(&[HEADER, VERSIONS, "unknown|value"]),
        records(&[HEADER, VERSIONS, "fixture|runtime-oracle|1|1|5001|1"]),
        records(&[HEADER, VERSIONS, "fixture|unknown|1|1|5001|1|1"]),
    ];

    for input in cases {
        assert_error(&input, ArtifactDecodeErrorKind::MalformedRecord, Some(3));
    }
}
