#[path = "headless/artifact_verify_support.rs"]
mod support;

use fenestra_ui_testkit::prototype::{
    HeadlessArtifactCapacityKindV1 as CapacityKind,
    HeadlessArtifactVerificationErrorKindV1 as Kind, HeadlessArtifactVerificationErrorV1,
    HeadlessFailureCauseV1, verify_headless_artifact_v1,
};

#[test]
fn registered_artifact_verifies_through_the_public_borrowed_seam() {
    let artifact = support::fixed_point(&support::canonical_bytes());
    let verifier: fn(&_) -> Result<(), HeadlessArtifactVerificationErrorV1> =
        verify_headless_artifact_v1;

    verifier(&artifact).expect("registered fixed run artifact should verify");
}

#[test]
fn verification_and_capacity_vocabularies_have_closed_priority() {
    assert_eq!(
        CapacityKind::ALL,
        [
            CapacityKind::Ir,
            CapacityKind::Style,
            CapacityKind::Runtime,
            CapacityKind::Projection,
            CapacityKind::Scheduler,
            CapacityKind::Renderer,
            CapacityKind::SchedulerTrace,
            CapacityKind::HeadlessTrace,
            CapacityKind::Artifact,
        ]
    );
    for (expected, kind) in [
        Kind::FixtureMismatch,
        Kind::CapacityMismatch(CapacityKind::Ir),
        Kind::ReplayFailed(HeadlessFailureCauseV1::Runtime),
        Kind::ResultMismatch,
        Kind::FinalGenerationMismatch,
        Kind::SurfaceMismatch,
        Kind::HeadlessTraceMismatch,
        Kind::SchedulerTraceMismatch,
        Kind::ComputedStyleMismatch,
        Kind::GeometryMismatch,
        Kind::SemanticsMismatch,
        Kind::HitMismatch,
        Kind::SceneMismatch,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(verification_rank(kind), expected);
    }
}

#[test]
fn every_fixture_metadata_field_is_verified_before_later_content() {
    let canonical = support::canonical_bytes();
    for (field, value) in [
        (2, "2"),
        (3, "2"),
        (4, "8002"),
        (5, "2"),
        (6, "2"),
        (7, "2"),
    ] {
        let changed = support::set_field(&canonical, "fixture|headless-spine|", field, value);
        assert_error(&changed, Kind::FixtureMismatch, None);
    }
}

#[test]
fn every_capacity_row_is_verified_with_its_closed_kind() {
    let canonical = support::canonical_bytes();
    for (index, kind) in CapacityKind::ALL.into_iter().enumerate() {
        let changed = mutate_capacity(&canonical, index);
        assert_error(&changed, Kind::CapacityMismatch(kind), None);
    }
}

#[test]
fn simultaneous_capacity_mismatches_follow_capacity_kind_order() {
    let canonical = support::canonical_bytes();
    for first in 0..CapacityKind::ALL.len() {
        let mut changed = canonical.clone();
        for index in first..CapacityKind::ALL.len() {
            changed = mutate_capacity(&changed, index);
        }
        assert_error(
            &changed,
            Kind::CapacityMismatch(CapacityKind::ALL[first]),
            None,
        );
    }
}

#[test]
fn fixture_and_capacity_mismatches_precede_later_semantic_content() {
    let canonical = support::canonical_bytes();
    let capacity = support::set_field(&canonical, "capacity-style|", 1, "1");
    let fixture_and_capacity = support::set_field(&capacity, "fixture|headless-spine|", 2, "2");
    assert_error(&fixture_and_capacity, Kind::FixtureMismatch, None);

    let capacity_and_result = support::replace_once(&capacity, "result|pass", "result|adapt");
    assert_error(
        &capacity_and_result,
        Kind::CapacityMismatch(CapacityKind::Style),
        None,
    );
}

#[test]
fn verification_error_is_copyable_bounded_and_payload_free() {
    let canonical = support::canonical_bytes();
    let changed = support::set_field(&canonical, "fixture|headless-spine|", 2, "2");
    let error = verification_error(&changed);
    assert_error_traits::<HeadlessArtifactVerificationErrorV1>();
    assert_error_type(&error);
    assert_eq!(error.kind(), Kind::FixtureMismatch);
    assert_eq!(error.index(), None);
    assert_eq!(
        format!("{error:?}"),
        "HeadlessArtifactVerificationErrorV1 { kind: FixtureMismatch, index: None }"
    );
    assert_eq!(
        error.to_string(),
        "headless artifact verification failed: FixtureMismatch"
    );
}

fn assert_error(bytes: &[u8], kind: Kind, index: Option<usize>) {
    let error = verification_error(bytes);
    assert_eq!(error.kind(), kind);
    assert_eq!(error.index(), index);
}

fn verification_error(bytes: &[u8]) -> HeadlessArtifactVerificationErrorV1 {
    let artifact = support::fixed_point(bytes);
    verify_headless_artifact_v1(&artifact).expect_err("mismatched artifact should not verify")
}

fn verification_rank(kind: Kind) -> usize {
    match kind {
        Kind::FixtureMismatch => 0,
        Kind::CapacityMismatch(_) => 1,
        Kind::ReplayFailed(_) => 2,
        Kind::ResultMismatch => 3,
        Kind::FinalGenerationMismatch => 4,
        Kind::SurfaceMismatch => 5,
        Kind::HeadlessTraceMismatch => 6,
        Kind::SchedulerTraceMismatch => 7,
        Kind::ComputedStyleMismatch => 8,
        Kind::GeometryMismatch => 9,
        Kind::SemanticsMismatch => 10,
        Kind::HitMismatch => 11,
        Kind::SceneMismatch => 12,
    }
}

fn mutate_capacity(bytes: &[u8], index: usize) -> Vec<u8> {
    let (prefix, field, value) = [
        ("capacity-ir|", 1, "0"),
        ("capacity-style|", 1, "1"),
        ("capacity-runtime|", 1, "7"),
        ("capacity-projection|", 1, "7"),
        ("capacity-scheduler|", 3, "7"),
        ("capacity-renderer|", 3, "7"),
        ("capacity-scheduler-trace|", 1, "255"),
        ("capacity-headless-trace|", 1, "127"),
        ("capacity-artifact|", 1, "65000"),
    ][index];
    support::set_field(bytes, prefix, field, value)
}

fn assert_error_type(_: &HeadlessArtifactVerificationErrorV1) {}

fn assert_error_traits<T: Copy + Eq + std::error::Error>() {}
