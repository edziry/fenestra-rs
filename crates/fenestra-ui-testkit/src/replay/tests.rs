use std::cell::Cell;

use fenestra_ui_ir::prototype::{InvalidationClass, InvalidationSet, StructuralRegionId};
use fenestra_ui_runtime::prototype::UiRuntime;

use crate::case::{GeneratedCaseV1, GeneratorConfigV1, SeedV1, TransactionIdV1, TransactionV1};
use crate::desired::DesiredStateV1;
use crate::error::{HarnessError, HarnessErrorKind, HarnessLimitKind};
use crate::fixture::RuntimeOracleFixtureV1;
use crate::generate::generate_case_v1;
use crate::observe::observe_snapshot_indexed_v1;
use crate::resolve::ResolvedBaseV1;
use crate::semantic::NodePathV1;

use super::commit::{
    CommitShapeV1, RejectionShapeV1, observe_after_verified_commit_v1, verify_commit_shape,
    verify_rejection_shape,
};
use super::replay_case_v1;

mod observation;

#[test]
fn harness_limits_precede_empty_transaction_semantics() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let generated = generate_case_v1(&fixture, SeedV1::new(0), GeneratorConfigV1::new(8, 2, 8))
        .expect("directed case should generate");
    let operation = generated.transactions()[0].operations()[0].clone();
    let overfull = vec![operation; fixture.harness_limits().operations_per_transaction() + 1];
    let limit_transaction = TransactionIdV1::new(11);
    let malformed = GeneratedCaseV1::new(
        generated.fixture_revision(),
        generated.config(),
        generated.seed(),
        vec![
            TransactionV1::new(TransactionIdV1::new(10), Vec::new()),
            TransactionV1::new(limit_transaction, overfull),
        ],
    );

    let error = replay_case_v1(&fixture, &malformed).expect_err("limit should win");

    assert_eq!(
        error.kind(),
        HarnessErrorKind::LimitExceeded(HarnessLimitKind::OperationsPerTransaction)
    );
    assert_eq!(error.transaction(), Some(limit_transaction));
    assert_eq!(error.operation(), None);
}

#[test]
fn identity_iteration_preserves_authored_member_order() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let mut runtime = UiRuntime::new(
        fixture.construction().clone(),
        fixture.replay_config().runtime_capacity(),
    )
    .expect("runtime should initialize");
    let base = runtime.committed();
    let primary = base
        .fragment(base.root(), StructuralRegionId::new(0))
        .expect("primary fragment should exist");
    let mut transaction = runtime.begin_transaction();
    transaction
        .insert_keyed(primary, 9, 2)
        .expect("insert should stage");
    transaction
        .move_keyed(primary, 9, 0)
        .expect("move should stage");
    let receipt = runtime.commit(transaction).expect("commit should succeed");
    let snapshot = runtime.committed();
    let observed =
        observe_snapshot_indexed_v1(fixture.construction(), &snapshot, fixture.harness_limits())
            .expect("snapshot should observe");
    let authored: Vec<_> = observed
        .identities()
        .nodes_in_authored_order()
        .map(|(path, _)| path.clone())
        .collect();
    let member_9 = NodePathV1::root().member(1, 9);
    let member_7 = NodePathV1::root().member(1, 7);
    let member_8 = NodePathV1::root().member(1, 8);
    let position = |path: &NodePathV1| {
        authored
            .iter()
            .position(|candidate| candidate == path)
            .expect("member path should be observed")
    };

    assert!(position(&member_9) < position(&member_7));
    assert!(position(&member_7) < position(&member_8));

    drop(receipt);
    drop(base);
}

