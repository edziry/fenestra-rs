use fenestra_ui_testkit::prototype::{
    FailureFingerprintKindV1, FingerprintFieldV1, FingerprintLocationV1, FingerprintSummaryV1,
    FragmentPathV1, NodePathV1, OperationIdV1, ReductionCompletionV1, TraceFaultV1,
    TransactionIdV1, decode_failure_artifact_v1, encode_failure_artifact_v1,
    verify_failure_artifact_v1,
};

const KNOWN_ARTIFACT: &[u8] = include_bytes!("artifacts/known-move-omission-v1.txt");

#[test]
fn committed_move_omission_artifact_satisfies_the_v1_contract() {
    assert!(KNOWN_ARTIFACT.is_ascii());
    assert_eq!(KNOWN_ARTIFACT.len(), 1_741);
    assert_eq!(
        KNOWN_ARTIFACT.iter().filter(|byte| **byte == b'\n').count(),
        69
    );
    assert_eq!(KNOWN_ARTIFACT.last(), Some(&b'\n'));

    let artifact =
        decode_failure_artifact_v1(KNOWN_ARTIFACT).expect("known artifact should decode");
    assert_eq!(
        encode_failure_artifact_v1(&artifact).expect("known artifact should encode"),
        KNOWN_ARTIFACT
    );
    verify_failure_artifact_v1(&artifact).expect("known artifact should verify");

    let generator = artifact.generator_config();
    assert_eq!(generator.transaction_count(), 16);
    assert_eq!(generator.max_operations_per_transaction(), 4);
    assert_eq!(generator.max_live_memberships(), 12);
    assert_eq!(artifact.seed().get(), 1_592_614_637);
    assert_eq!(artifact.original_case().transactions().len(), 16);
    assert_eq!(artifact.original_case().operation_count(), 32);
    assert_eq!(
        artifact.fault(),
        TraceFaultV1::OmitMove {
            target: OperationIdV1::new(4),
        }
    );

    let reduction = artifact.reduction();
    assert_eq!(reduction.max_evaluations(), 4_096);
    assert_eq!(reduction.used_evaluations(), 35);
    assert_eq!(reduction.completion(), ReductionCompletionV1::FixedPoint);
    assert!(
        artifact.minimized_case().operation_count() < artifact.original_case().operation_count()
    );

    let [transaction] = artifact.minimized_case().transactions() else {
        panic!("known reduction should retain one transaction");
    };
    assert_eq!(transaction.id(), TransactionIdV1::new(2));
    assert_eq!(
        transaction
            .operations()
            .iter()
            .map(|operation| operation.id().get())
            .collect::<Vec<_>>(),
        vec![3, 4]
    );

    assert_eq!(artifact.original_failure(), artifact.minimized_failure());
    let failure = artifact.original_failure();
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
    assert_eq!(artifact.events().len(), 1);
}
