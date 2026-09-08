use std::cell::Cell;
use std::panic::{AssertUnwindSafe, catch_unwind};

use fenestra_ui_ir::prototype::PropertyValue;

use super::{PROPERTY, changed_transaction, runtime};
use crate::runtime::{CapacityKind, CommitWithError, TransactionErrorKind};

#[test]
fn successful_preparation_observes_and_returns_the_exact_candidate() {
    let mut runtime = runtime(4);
    let before = runtime.committed();
    let transaction = changed_transaction(&runtime, 10);
    let (receipt, candidate) = runtime
        .commit_with(transaction, |candidate| {
            assert_eq!(candidate.generation().get(), 1);
            assert_eq!(
                candidate.property(before.root(), PROPERTY),
                Some(&PropertyValue::ScalarI32(10))
            );
            assert_eq!(before.generation().get(), 0);
            Ok::<_, ()>(candidate.clone())
        })
        .unwrap();

    assert_eq!(receipt.generation().get(), 1);
    assert_eq!(receipt.mutations().count(), 1);
    assert!(candidate.shares_state_with(&runtime.committed()));
    assert!(!before.shares_state_with(&candidate));
}

#[test]
fn rejected_preparation_preserves_state_and_transaction_base() {
    let mut runtime = runtime(4);
    let before = runtime.committed();
    let retry = changed_transaction(&runtime, 20);
    let transaction = changed_transaction(&runtime, 10);
    let result = runtime.commit_with(transaction, |candidate| {
        assert_eq!(candidate.generation().get(), 1);
        Err::<(), _>("sidecar rejected")
    });

    assert!(matches!(
        result,
        Err(CommitWithError::Preparation("sidecar rejected"))
    ));
    assert!(before.shares_state_with(&runtime.committed()));
    assert_eq!(runtime.commit(retry).unwrap().generation().get(), 1);
}

#[test]
fn preparation_unwind_preserves_the_published_state() {
    let mut runtime = runtime(4);
    let before = runtime.committed();
    let transaction = changed_transaction(&runtime, 10);

    let panic = catch_unwind(AssertUnwindSafe(|| {
        runtime.commit_with(transaction, |_| -> Result<(), ()> {
            panic!("preparation failed")
        })
    }));

    assert!(panic.is_err());
    assert!(before.shares_state_with(&runtime.committed()));
    let retry = changed_transaction(&runtime, 20);
    assert_eq!(runtime.commit(retry).unwrap().generation().get(), 1);
}

#[test]
fn no_op_preparation_runs_once_against_the_existing_generation() {
    let mut runtime = runtime(0);
    runtime.set_generation_for_test(u64::MAX);
    let before = runtime.committed();
    let calls = Cell::new(0);
    let transaction = changed_transaction(&runtime, 0);

    let (receipt, sidecar) = runtime
        .commit_with(transaction, |candidate| {
            calls.set(calls.get() + 1);
            assert!(candidate.shares_state_with(&before));
            Ok::<_, ()>(candidate.generation().get())
        })
        .unwrap();

    assert_eq!(calls.get(), 1);
    assert!(receipt.is_empty());
    assert_eq!(sidecar, u64::MAX);
    assert!(before.shares_state_with(&runtime.committed()));
    let transaction = runtime.begin_transaction();
    assert!(matches!(
        runtime.commit_with(transaction, |_| Err::<(), _>("no-op rejection")),
        Err(CommitWithError::Preparation("no-op rejection"))
    ));
    assert!(before.shares_state_with(&runtime.committed()));
}

#[test]
fn stale_and_poisoned_transactions_do_not_invoke_preparation() {
    let mut runtime = runtime(4);
    let stale = runtime.begin_transaction();
    let changed = changed_transaction(&runtime, 10);
    runtime.commit(changed).unwrap();
    let error = runtime
        .commit_with(stale, |_| -> Result<(), ()> {
            panic!("stale transaction reached preparation")
        })
        .unwrap_err();
    assert!(matches!(
        error,
        CommitWithError::Runtime(error) if error.kind() == TransactionErrorKind::StaleBase
    ));

    let mut poisoned = runtime.begin_transaction();
    for _ in 0..8 {
        poisoned
            .set_property(
                runtime.committed().root(),
                PROPERTY,
                PropertyValue::ScalarI32(20),
            )
            .unwrap();
    }
    poisoned
        .set_property(
            runtime.committed().root(),
            PROPERTY,
            PropertyValue::ScalarI32(20),
        )
        .unwrap_err();
    let error = runtime
        .commit_with(poisoned, |_| -> Result<(), ()> {
            panic!("poisoned transaction reached preparation")
        })
        .unwrap_err();
    assert!(matches!(
        error,
        CommitWithError::Runtime(error)
            if error.kind() == TransactionErrorKind::CapacityExceeded(CapacityKind::Operations)
    ));
}

#[test]
fn retention_and_generation_guards_precede_preparation() {
    for retained in [0, 4] {
        let mut runtime = runtime(retained);
        runtime.set_generation_for_test(u64::MAX);
        let before = runtime.committed();
        let transaction = changed_transaction(&runtime, 10);
        let error = runtime
            .commit_with(transaction, |_| -> Result<(), ()> {
                panic!("failed publication guard reached preparation")
            })
            .unwrap_err();
        let expected = if retained == 0 {
            TransactionErrorKind::CapacityExceeded(CapacityKind::RetainedGenerations)
        } else {
            TransactionErrorKind::GenerationExhausted
        };
        assert!(matches!(error, CommitWithError::Runtime(error) if error.kind() == expected));
        assert!(before.shares_state_with(&runtime.committed()));
    }
}

#[test]
fn returned_sidecars_participate_in_existing_snapshot_retention() {
    let mut runtime = runtime(1);
    let transaction = changed_transaction(&runtime, 10);
    let (receipt, sidecar) = runtime
        .commit_with(transaction, |candidate| Ok::<_, ()>(candidate.clone()))
        .unwrap();
    drop(receipt);
    let transaction = changed_transaction(&runtime, 20);
    drop(runtime.commit(transaction).unwrap());
    let transaction = changed_transaction(&runtime, 30);
    let error = runtime.commit(transaction).unwrap_err();
    assert_eq!(
        error.kind(),
        TransactionErrorKind::CapacityExceeded(CapacityKind::RetainedGenerations)
    );
    drop(sidecar);
    let transaction = changed_transaction(&runtime, 30);
    assert_eq!(runtime.commit(transaction).unwrap().generation().get(), 3);
}
