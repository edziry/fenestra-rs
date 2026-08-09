use fenestra_ui_testkit::prototype::{
    ArtifactFixtureMetadataV1, ArtifactReductionV1, ArtifactReplayConfigV1, FailureArtifactV1,
    ReductionCompletionV1, decode_failure_artifact_v1, encode_failure_artifact_v1,
};

const CANONICAL: &[u8] = include_bytes!("fixtures/canonical_structural_failure_v1.txt");

#[test]
fn typed_failure_codec_is_available_to_unpublished_oracle_consumers() {
    let artifact: FailureArtifactV1 =
        decode_failure_artifact_v1(CANONICAL).expect("canonical artifact should decode");

    assert_fixture(artifact.fixture());
    assert_replay_config(artifact.replay_config());
    assert_eq!(artifact.generator_config().transaction_count(), 8);
    assert_eq!(artifact.seed().get(), 0);
    assert_eq!(artifact.original_case().transactions().len(), 8);
    assert_eq!(artifact.minimized_case().transactions().len(), 1);
    assert_eq!(artifact.original_failure().transaction().get(), 2);
    assert_eq!(artifact.minimized_failure().transaction().get(), 2);
    assert_eq!(artifact.events().len(), 1);
    assert_reduction(artifact.reduction());

    let encoded = encode_failure_artifact_v1(&artifact).expect("canonical artifact should encode");
    assert_eq!(encoded, CANONICAL);
}

#[test]
fn typed_failure_artifact_debug_is_bounded_and_payload_free() {
    let artifact = decode_failure_artifact_v1(CANONICAL).expect("canonical artifact should decode");

    assert_eq!(
        format!("{artifact:?}"),
        "FailureArtifactV1 { fixture_revision: 1, original_transaction_count: 8, original_operation_count: 10, minimized_transaction_count: 1, minimized_operation_count: 2, trace_event_count: 1 }"
    );
}

fn assert_fixture(fixture: ArtifactFixtureMetadataV1) {
    assert_eq!(fixture.fixture_revision(), 1);
    assert_eq!(fixture.schema_format(), 1);
    assert_eq!(fixture.schema_namespace(), 5_001);
    assert_eq!(fixture.schema_revision(), 1);
    assert_eq!(fixture.construction_format(), 1);
}

fn assert_replay_config(config: ArtifactReplayConfigV1) {
    let fields: [u32; 6] = [
        config.operations(),
        config.structural_changes(),
        config.live_nodes(),
        config.live_fragments(),
        config.live_property_slots(),
        config.retained_generations(),
    ];
    assert_eq!(fields, [4, 64, 256, 128, 1_024, 1]);
}

fn assert_reduction(reduction: ArtifactReductionV1) {
    assert_eq!(reduction.max_evaluations(), 4_096);
    assert_eq!(reduction.used_evaluations(), 4_096);
    assert_eq!(
        reduction.completion(),
        ReductionCompletionV1::BudgetExhausted
    );
}
