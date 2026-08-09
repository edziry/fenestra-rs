use fenestra_ui_testkit::prototype::{
    CaseDecodeContextV1, FingerprintSummaryV1, GeneratedCaseV1, GeneratorConfigV1, OperationIdV1,
    ReducerConfigV1, ReducerErrorKind, ReductionCompletionV1, ReplayFailureV1,
    RuntimeOracleFixtureV1, SeedV1, TraceFaultV1, decode_case_v1, generate_case_v1,
    reduce_failure_case_v1, replay_case_with_fault_v1,
};

#[test]
fn known_failure_reduces_deterministically_to_a_fixed_point() {
    let (fixture, original, fault, failure) = directed_failure();
    let reduced = reduce_failure_case_v1(
        &fixture,
        &original,
        fault,
        &failure,
        ReducerConfigV1::new(4_096),
    )
    .expect("known failure should reduce");

    assert_eq!(reduced.used_evaluations(), 35);
    assert_eq!(reduced.completion(), ReductionCompletionV1::FixedPoint);
    assert_minimized_shape(reduced.minimized_case());

    let replay = replay_case_with_fault_v1(&fixture, reduced.minimized_case(), fault)
        .expect("minimized case should replay");
    assert_eq!(replay.failure(), Some(&failure));

    let second = reduce_failure_case_v1(
        &fixture,
        reduced.minimized_case(),
        fault,
        &failure,
        ReducerConfigV1::new(4_096),
    )
    .expect("fixed point should reduce idempotently");
    assert_eq!(second.minimized_case(), reduced.minimized_case());
    assert_eq!(second.used_evaluations(), 23);
    assert_eq!(second.completion(), ReductionCompletionV1::FixedPoint);
}

#[test]
fn final_budget_evaluation_is_accepted_before_exhaustion() {
    let (fixture, original, fault, failure) = directed_failure();

    let before_accept = reduce_failure_case_v1(
        &fixture,
        &original,
        fault,
        &failure,
        ReducerConfigV1::new(9),
    )
    .expect("bounded reduction should complete");
    assert_eq!(before_accept.minimized_case(), &original);
    assert_eq!(before_accept.used_evaluations(), 9);
    assert_eq!(
        before_accept.completion(),
        ReductionCompletionV1::BudgetExhausted
    );

    let accepted = reduce_failure_case_v1(
        &fixture,
        &original,
        fault,
        &failure,
        ReducerConfigV1::new(10),
    )
    .expect("final permitted candidate should be accepted");
    assert_eq!(accepted.used_evaluations(), 10);
    assert_eq!(
        accepted.completion(),
        ReductionCompletionV1::BudgetExhausted
    );
    assert_eq!(transaction_ids(accepted.minimized_case()), vec![0, 1, 2]);
    assert_eq!(accepted.minimized_case().operation_count(), 5);

    for (budget, completion) in [
        (35, ReductionCompletionV1::BudgetExhausted),
        (36, ReductionCompletionV1::FixedPoint),
    ] {
        let result = reduce_failure_case_v1(
            &fixture,
            &original,
            fault,
            &failure,
            ReducerConfigV1::new(budget),
        )
        .expect("boundary reduction should complete");
        assert_minimized_shape(result.minimized_case());
        assert_eq!(result.used_evaluations(), 35);
        assert_eq!(result.completion(), completion);
    }
}

#[test]
fn reducer_configuration_is_closed_and_privacy_safe() {
    let (fixture, original, fault, failure) = directed_failure();

    for maximum in [0, 4_097] {
        let result = reduce_failure_case_v1(
            &fixture,
            &original,
            fault,
            &failure,
            ReducerConfigV1::new(maximum),
        );
        let Err(error) = result else {
            panic!("invalid reducer configuration should fail");
        };
        assert_eq!(error.kind(), ReducerErrorKind::InvalidConfiguration);
        assert_eq!(
            format!("{error:?}"),
            "ReducerError { kind: InvalidConfiguration }"
        );
        assert_eq!(
            error.to_string(),
            "runtime oracle reduction failed: InvalidConfiguration"
        );
    }
}

#[test]
fn reducer_rejects_a_smaller_candidate_with_a_different_failure() {
    let (fixture, original, fault, _) = directed_failure();
    let context = || {
        CaseDecodeContextV1::new(
            original.fixture_revision(),
            original.config(),
            original.seed(),
        )
    };
    let target_case = decode_case_v1(
        concat!(
            "tx|2|2\n",
            "op|3|insert|root/r:1|9|2\n",
            "op|4|move|root/r:1|9|1\n",
        )
        .as_bytes(),
        context(),
    )
    .expect("target case should decode");
    let smaller_case = decode_case_v1(
        concat!(
            "tx|2|2\n",
            "op|3|insert|root/r:1|9|2\n",
            "op|4|move|root/r:1|9|0\n",
        )
        .as_bytes(),
        context(),
    )
    .expect("smaller candidate should decode");
    let target_trace = replay_case_with_fault_v1(&fixture, &target_case, fault)
        .expect("target case should produce a terminal trace");
    let target = target_trace
        .failure()
        .expect("target case should fail")
        .clone();
    let smaller_trace = replay_case_with_fault_v1(&fixture, &smaller_case, fault)
        .expect("smaller candidate should produce a terminal trace");
    let different = smaller_trace
        .failure()
        .expect("smaller candidate should fail");

    assert_eq!(different.transaction(), target.transaction());
    assert_eq!(different.operation(), target.operation());
    assert_eq!(
        target.fingerprint().expected(),
        &FingerprintSummaryV1::Keys(vec![7, 9, 8])
    );
    assert_eq!(
        target.fingerprint().observed(),
        &FingerprintSummaryV1::Keys(vec![7, 8, 9])
    );
    assert_eq!(
        different.fingerprint().expected(),
        &FingerprintSummaryV1::Keys(vec![9, 7, 8])
    );
    assert_eq!(
        different.fingerprint().observed(),
        target.fingerprint().observed()
    );
    assert_ne!(different, &target);

    let reduced = reduce_failure_case_v1(
        &fixture,
        &target_case,
        fault,
        &target,
        ReducerConfigV1::new(4_096),
    )
    .expect("target case should reduce");
    assert_eq!(reduced.minimized_case(), &target_case);
    assert_eq!(reduced.used_evaluations(), 24);
    assert_eq!(reduced.completion(), ReductionCompletionV1::FixedPoint);
}

fn directed_failure() -> (
    RuntimeOracleFixtureV1,
    GeneratedCaseV1,
    TraceFaultV1,
    ReplayFailureV1,
) {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let original = generate_case_v1(&fixture, SeedV1::new(0), GeneratorConfigV1::new(8, 2, 8))
        .expect("directed case should generate");
    let fault = TraceFaultV1::OmitMove {
        target: OperationIdV1::new(4),
    };
    let trace =
        replay_case_with_fault_v1(&fixture, &original, fault).expect("known fault should replay");
    let failure = trace.failure().expect("known fault should fail").clone();
    (fixture, original, fault, failure)
}

fn assert_minimized_shape(case: &GeneratedCaseV1) {
    let [transaction] = case.transactions() else {
        panic!("fixed point should retain one transaction");
    };
    assert_eq!(transaction.id().get(), 2);
    assert_eq!(
        transaction
            .operations()
            .iter()
            .map(|operation| operation.id().get())
            .collect::<Vec<_>>(),
        vec![3, 4]
    );
}

fn transaction_ids(case: &GeneratedCaseV1) -> Vec<u32> {
    case.transactions()
        .iter()
        .map(|transaction| transaction.id().get())
        .collect()
}
