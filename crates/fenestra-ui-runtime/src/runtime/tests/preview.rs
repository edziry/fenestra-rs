use fenestra_ui_ir::prototype::{PropertyId, PropertyValue};

use super::{
    PROPERTY, REGION, RuntimeCapacity, UiRuntime, changed_transaction, construction, runtime,
};
use crate::runtime::{
    CapacityKind, CommittedRuntimeSnapshot, TransactionError, TransactionErrorKind, UiTransaction,
};

#[test]
fn preview_is_readonly_and_does_not_publish_or_invalidate_pending_transactions() {
    let mut runtime = runtime(4);
    let before = runtime.committed();
    let pending = changed_transaction(&runtime, 20);
    let preview: fn(
        &UiRuntime,
        UiTransaction,
    ) -> Result<CommittedRuntimeSnapshot, TransactionError> = UiRuntime::preview;
    let candidate = preview(&runtime, changed_transaction(&runtime, 10)).unwrap();
    assert_eq!(candidate.generation().get(), 1);
    assert_eq!(
        candidate.property(before.root(), PROPERTY),
        Some(&PropertyValue::ScalarI32(10))
    );
    assert!(before.shares_state_with(&runtime.committed()));
    assert!(!candidate.shares_state_with(&before));
    runtime.commit(pending).unwrap();
    assert_eq!(runtime.committed().generation().get(), 1);
    assert_eq!(
        runtime.committed().property(before.root(), PROPERTY),
        Some(&PropertyValue::ScalarI32(20))
    );
    assert_eq!(
        candidate.property(before.root(), PROPERTY),
        Some(&PropertyValue::ScalarI32(10))
    );
}

#[test]
fn previewed_identities_are_not_reserved_and_successful_commit_rebuilds_its_own_snapshot() {
    let mut runtime = runtime(4);
    let before = runtime.committed();
    let fragment = before.fragment(before.root(), REGION).unwrap();
    let insert = |runtime: &UiRuntime| {
        let mut transaction = runtime.begin_transaction();
        transaction.insert_keyed(fragment, 10, 1).unwrap();
        transaction
    };
    let candidate = runtime.preview(insert(&runtime)).unwrap();
    let candidate_node = candidate.keyed_member(fragment, 10).unwrap();
    assert_eq!(before.keyed_member(fragment, 10), None);
    assert!(before.shares_state_with(&runtime.committed()));
    let again = runtime.preview(insert(&runtime)).unwrap();
    assert_eq!(again.keyed_member(fragment, 10), Some(candidate_node));
    assert!(!candidate.shares_state_with(&again));

    let mut rejected = insert(&runtime);
    rejected
        .set_property(
            before.root(),
            PropertyId::new(99),
            PropertyValue::ScalarI32(1),
        )
        .unwrap();
    let error = runtime.preview(rejected).unwrap_err();
    assert_eq!(error.kind(), TransactionErrorKind::UnknownProperty);
    assert_eq!(error.operation_index(), Some(1));
    runtime.commit(insert(&runtime)).unwrap();
    let committed = runtime.committed();
    assert_eq!(committed.keyed_member(fragment, 10), Some(candidate_node));
    assert_eq!(committed.generation(), candidate.generation());
    assert!(!committed.shares_state_with(&candidate));
}

#[test]
fn noop_preview_keeps_identity_even_when_publication_limits_are_exhausted() {
    let mut runtime = runtime(0);
    runtime.set_generation_for_test(u64::MAX);
    let before = runtime.committed();
    for transaction in [
        runtime.begin_transaction(),
        changed_transaction(&runtime, 0),
    ] {
        let candidate = runtime.preview(transaction).unwrap();
        assert!(candidate.shares_state_with(&before));
        assert_eq!(candidate.generation().get(), u64::MAX);
    }
}

