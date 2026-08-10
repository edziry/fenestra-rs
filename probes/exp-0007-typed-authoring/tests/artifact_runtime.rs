#[allow(dead_code, unused_imports)]
#[path = "support/runtime_equivalence/mod.rs"]
mod support;

use fenestra_ui_exp_0007_typed_authoring::{generated_layout_board_v1, macro_layout_board_v1};
use fenestra_ui_testkit::prototype::{HeadlessFixtureV1, HeadlessProjectionFaultV1};

use support::{
    LaneLog, REGISTERED_RUNTIME_ARTIFACT_LIMITS_V1, RuntimeArtifactEncodeErrorKindV1,
    RuntimeArtifactFaultV1, RuntimeArtifactLimitKindV1, RuntimeArtifactLimitsV1,
    RuntimeArtifactSliceV1, encode_runtime_artifact_model_v1, encode_runtime_artifact_v1,
    inject_runtime_artifact_fault_v1, registered_operations, run_lane, runtime_artifact_model_v1,
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
    for (tag, expected) in [
        ("receipt|begin|", 6),
        ("receipt|end", 6),
        ("mutation|", 5),
        ("manifest|", 2),
        ("node|", 33),
        ("property|", 165),
        ("child|", 18),
        ("fragment|", 6),
        ("member|", 15),
        ("computed-record|", 33),
        ("geometry-record|", 33),
        ("semantic-record|", 6),
        ("hit-record|", 21),
        ("scene-record|", 33),
    ] {
        assert_eq!(
            GOLDEN.lines().filter(|line| line.starts_with(tag)).count(),
            expected,
            "wrong count for {tag}"
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
            RuntimeArtifactLimitKindV1::Records,
            RuntimeArtifactLimitKindV1::LineBytes,
            RuntimeArtifactLimitKindV1::ArtifactBytes,
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
            RuntimeArtifactLimitsV1::new(artifact_bytes - 1, line_bytes - 1, records - 1),
            RuntimeArtifactLimitKindV1::Records,
        ),
        (
            RuntimeArtifactLimitsV1::new(artifact_bytes - 1, line_bytes - 1, records),
            RuntimeArtifactLimitKindV1::LineBytes,
        ),
        (
            RuntimeArtifactLimitsV1::new(artifact_bytes - 1, line_bytes, records),
            RuntimeArtifactLimitKindV1::ArtifactBytes,
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
fn every_runtime_field_fault_is_detected_and_atomic_to_one_slice() {
    let lane = build_lanes().remove(2);
    let baseline = runtime_artifact_model_v1(&lane).expect("the registered log should normalize");
    let baseline_artifact =
        encode_runtime_artifact_model_v1(&baseline, REGISTERED_RUNTIME_ARTIFACT_LIMITS_V1)
            .expect("the registered model should encode");
    let faults = [
        (
            RuntimeArtifactFaultV1::ReceiptGeneration,
            RuntimeArtifactSliceV1::Receipt,
        ),
        (
            RuntimeArtifactFaultV1::ReceiptInvalidation,
            RuntimeArtifactSliceV1::Receipt,
        ),
        (
            RuntimeArtifactFaultV1::MutationKind,
            RuntimeArtifactSliceV1::Receipt,
        ),
        (
            RuntimeArtifactFaultV1::MutationPath,
            RuntimeArtifactSliceV1::Receipt,
        ),
        (
            RuntimeArtifactFaultV1::MutationProperty,
            RuntimeArtifactSliceV1::Receipt,
        ),
        (
            RuntimeArtifactFaultV1::MutationOldValue,
            RuntimeArtifactSliceV1::Receipt,
        ),
        (
            RuntimeArtifactFaultV1::MutationNewValue,
            RuntimeArtifactSliceV1::Receipt,
        ),
        (
            RuntimeArtifactFaultV1::MutationKey,
            RuntimeArtifactSliceV1::Receipt,
        ),
        (
            RuntimeArtifactFaultV1::MutationRoot,
            RuntimeArtifactSliceV1::Receipt,
        ),
        (
            RuntimeArtifactFaultV1::MutationIndices,
            RuntimeArtifactSliceV1::Receipt,
        ),
        (
            RuntimeArtifactFaultV1::CreatedManifest,
            RuntimeArtifactSliceV1::Receipt,
        ),
        (
            RuntimeArtifactFaultV1::RetiredManifest,
            RuntimeArtifactSliceV1::Receipt,
        ),
        (
            RuntimeArtifactFaultV1::StateNodePath,
            RuntimeArtifactSliceV1::State,
        ),
        (
            RuntimeArtifactFaultV1::StateNodeParent,
            RuntimeArtifactSliceV1::State,
        ),
        (
            RuntimeArtifactFaultV1::StateNodeTemplate,
            RuntimeArtifactSliceV1::State,
        ),
        (
            RuntimeArtifactFaultV1::StateNodeComponent,
            RuntimeArtifactSliceV1::State,
        ),
        (
            RuntimeArtifactFaultV1::StateNodeOrder,
            RuntimeArtifactSliceV1::State,
        ),
        (
            RuntimeArtifactFaultV1::StatePropertyOrder,
            RuntimeArtifactSliceV1::State,
        ),
        (
            RuntimeArtifactFaultV1::StatePropertyId,
            RuntimeArtifactSliceV1::State,
        ),
        (
            RuntimeArtifactFaultV1::StatePropertyValue,
            RuntimeArtifactSliceV1::State,
        ),
        (
            RuntimeArtifactFaultV1::StateChildOrder,
            RuntimeArtifactSliceV1::State,
        ),
        (
            RuntimeArtifactFaultV1::StateChildKind,
            RuntimeArtifactSliceV1::State,
        ),
        (
            RuntimeArtifactFaultV1::StateChildTarget,
            RuntimeArtifactSliceV1::State,
        ),
        (
            RuntimeArtifactFaultV1::StateFragmentPath,
            RuntimeArtifactSliceV1::State,
        ),
        (
            RuntimeArtifactFaultV1::StateFragmentDescriptor,
            RuntimeArtifactSliceV1::State,
        ),
        (
            RuntimeArtifactFaultV1::StateMemberOrder,
            RuntimeArtifactSliceV1::State,
        ),
        (
            RuntimeArtifactFaultV1::StateMemberKey,
            RuntimeArtifactSliceV1::State,
        ),
        (
            RuntimeArtifactFaultV1::StateMemberPath,
            RuntimeArtifactSliceV1::State,
        ),
        (
            RuntimeArtifactFaultV1::Surface,
            RuntimeArtifactSliceV1::Projection,
        ),
        (
            RuntimeArtifactFaultV1::Projection(HeadlessProjectionFaultV1::ComputedStyle),
            RuntimeArtifactSliceV1::Projection,
        ),
        (
            RuntimeArtifactFaultV1::Projection(HeadlessProjectionFaultV1::GeometryOrder),
            RuntimeArtifactSliceV1::Projection,
        ),
        (
            RuntimeArtifactFaultV1::Projection(HeadlessProjectionFaultV1::SemanticMembership),
            RuntimeArtifactSliceV1::Projection,
        ),
        (
            RuntimeArtifactFaultV1::Projection(HeadlessProjectionFaultV1::HitOrder),
            RuntimeArtifactSliceV1::Projection,
        ),
        (
            RuntimeArtifactFaultV1::Projection(HeadlessProjectionFaultV1::SceneOutput),
            RuntimeArtifactSliceV1::Projection,
        ),
    ];
    for (fault, changed_slice) in faults {
        let faulted_lane = inject_runtime_artifact_fault_v1(&lane, fault)
            .expect("the registered runtime fault should apply");
        let faulted = runtime_artifact_model_v1(&faulted_lane).unwrap_or_else(|error| {
            panic!("the faulted {fault:?} log should normalize: {error:?}")
        });
        let observed =
            encode_runtime_artifact_model_v1(&faulted, REGISTERED_RUNTIME_ARTIFACT_LIMITS_V1)
                .expect("the faulted model should encode");
        assert_ne!(observed, baseline_artifact, "missed {fault:?}");
        for slice in RuntimeArtifactSliceV1::ALL {
            assert_eq!(
                baseline.same_slice(&faulted, slice),
                slice != changed_slice,
                "non-atomic {fault:?} in {slice:?}"
            );
            assert_eq!(
                same_lane_slice(&lane, &faulted_lane, slice),
                slice != changed_slice,
                "non-atomic typed log {fault:?} in {slice:?}"
            );
        }
    }
}

fn same_lane_slice(left: &LaneLog, right: &LaneLog, slice: RuntimeArtifactSliceV1) -> bool {
    match slice {
        RuntimeArtifactSliceV1::Receipt => left.receipts() == right.receipts(),
        RuntimeArtifactSliceV1::State => left.states() == right.states(),
        RuntimeArtifactSliceV1::Projection => left.projections() == right.projections(),
        RuntimeArtifactSliceV1::FinalKeys => left.final_keys() == right.final_keys(),
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
