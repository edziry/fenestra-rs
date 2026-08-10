#[allow(dead_code, unused_imports)]
#[path = "support/runtime_equivalence/mod.rs"]
mod support;

use fenestra_ui_exp_0007_typed_authoring::{generated_layout_board_v1, macro_layout_board_v1};
use fenestra_ui_testkit::prototype::{HeadlessFixtureV1, HeadlessProjectionFaultV1};

use support::{
    LaneLog, REGISTERED_RUNTIME_ARTIFACT_LIMITS_V1, RuntimeArtifactEncodeErrorKindV1,
    RuntimeArtifactFaultV1, RuntimeArtifactLimitKindV1, RuntimeArtifactLimitsV1,
    encode_runtime_artifact_v1, inject_runtime_artifact_fault_v1, registered_operations, run_lane,
    validate_programs,
};

const GOLDEN: &str = include_str!("artifacts/layout-board-runtime-v1.txt");

#[test]
fn fen_ui_and_manual_lanes_each_match_the_committed_runtime_artifact() {
    for lane in build_lanes() {
        let first = encode(&lane, REGISTERED_RUNTIME_ARTIFACT_LIMITS_V1);
        let second = encode(&lane, REGISTERED_RUNTIME_ARTIFACT_LIMITS_V1);
        assert_eq!(first, GOLDEN);
        assert_eq!(second, GOLDEN);
    }
}

#[test]
fn committed_runtime_artifact_is_closed_bounded_ascii_and_exactly_478_lines() {
    assert!(GOLDEN.is_ascii());
    assert!(!GOLDEN.contains('\r'));
    assert!(GOLDEN.ends_with('\n'));
    assert!(!GOLDEN.ends_with("\n\n"));
    assert_eq!(GOLDEN.lines().count(), 478);
    assert!(GOLDEN.len() <= 32_768);
    assert!(GOLDEN.lines().all(|line| line.len() <= 512));
    assert_eq!(
        GOLDEN
            .lines()
            .filter(|line| line.starts_with("generation|1|") && line.ends_with("|begin"))
            .count(),
        6
    );
    for generation in 0..=5 {
        assert!(GOLDEN.contains(&format!("generation|1|{generation}|begin\n")));
        assert!(GOLDEN.contains(&format!("generation|1|{generation}|end\n")));
    }
    for family in ["computed", "geometry", "semantics", "hits", "scene"] {
        assert_eq!(
            GOLDEN
                .lines()
                .filter(|line| line.starts_with(&format!("{family}|count=")))
                .count(),
            6
        );
    }
    for forbidden in ["NodeId", "FragmentId", "Debug", "/home/", "C:\\", "OUT_DIR"] {
        assert!(!GOLDEN.contains(forbidden));
    }
}

#[test]
fn runtime_artifact_limits_are_inclusive_and_one_under_is_typed() {
    assert_eq!(
        RuntimeArtifactLimitKindV1::ALL,
        [
            RuntimeArtifactLimitKindV1::ArtifactBytes,
            RuntimeArtifactLimitKindV1::LineBytes,
            RuntimeArtifactLimitKindV1::Records,
        ]
    );
    assert_eq!(
        REGISTERED_RUNTIME_ARTIFACT_LIMITS_V1.limit(RuntimeArtifactLimitKindV1::ArtifactBytes),
        32_768
    );
    assert_eq!(
        REGISTERED_RUNTIME_ARTIFACT_LIMITS_V1.limit(RuntimeArtifactLimitKindV1::LineBytes),
        512
    );
    assert_eq!(
        REGISTERED_RUNTIME_ARTIFACT_LIMITS_V1.limit(RuntimeArtifactLimitKindV1::Records),
        512
    );

    let lane = build_lanes().remove(0);
    let artifact_bytes = GOLDEN.len();
    let line_bytes = GOLDEN.lines().map(str::len).max().unwrap_or(0);
    let records = GOLDEN.lines().count();
    let exact = RuntimeArtifactLimitsV1::new(artifact_bytes, line_bytes, records);
    assert_eq!(encode(&lane, exact), GOLDEN);
    let cases = [
        (
            RuntimeArtifactLimitsV1::new(artifact_bytes - 1, line_bytes, records),
            RuntimeArtifactLimitKindV1::ArtifactBytes,
        ),
        (
            RuntimeArtifactLimitsV1::new(artifact_bytes, line_bytes - 1, records),
            RuntimeArtifactLimitKindV1::LineBytes,
        ),
        (
            RuntimeArtifactLimitsV1::new(artifact_bytes, line_bytes, records - 1),
            RuntimeArtifactLimitKindV1::Records,
        ),
    ];
    for (limits, expected) in cases {
        let error = encode_runtime_artifact_v1(&lane, limits).expect_err("one under should fail");
        assert_eq!(
            error.kind(),
            RuntimeArtifactEncodeErrorKindV1::LimitExceeded(expected)
        );
    }
}

#[test]
fn receipt_manifest_state_and_all_projection_family_faults_change_the_artifact() {
    let lane = build_lanes().remove(2);
    let baseline = encode(&lane, REGISTERED_RUNTIME_ARTIFACT_LIMITS_V1);
    let faults = [
        RuntimeArtifactFaultV1::Receipt,
        RuntimeArtifactFaultV1::Manifest,
        RuntimeArtifactFaultV1::StateOrder,
        RuntimeArtifactFaultV1::Projection(HeadlessProjectionFaultV1::ComputedStyle),
        RuntimeArtifactFaultV1::Projection(HeadlessProjectionFaultV1::GeometryOrder),
        RuntimeArtifactFaultV1::Projection(HeadlessProjectionFaultV1::SemanticMembership),
        RuntimeArtifactFaultV1::Projection(HeadlessProjectionFaultV1::HitOrder),
        RuntimeArtifactFaultV1::Projection(HeadlessProjectionFaultV1::SceneOutput),
    ];
    for fault in faults {
        let faulted = inject_runtime_artifact_fault_v1(&lane, fault)
            .expect("the registered runtime fault should apply");
        let observed = encode(&faulted, REGISTERED_RUNTIME_ARTIFACT_LIMITS_V1);
        assert_ne!(observed, baseline, "missed {fault:?}");
    }
}

fn build_lanes() -> Vec<LaneLog> {
    let fixture = HeadlessFixtureV1::build().expect("the manual fixture should validate");
    let operations = registered_operations(&fixture);
    let fen = validate_programs(generated_layout_board_v1());
    let ui = validate_programs(macro_layout_board_v1());
    let manual = fixture.style().clone();
    [fen, ui, manual]
        .into_iter()
        .map(|style| run_lane(&fixture, style, &operations))
        .collect()
}

fn encode(lane: &LaneLog, limits: RuntimeArtifactLimitsV1) -> String {
    encode_runtime_artifact_v1(lane, limits).expect("the registered runtime artifact should encode")
}