#[test]
fn stale_poisoned_and_invalid_transactions_preserve_exact_errors() {
    let mut runtime = runtime(4);
    let stale = runtime.begin_transaction();
    let mut poisoned = runtime.begin_transaction();
    for _ in 0..8 {
        poisoned
            .set_property(
                runtime.committed().root(),
                PROPERTY,
                PropertyValue::ScalarI32(1),
            )
            .unwrap();
    }
    let poison = poisoned
        .set_property(
            runtime.committed().root(),
            PROPERTY,
            PropertyValue::ScalarI32(1),
        )
        .unwrap_err();
    runtime.commit(changed_transaction(&runtime, 10)).unwrap();
    let before = runtime.committed();
    assert_eq!(
        runtime.preview(stale).unwrap_err().kind(),
        TransactionErrorKind::StaleBase
    );
    assert_eq!(runtime.preview(poisoned).unwrap_err(), poison);
    let invalid = |runtime: &UiRuntime| {
        let mut transaction = runtime.begin_transaction();
        transaction
            .set_property(
                before.root(),
                PropertyId::new(99),
                PropertyValue::ScalarI32(1),
            )
            .unwrap();
        transaction
    };
    assert_eq!(
        runtime.preview(invalid(&runtime)).unwrap_err(),
        runtime.commit(invalid(&runtime)).unwrap_err()
    );
    assert!(before.shares_state_with(&runtime.committed()));
}

#[test]
fn preview_preserves_live_capacity_rejection_and_successful_retry() {
    let mut runtime =
        UiRuntime::new(construction(), RuntimeCapacity::new(8, 8, 2, 8, 8, 4)).unwrap();
    let before = runtime.committed();
    let fragment = before.fragment(before.root(), REGION).unwrap();
    let mut transaction = runtime.begin_transaction();
    transaction.insert_keyed(fragment, 10, 1).unwrap();
    let error = runtime.preview(transaction).unwrap_err();
    assert_eq!(
        error.kind(),
        TransactionErrorKind::CapacityExceeded(CapacityKind::LiveNodes)
    );
    assert_eq!(error.operation_index(), Some(0));
    assert!(before.shares_state_with(&runtime.committed()));
    runtime.commit(changed_transaction(&runtime, 10)).unwrap();
    assert_eq!(runtime.committed().generation().get(), 1);
}

#[test]
fn preview_checks_retained_capacity_before_generation_exhaustion() {
    for retained in [0, 4] {
        let mut runtime = runtime(retained);
        runtime.set_generation_for_test(u64::MAX);
        let before = runtime.committed();
        let expected = if retained == 0 {
            TransactionErrorKind::CapacityExceeded(CapacityKind::RetainedGenerations)
        } else {
            TransactionErrorKind::GenerationExhausted
        };
        assert_eq!(
            runtime
                .preview(changed_transaction(&runtime, 10))
                .unwrap_err()
                .kind(),
            expected
        );
        assert!(before.shares_state_with(&runtime.committed()));
    }
}

#[test]
fn preview_counts_only_live_retired_generations_and_does_not_reserve_retention() {
    let mut runtime = runtime(1);
    let before = runtime.committed();
    drop(runtime.commit(changed_transaction(&runtime, 10)).unwrap());
    assert_eq!(
        runtime
            .preview(changed_transaction(&runtime, 20))
            .unwrap_err()
            .kind(),
        TransactionErrorKind::CapacityExceeded(CapacityKind::RetainedGenerations)
    );
    drop(before);
    let candidate = runtime.preview(changed_transaction(&runtime, 20)).unwrap();
    drop(runtime.commit(changed_transaction(&runtime, 20)).unwrap());
    drop(runtime.commit(changed_transaction(&runtime, 30)).unwrap());
    assert_eq!(runtime.committed().generation().get(), 3);
    assert_eq!(candidate.generation().get(), 2);
    assert_eq!(
        candidate.property(candidate.root(), PROPERTY),
        Some(&PropertyValue::ScalarI32(20))
    );
}
