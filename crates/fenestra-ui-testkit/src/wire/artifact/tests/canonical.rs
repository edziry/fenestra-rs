use fenestra_ui_ir::prototype::{InvalidationClass, InvalidationSet};

use super::super::{
    ArtifactFixtureMetadataV1, ArtifactReductionV1, FailureArtifactV1, decode_failure_artifact_v1,
    encode_failure_artifact_v1,
};
use super::support::canonical_structural_artifact;
use crate::case::{OperationIdV1, SemanticOperationV1, TransactionIdV1};
use crate::failure::ReplayFailureV1;
use crate::fingerprint::{
    FailureFingerprintKindV1, FingerprintFieldV1, FingerprintLocationV1, FingerprintSummaryV1,
};
use crate::fixture::RuntimeOracleFixtureV1;
use crate::reducer::ReductionCompletionV1;
use crate::semantic::{FragmentPathV1, NodePathV1};
use crate::trace::{TraceComparisonV1, TraceFaultV1, TraceOutcomeV1};
use crate::wire::case::encode_case_v1;

#[test]
fn canonical_failure_artifact_decodes_to_typed_fields() {
    let bytes = canonical_structural_artifact();
    assert_eq!(bytes.len(), 958);
    assert_eq!(bytes.iter().filter(|byte| **byte == b'\n').count(), 39);

    let artifact = decode_canonical();
    assert_fixture(artifact.fixture());
    assert_eq!(
        artifact.replay_config(),
        RuntimeOracleFixtureV1::build()
            .expect("fixture should validate")
            .replay_config()
    );
    assert_eq!(artifact.generator_config().transaction_count(), 8);
    assert_eq!(
        artifact.generator_config().max_operations_per_transaction(),
        2
    );
    assert_eq!(artifact.generator_config().max_live_memberships(), 8);
    assert_eq!(artifact.seed().get(), 0);

    assert_case_shape(artifact.original_case(), 8, 10, 319);
    assert_case_shape(artifact.minimized_case(), 1, 2, 55);
    assert_directed_ids(artifact.original_case());
    assert_minimized_ids(artifact.minimized_case());
    assert_target_move(artifact.original_case());
    assert_target_move(artifact.minimized_case());
    assert_eq!(
        artifact.fault(),
        TraceFaultV1::OmitMove {
            target: OperationIdV1::new(4),
        }
    );
    assert_failure(artifact.original_failure());
    assert_failure(artifact.minimized_failure());
    assert_reduction(artifact.reduction());
    assert_trace(&artifact);
}

#[test]
fn canonical_failure_artifact_encode_and_decode_are_a_fixed_roundtrip() {
    let artifact = decode_canonical();

    let encoded = encode_failure_artifact_v1(&artifact)
        .expect("typed canonical artifact should encode within its limits");
    assert_eq!(encoded, canonical_structural_artifact());

    let decoded = decode_failure_artifact_v1(&encoded)
        .expect("encoded canonical artifact should decode again");
    assert!(decoded == artifact);
}

fn decode_canonical() -> FailureArtifactV1 {
    decode_failure_artifact_v1(canonical_structural_artifact())
        .expect("canonical failure artifact should decode")
}

fn assert_fixture(metadata: ArtifactFixtureMetadataV1) {
    assert_eq!(metadata.fixture_revision(), 1);
    assert_eq!(metadata.schema_format(), 1);
    assert_eq!(metadata.schema_namespace(), 5_001);
    assert_eq!(metadata.schema_revision(), 1);
    assert_eq!(metadata.construction_format(), 1);
}

fn assert_case_shape(
    case: &crate::case::GeneratedCaseV1,
    transactions: usize,
    operations: usize,
    bytes: usize,
) {
    assert_eq!(case.fixture_revision(), 1);
    assert_eq!(case.config().transaction_count(), 8);
    assert_eq!(case.config().max_operations_per_transaction(), 2);
    assert_eq!(case.config().max_live_memberships(), 8);
    assert_eq!(case.seed().get(), 0);
    assert_eq!(case.transactions().len(), transactions);
    assert_eq!(case.operation_count(), operations);
    assert_eq!(
        encode_case_v1(case)
            .expect("decoded case should remain canonically encodable")
            .len(),
        bytes
    );
}

