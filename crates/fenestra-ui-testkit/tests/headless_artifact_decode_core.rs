#[path = "headless/artifact_decode_support.rs"]
mod support;

use fenestra_ui_testkit::prototype::{
    HeadlessArtifactDecodeErrorV1, HeadlessArtifactV1, HeadlessResultV1,
    encode_headless_artifact_v1,
};

#[test]
fn canonical_decode_owns_the_same_typed_artifact_as_the_fixed_run() {
    let bytes = support::canonical_bytes();
    let expected = support::canonical_artifact();
    let decoded: HeadlessArtifactV1 = support::decode(&bytes);
    drop(bytes);

    assert_eq!(decoded, expected);
    assert_eq!(decoded.result(), HeadlessResultV1::Pass);
    assert_eq!(decoded.final_generation(), 9);
    assert_eq!(decoded.headless_event_count(), 55);
    assert_eq!(decoded.scheduler_event_count(), 41);
    assert_eq!(
        encode_headless_artifact_v1(&decoded).expect("decoded artifact should re-encode"),
        support::canonical_bytes()
    );
    assert_error_traits::<HeadlessArtifactDecodeErrorV1>();
}

#[test]
fn decoder_preserves_valid_nonregistered_values_instead_of_returning_a_fixed_model() {
    let canonical = support::canonical_bytes();
    let adapted = support::replace_once(&canonical, "result|pass", "result|adapt");
    let changed = support::replace_once(
        &adapted,
        "computed|root|84|80|rgba8:090909ff|true|ignore",
        "computed|root|83|80|rgba8:090909ff|true|ignore",
    );
    let decoded = support::decode(&changed);

    assert_eq!(decoded.result(), HeadlessResultV1::Adapt);
    assert_ne!(decoded, support::canonical_artifact());
    assert_eq!(
        encode_headless_artifact_v1(&decoded).expect("adapted artifact should re-encode"),
        changed
    );
}

#[test]
fn nonregistered_fixture_metadata_roundtrips_for_later_semantic_verification() {
    let canonical = support::canonical_bytes();
    let mut changed = support::set_field(&canonical, "fixture|headless-spine|", 2, "2");
    changed = support::set_field(&changed, "fixture|headless-spine|", 3, "2");
    changed = support::set_field(&changed, "fixture|headless-spine|", 4, "8002");
    changed = support::set_field(&changed, "fixture|headless-spine|", 5, "2");
    changed = support::set_field(&changed, "fixture|headless-spine|", 6, "2");
    changed = support::set_field(&changed, "fixture|headless-spine|", 7, "2");
    let decoded = support::decode(&changed);

    assert_ne!(decoded, support::canonical_artifact());
    assert_eq!(
        encode_headless_artifact_v1(&decoded).expect("metadata variant should re-encode"),
        changed
    );
}

fn assert_error_traits<T: Copy + Eq + std::error::Error>() {}
