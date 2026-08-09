#[path = "headless/artifact_verify_support.rs"]
mod support;

use fenestra_ui_testkit::prototype::{
    HeadlessArtifactVerificationErrorKindV1 as Kind, HeadlessArtifactVerificationErrorV1,
    verify_headless_artifact_v1,
};

#[test]
fn every_projection_family_reports_its_first_differing_record_index() {
    let canonical = support::canonical_bytes();
    assert_error(
        &computed_mismatch(&canonical),
        Kind::ComputedStyleMismatch,
        Some(0),
    );
    assert_error(
        &geometry_mismatch(&canonical),
        Kind::GeometryMismatch,
        Some(1),
    );
    assert_error(
        &support::add_semantic_record(&canonical),
        Kind::SemanticsMismatch,
        Some(0),
    );
    assert_error(&hit_mismatch(&canonical), Kind::HitMismatch, Some(1));
    assert_error(&scene_mismatch(&canonical), Kind::SceneMismatch, Some(2));
}

#[test]
fn multiple_fields_in_one_record_keep_the_same_first_family_and_index() {
    let canonical = support::canonical_bytes();
    let computed = computed_mismatch(&canonical);
    let computed = support::set_field(&computed, "computed|root|", 3, "79");
    assert_error(&computed, Kind::ComputedStyleMismatch, Some(0));

    let geometry = geometry_mismatch(&canonical);
    let geometry = support::set_field(&geometry, "geometry|root/s:0|", 8, "79");
    assert_error(&geometry, Kind::GeometryMismatch, Some(1));

    let scene = scene_mismatch(&canonical);
    let scene = support::set_field(&scene, "scene|root/s:0/m:1:10|", 6, "rgba8:515a64ff");
    assert_error(&scene, Kind::SceneMismatch, Some(2));
}

#[test]
fn scheduler_then_projection_family_priority_is_exact() {
    let canonical = support::canonical_bytes();
    let scheduler_and_computed = scheduler_mismatch(&computed_mismatch(&canonical));
    assert_error(
        &scheduler_and_computed,
        Kind::SchedulerTraceMismatch,
        Some(0),
    );

    let computed_and_geometry = geometry_mismatch(&computed_mismatch(&canonical));
    assert_error(&computed_and_geometry, Kind::ComputedStyleMismatch, Some(0));

    let geometry_and_semantics = support::add_semantic_record(&geometry_mismatch(&canonical));
    assert_error(&geometry_and_semantics, Kind::GeometryMismatch, Some(1));

    let semantics_and_hit = hit_mismatch(&support::add_semantic_record(&canonical));
    assert_error(&semantics_and_hit, Kind::SemanticsMismatch, Some(0));

    let hit_and_scene = scene_mismatch(&hit_mismatch(&canonical));
    assert_error(&hit_and_scene, Kind::HitMismatch, Some(1));
}

fn computed_mismatch(bytes: &[u8]) -> Vec<u8> {
    support::set_field(bytes, "computed|root|", 2, "83")
}

fn geometry_mismatch(bytes: &[u8]) -> Vec<u8> {
    support::set_field(bytes, "geometry|root/s:0|", 4, "79")
}

fn hit_mismatch(bytes: &[u8]) -> Vec<u8> {
    support::set_field(bytes, "hit|root/s:0/m:1:30|", 4, "39")
}

fn scene_mismatch(bytes: &[u8]) -> Vec<u8> {
    support::set_field(bytes, "scene|root/s:0/m:1:10|", 4, "39")
}

fn scheduler_mismatch(bytes: &[u8]) -> Vec<u8> {
    support::set_field(bytes, "s-event|1|0|", 10, "faulted")
}

fn assert_error(bytes: &[u8], kind: Kind, index: Option<usize>) {
    let artifact = support::fixed_point(bytes);
    let error: HeadlessArtifactVerificationErrorV1 =
        verify_headless_artifact_v1(&artifact).expect_err("mismatched artifact should not verify");
    assert_eq!(error.kind(), kind);
    assert_eq!(error.index(), index);
}