fn assert_directed_ids(case: &crate::case::GeneratedCaseV1) {
    assert_eq!(
        case.transactions()
            .iter()
            .map(|transaction| transaction.id().get())
            .collect::<Vec<_>>(),
        (0..=7).collect::<Vec<_>>()
    );
    assert_eq!(
        case.transactions()
            .iter()
            .flat_map(|transaction| transaction.operations())
            .map(|operation| operation.id().get())
            .collect::<Vec<_>>(),
        (0..=9).collect::<Vec<_>>()
    );
}

fn assert_minimized_ids(case: &crate::case::GeneratedCaseV1) {
    let [transaction] = case.transactions() else {
        panic!("minimized case should retain one transaction");
    };
    assert_eq!(transaction.id(), TransactionIdV1::new(2));
    assert_eq!(
        transaction
            .operations()
            .iter()
            .map(|operation| operation.id())
            .collect::<Vec<_>>(),
        vec![OperationIdV1::new(3), OperationIdV1::new(4)]
    );
}

fn assert_target_move(case: &crate::case::GeneratedCaseV1) {
    let operation = case
        .transactions()
        .iter()
        .flat_map(|transaction| transaction.operations())
        .find(|operation| operation.id() == OperationIdV1::new(4))
        .expect("fault target should exist");
    let SemanticOperationV1::MoveKeyed {
        fragment,
        key,
        final_index,
    } = operation.operation()
    else {
        panic!("fault target should remain a move");
    };
    assert_eq!(fragment, &FragmentPathV1::new(NodePathV1::root(), 1));
    assert_eq!(*key, 9);
    assert_eq!(*final_index, 0);
}

fn assert_failure(failure: &ReplayFailureV1) {
    assert_eq!(failure.transaction(), TransactionIdV1::new(2));
    assert_eq!(failure.operation(), None);
    let fingerprint = failure.fingerprint();
    assert_eq!(fingerprint.kind(), FailureFingerprintKindV1::StateMismatch);
    assert_eq!(
        fingerprint.location(),
        &FingerprintLocationV1::Fragment(FragmentPathV1::new(NodePathV1::root(), 1))
    );
    assert_eq!(fingerprint.field(), FingerprintFieldV1::KeyedOrder);
    assert_eq!(
        fingerprint.expected(),
        &FingerprintSummaryV1::Keys(vec![9, 7, 8])
    );
    assert_eq!(
        fingerprint.observed(),
        &FingerprintSummaryV1::Keys(vec![7, 8, 9])
    );
}

fn assert_reduction(reduction: ArtifactReductionV1) {
    assert_eq!(reduction.max_evaluations(), 4_096);
    assert_eq!(reduction.used_evaluations(), 4_096);
    assert_eq!(
        reduction.completion(),
        ReductionCompletionV1::BudgetExhausted
    );
}

fn assert_trace(artifact: &FailureArtifactV1) {
    let [event] = artifact.events() else {
        panic!("canonical artifact should retain one trace event");
    };
    assert_eq!(event.sequence(), 0);
    assert_eq!(event.transaction(), TransactionIdV1::new(2));
    assert_eq!(
        event.operations(),
        &[OperationIdV1::new(3), OperationIdV1::new(4)]
    );
    assert_eq!(event.before_generation(), 0);
    assert_eq!(event.after_generation(), 1);
    assert_eq!(event.outcome(), TraceOutcomeV1::Commit);
    assert_eq!(event.mutation_count(), 1);
    let invalidation = InvalidationSet::from_class(InvalidationClass::Structure)
        .union(InvalidationSet::from_class(InvalidationClass::Layout))
        .union(InvalidationSet::from_class(InvalidationClass::Paint));
    assert_eq!(event.invalidation(), invalidation);
    assert_eq!(event.comparison(), TraceComparisonV1::Mismatch);
}
