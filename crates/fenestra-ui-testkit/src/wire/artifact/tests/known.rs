use fenestra_ui_ir::prototype::{SUPPORTED_CONSTRUCTION_FORMAT, SUPPORTED_SCHEMA_FORMAT};

use super::super::{
    ArtifactFixtureMetadataV1, ArtifactReductionV1, ArtifactReplayConfigV1, FailureArtifactV1,
    decode_failure_artifact_v1, encode_failure_artifact_v1, verify_failure_artifact_v1,
};
use crate::case::{GeneratorConfigV1, OperationIdV1, REGISTERED_FIXTURE_REVISION_V1, SeedV1};
use crate::fixture::{RuntimeOracleFixtureV1, SCHEMA_NAMESPACE, SCHEMA_REVISION};
use crate::generate::generate_case_v1;
use crate::reducer::{ReducerConfigV1, ReductionCompletionV1, reduce_failure_case_v1};
use crate::replay::{replay_case_v1, replay_case_with_fault_v1};
use crate::trace::TraceFaultV1;

const KNOWN_ARTIFACT: &[u8] =
    include_bytes!("../../../../tests/artifacts/known-move-omission-v1.txt");

#[test]
fn registered_inputs_reproduce_identical_known_artifact_bytes() {
    let generated = generate_known_artifact();
    let first =
        encode_failure_artifact_v1(&generated).expect("registered known artifact should encode");
    let second = encode_failure_artifact_v1(&generate_known_artifact())
        .expect("equal registered inputs should encode again");
    let decoded =
        decode_failure_artifact_v1(KNOWN_ARTIFACT).expect("committed known artifact should decode");

    assert_eq!(first, KNOWN_ARTIFACT);
    assert_eq!(second, first);
    assert!(decoded == generated);
    verify_failure_artifact_v1(&decoded).expect("committed known artifact should verify");
}

fn generate_known_artifact() -> FailureArtifactV1 {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let generator_config = GeneratorConfigV1::new(16, 4, 12);
    let seed = SeedV1::new(1_592_614_637);
    let original_case = generate_case_v1(&fixture, seed, generator_config)
        .expect("registered known case should generate");
    let fault = TraceFaultV1::OmitMove {
        target: OperationIdV1::new(4),
    };
    let original_trace = replay_case_with_fault_v1(&fixture, &original_case, fault)
        .expect("registered fault should replay");
    let original_failure = original_trace
        .failure()
        .expect("registered fault should fail")
        .clone();
    let reduction = reduce_failure_case_v1(
        &fixture,
        &original_case,
        fault,
        &original_failure,
        ReducerConfigV1::new(4_096),
    )
    .expect("registered failure should reduce");
    let minimized_case = reduction.minimized_case().clone();
    let minimized_trace = replay_case_with_fault_v1(&fixture, &minimized_case, fault)
        .expect("registered minimized fault should replay");
    let minimized_failure = minimized_trace
        .failure()
        .expect("registered minimized fault should fail")
        .clone();
    replay_case_v1(&fixture, &minimized_case)
        .expect("registered minimized case should pass without the fault");
    let fixed = reduce_failure_case_v1(
        &fixture,
        &minimized_case,
        fault,
        &minimized_failure,
        ReducerConfigV1::new(4_096),
    )
    .expect("registered minimized failure should be a fixed point");
    assert_eq!(fixed.minimized_case(), &minimized_case);
    assert_eq!(fixed.used_evaluations(), 23);
    assert_eq!(fixed.completion(), ReductionCompletionV1::FixedPoint);

    let replay_config = ArtifactReplayConfigV1::new([4, 64, 256, 128, 1_024, 1]);
    assert_eq!(replay_config, fixture.replay_config());
    FailureArtifactV1::new(
        ArtifactFixtureMetadataV1::new(
            REGISTERED_FIXTURE_REVISION_V1,
            SUPPORTED_SCHEMA_FORMAT.get(),
            SCHEMA_NAMESPACE.get(),
            SCHEMA_REVISION.get(),
            SUPPORTED_CONSTRUCTION_FORMAT.get(),
        ),
        replay_config,
        generator_config,
        seed,
        original_case,
        fault,
        original_failure,
        ArtifactReductionV1::new(4_096, reduction.used_evaluations(), reduction.completion()),
        minimized_case,
        minimized_failure,
        minimized_trace.events().to_vec(),
    )
}