#[test]
fn resolution_is_observable_before_candidate_staging() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let generated = generate_case_v1(&fixture, SeedV1::new(0), GeneratorConfigV1::new(8, 2, 8))
        .expect("directed case should generate");
    let operation = &generated.transactions()[0].operations()[0];
    let mut desired =
        DesiredStateV1::from_construction(fixture.construction(), fixture.harness_limits())
            .expect("desired state should initialize");
    let mut runtime = UiRuntime::new(
        fixture.construction().clone(),
        fixture.replay_config().runtime_capacity(),
    )
    .expect("runtime should initialize");
    let base = runtime.committed();
    let observed =
        observe_snapshot_indexed_v1(fixture.construction(), &base, fixture.harness_limits())
            .expect("base should observe");
    let resolved = ResolvedBaseV1::new(observed.identities(), &desired)
        .resolve(generated.transactions()[0].id(), operation, &desired)
        .expect("operation should resolve without staging");

    let untouched = runtime.begin_transaction();
    let no_op = runtime
        .commit(untouched)
        .expect("empty commit should succeed");
    assert!(no_op.is_empty());
    assert!(base.shares_state_with(&runtime.committed()));
    drop(no_op);

    desired
        .apply_operation(operation.operation(), fixture.harness_limits())
        .expect("semantic operation should validate");
    let mut candidate = runtime.begin_transaction();
    resolved
        .stage(&mut candidate)
        .expect("resolved operation should stage");
    let receipt = runtime
        .commit(candidate)
        .expect("resolved operation should commit");

    assert!(!receipt.is_empty());
    assert_eq!(runtime.committed().generation().get(), 1);
}

#[test]
fn commit_shape_rejects_incoherent_publication_observations() {
    let none = InvalidationSet::NONE;
    let paint = InvalidationSet::from_class(InvalidationClass::Paint);
    let no_op = CommitShapeV1::new(true, true, 4, 4, 4, 0, none);
    let publication = CommitShapeV1::new(false, false, 4, 5, 5, 1, paint);

    assert!(!verify_commit_shape(no_op).expect("coherent no-op should validate"));
    assert!(verify_commit_shape(publication).expect("coherent publication should validate"));

    let incoherent = [
        CommitShapeV1::new(true, false, 4, 4, 4, 0, none),
        CommitShapeV1::new(true, true, 4, 5, 5, 0, none),
        CommitShapeV1::new(true, true, 4, 4, 5, 0, none),
        CommitShapeV1::new(true, true, 4, 4, 4, 1, none),
        CommitShapeV1::new(true, true, 4, 4, 4, 0, paint),
        CommitShapeV1::new(false, true, 4, 5, 5, 1, paint),
        CommitShapeV1::new(false, false, 4, 4, 4, 1, paint),
        CommitShapeV1::new(false, false, 4, 5, 4, 1, paint),
        CommitShapeV1::new(false, false, 4, 5, 5, 0, paint),
        CommitShapeV1::new(false, false, 4, 5, 5, 1, none),
        CommitShapeV1::new(false, false, u64::MAX, 0, 0, 1, paint),
    ];

    for shape in incoherent {
        let error = verify_commit_shape(shape).expect_err("shape should be rejected");
        assert_eq!(error.kind(), HarnessErrorKind::TraceMismatch);
        assert_eq!(error.transaction(), None);
        assert_eq!(error.operation(), None);
    }
}

#[test]
fn invalid_commit_shape_precedes_lazy_post_commit_observation() {
    let calls = Cell::new(0);
    let invalid = CommitShapeV1::new(true, false, 4, 4, 4, 0, InvalidationSet::NONE);

    let error = observe_after_verified_commit_v1(invalid, || -> Result<(), HarnessError> {
        calls.set(calls.get() + 1);
        Err(HarnessError::new(HarnessErrorKind::StateMismatch))
    })
    .expect_err("invalid shape must win before observation");

    assert_eq!(error.kind(), HarnessErrorKind::TraceMismatch);
    assert_eq!(calls.get(), 0);
    assert_eq!(error.transaction(), None);
    assert_eq!(error.operation(), None);
}

#[test]
fn rejected_candidate_must_preserve_the_exact_committed_state() {
    verify_rejection_shape(RejectionShapeV1::new(true, 4, 4))
        .expect("an unchanged committed state is a coherent rejection");

    for shape in [
        RejectionShapeV1::new(false, 4, 4),
        RejectionShapeV1::new(true, 4, 5),
    ] {
        let error = verify_rejection_shape(shape)
            .expect_err("a rejected candidate must not publish any state");
        assert_eq!(error.kind(), HarnessErrorKind::TraceMismatch);
    }
}

#[test]
fn transaction_location_clears_an_older_operation_location() {
    let transaction = TransactionIdV1::new(8);
    let error = HarnessError::new(HarnessErrorKind::InvalidOperation)
        .at_operation(TransactionIdV1::new(7), crate::case::OperationIdV1::new(3))
        .at_transaction(transaction);

    assert_eq!(error.transaction(), Some(transaction));
    assert_eq!(error.operation(), None);
}
