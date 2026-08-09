use fenestra_ui_testkit::prototype::{
    FailureFingerprintKindV1, FailureFingerprintV1, FingerprintFieldV1, FingerprintLocationV1,
    FingerprintSummaryV1, FragmentPathV1, GeneratorConfigV1, HarnessErrorKind, NodePathV1,
    OperationIdV1, ReplayFailureV1, RuntimeOracleFixtureV1, SeedV1, TraceFaultV1,
    TraceTerminationV1, TransactionIdV1, generate_case_v1, replay_case_with_fault_v1,
    replay_case_with_trace_v1,
};

#[test]
fn omit_move_fault_reports_first_semantic_fragment_mismatch() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let case = generate_case_v1(&fixture, SeedV1::new(0), GeneratorConfigV1::new(8, 2, 8))
        .expect("directed case should generate");

    let normal = replay_case_with_trace_v1(&fixture, &case)
        .expect("the unfaulted directed case should replay");
    assert_eq!(normal.termination(), TraceTerminationV1::Success);
    assert!(normal.failure().is_none());

    let fault = TraceFaultV1::OmitMove {
        target: OperationIdV1::new(4),
    };
    let faulted = replay_case_with_fault_v1(&fixture, &case, fault)
        .expect("the known fault should produce a terminal trace");

    assert_eq!(faulted.termination(), TraceTerminationV1::Mismatch);
    assert_eq!(faulted.events().len(), 3);
    assert!(
        faulted
            .canonical_bytes()
            .ends_with(b"event|2|2|3,4|1|2|commit|1|structure,layout,paint|mismatch\n")
    );

    let provenance = faulted.provenance();
    assert_eq!(provenance.fixture_revision(), case.fixture_revision());
    assert_eq!(provenance.replay_config(), fixture.replay_config());
    assert_eq!(provenance.generator_config(), case.config());
    assert_eq!(provenance.seed(), case.seed());
    assert_eq!(provenance.fault(), Some(fault));

    let report = faulted.report();
    assert_eq!(report.transaction_count(), 8);
    assert_eq!(report.operation_count(), 10);
    assert_eq!(report.verified_step_count(), 2);
    assert_eq!(report.publication_count(), 2);
    assert_eq!(report.noop_count(), 1);
    assert_eq!(report.final_generation(), 2);
    assert_eq!(report.identity().preserved(), 26);
    assert_eq!(report.identity().retired(), 0);
    assert_eq!(report.identity().fresh(), 0);
    assert_eq!(report.identity().alias_free_snapshots(), 3);

    let failure: &ReplayFailureV1 = faulted
        .failure()
        .expect("the faulted trace should retain its first failure");
    assert_eq!(failure.transaction(), TransactionIdV1::new(2));
    assert_eq!(failure.operation(), None);

    let fingerprint: &FailureFingerprintV1 = failure.fingerprint();
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

#[test]
fn omit_move_fault_rejects_missing_or_non_move_targets() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let case = generate_case_v1(&fixture, SeedV1::new(0), GeneratorConfigV1::new(8, 2, 8))
        .expect("directed case should generate");

    let non_move = replay_case_with_fault_v1(
        &fixture,
        &case,
        TraceFaultV1::OmitMove {
            target: OperationIdV1::new(0),
        },
    )
    .expect_err("a set operation is not a valid omit-move target");
    assert_eq!(non_move.kind(), HarnessErrorKind::InvalidOperation);
    assert_eq!(non_move.transaction(), Some(TransactionIdV1::new(0)));
    assert_eq!(non_move.operation(), Some(OperationIdV1::new(0)));

    let missing = replay_case_with_fault_v1(
        &fixture,
        &case,
        TraceFaultV1::OmitMove {
            target: OperationIdV1::new(u32::MAX),
        },
    )
    .expect_err("an absent operation is not a valid omit-move target");
    assert_eq!(missing.kind(), HarnessErrorKind::InvalidOperation);
    assert_eq!(missing.transaction(), None);
    assert_eq!(missing.operation(), None);
}
