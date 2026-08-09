#[path = "headless/artifact_expected.rs"]
mod expected;

use fenestra_ui_runtime::prototype::HeadlessSurface;
use fenestra_ui_testkit::prototype::{
    HeadlessArtifactEncodeErrorV1, HeadlessArtifactV1, HeadlessResultV1,
    HeadlessTraceProjectionCountsV1, build_headless_artifact_v1, encode_headless_artifact_v1,
    run_headless_spine_v1,
};

#[test]
fn fixed_run_encodes_the_complete_canonical_headless_artifact() {
    let run = run_headless_spine_v1().expect("the registered headless run should pass");
    let artifact: HeadlessArtifactV1 = build_headless_artifact_v1(&run);

    assert_eq!(run.result(), HeadlessResultV1::Pass);
    assert_eq!(artifact.result(), HeadlessResultV1::Pass);
    assert_eq!(artifact.final_generation(), 9);
    assert_eq!(artifact.final_surface(), HeadlessSurface::new(90, 70));
    let counts: HeadlessTraceProjectionCountsV1 = artifact.final_projection_counts();
    assert_eq!(
        [
            counts.computed_styles(),
            counts.geometries(),
            counts.semantics(),
            counts.hit_regions(),
            counts.scene_rectangles(),
        ],
        [5, 5, 0, 2, 4]
    );
    assert_eq!(artifact.headless_event_count(), 55);
    assert_eq!(artifact.scheduler_event_count(), 41);
    assert_clone_eq::<HeadlessArtifactV1>();
    assert_eq!(artifact.clone(), artifact);

    drop(run);
    let encoded = encode(&artifact);
    expected::assert_canonical_artifact(&encoded);
}

#[test]
fn two_fresh_runs_encode_to_identical_owned_bytes() {
    let first_run = run_headless_spine_v1().expect("the first headless run should pass");
    let first = build_headless_artifact_v1(&first_run);
    drop(first_run);

    let second_run = run_headless_spine_v1().expect("the second headless run should pass");
    let second = build_headless_artifact_v1(&second_run);
    drop(second_run);

    assert_eq!(encode(&first), encode(&second));
}

#[test]
fn artifact_debug_and_wire_text_disclose_only_closed_synthetic_data() {
    let run = run_headless_spine_v1().expect("the registered headless run should pass");
    let artifact = build_headless_artifact_v1(&run);
    let encoded = encode(&artifact);

    assert_eq!(
        format!("{artifact:?}"),
        "HeadlessArtifactV1 { headless_event_count: 55, scheduler_event_count: 41, computed_style_count: 5, geometry_count: 5, semantic_count: 0, hit_region_count: 2, scene_rectangle_count: 4 }"
    );
    assert!(encoded.is_ascii());
    assert!(
        encoded
            .iter()
            .all(|byte| *byte == b'\n' || (0x20..=0x7e).contains(byte))
    );
    let text = std::str::from_utf8(&encoded).expect("ASCII artifact should be UTF-8");
    for forbidden in [
        "NodeId",
        "FragmentId",
        "RuntimeState",
        "snapshot",
        "source-text",
        "/home/",
        "0x",
    ] {
        assert!(!text.contains(forbidden), "artifact disclosed {forbidden}");
    }
}

fn encode(artifact: &HeadlessArtifactV1) -> Vec<u8> {
    let encoder: fn(&HeadlessArtifactV1) -> Result<Vec<u8>, HeadlessArtifactEncodeErrorV1> =
        encode_headless_artifact_v1;
    encoder(artifact).expect("the registered artifact should fit every output bound")
}

fn assert_clone_eq<T: Clone + Eq>() {}
