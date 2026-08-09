mod support;

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{
    CapacityKind, RuntimeCapacity, RuntimeInitializationErrorKind, TransactionErrorKind, UiRuntime,
};

use support::multiplicity::{OUTER_REGION, hidden_overflow_construction};
use support::{KEY, LIST, WIDTH, capacity, construction};

fn initialization_error(capacity: RuntimeCapacity) -> RuntimeInitializationErrorKind {
    UiRuntime::new(construction(), capacity)
        .err()
        .expect("initialization should fail")
        .kind()
}

#[test]
fn initialization_limits_are_inclusive_and_use_global_priority() {
    if let Err(error) = UiRuntime::new(
        construction(),
        capacity()
            .with_live_nodes(6)
            .with_live_fragments(4)
            .with_live_property_slots(8),
    ) {
        panic!("exact footprint should fit: {error}");
    }

    assert_eq!(
        initialization_error(
            capacity()
                .with_live_nodes(5)
                .with_live_fragments(3)
                .with_live_property_slots(7),
        ),
        RuntimeInitializationErrorKind::CapacityExceeded(CapacityKind::LiveNodes)
    );
    assert_eq!(
        initialization_error(capacity().with_live_fragments(3)),
        RuntimeInitializationErrorKind::CapacityExceeded(CapacityKind::LiveFragments)
    );
    assert_eq!(
        initialization_error(capacity().with_live_property_slots(7)),
        RuntimeInitializationErrorKind::CapacityExceeded(CapacityKind::LivePropertySlots)
    );
}

#[test]
fn insertion_preflights_every_live_and_structural_capacity() {
    let cases = [
        (
            capacity().with_structural_changes(2),
            CapacityKind::StructuralChanges,
        ),
        (capacity().with_live_nodes(7), CapacityKind::LiveNodes),
        (
            capacity().with_live_fragments(4),
            CapacityKind::LiveFragments,
        ),
        (
            capacity().with_live_property_slots(9),
            CapacityKind::LivePropertySlots,
        ),
    ];

    for (limits, expected) in cases {
        let mut runtime = UiRuntime::new(construction(), limits).unwrap();
        let before = runtime.committed();
        let list = before.fragment(before.root(), LIST).unwrap();
        let mut transaction = runtime.begin_transaction();
        transaction.insert_keyed(list, 99, 2).unwrap();

        let error = runtime.commit(transaction).unwrap_err();

        assert_eq!(
            error.kind(),
            TransactionErrorKind::CapacityExceeded(expected)
        );
        assert_eq!(error.operation_index(), Some(0));
        assert!(before.shares_state_with(&runtime.committed()));
    }
}

#[test]
fn structural_capacity_wins_a_multi_capacity_insertion() {
    let limits = capacity()
        .with_structural_changes(2)
        .with_live_nodes(7)
        .with_live_fragments(4)
        .with_live_property_slots(9);
    let mut runtime = UiRuntime::new(construction(), limits).unwrap();
    let committed = runtime.committed();
    let list = committed.fragment(committed.root(), LIST).unwrap();
    let mut transaction = runtime.begin_transaction();
    transaction.insert_keyed(list, 99, 2).unwrap();

    let error = runtime.commit(transaction).unwrap_err();

    assert_eq!(
        error.kind(),
        TransactionErrorKind::CapacityExceeded(CapacityKind::StructuralChanges)
    );
    assert_eq!(error.operation_index(), Some(0));
}

#[test]
fn structural_capacity_wins_when_hidden_factory_multiplicity_overflows() {
    let limits = RuntimeCapacity::new(1, usize::MAX, usize::MAX, usize::MAX, usize::MAX, 1);
    let mut runtime = UiRuntime::new(hidden_overflow_construction(), limits).unwrap();
    let before = runtime.committed();
    let outer = before.fragment(before.root(), OUTER_REGION).unwrap();
    let mut transaction = runtime.begin_transaction();
    transaction.insert_keyed(outer, 7, 0).unwrap();

    let error = runtime.commit(transaction).unwrap_err();

    assert_eq!(
        error.kind(),
        TransactionErrorKind::CapacityExceeded(CapacityKind::StructuralChanges)
    );
    assert_eq!(error.operation_index(), Some(0));
    assert!(before.shares_state_with(&runtime.committed()));
    assert_eq!(runtime.committed().generation().get(), 0);
}

#[test]
fn cumulative_structural_work_is_bounded_before_mutation() {
    let limits = capacity().with_structural_changes(5);
    let mut runtime = UiRuntime::new(construction(), limits).unwrap();
    let before = runtime.committed();
    let list = before.fragment(before.root(), LIST).unwrap();
    let mut transaction = runtime.begin_transaction();
    transaction.remove_keyed(list, KEY).unwrap();
    transaction.insert_keyed(list, KEY, 1).unwrap();

    let error = runtime.commit(transaction).unwrap_err();

    assert_eq!(
        error.kind(),
        TransactionErrorKind::CapacityExceeded(CapacityKind::StructuralChanges)
    );
    assert_eq!(error.operation_index(), Some(1));
    assert!(before.shares_state_with(&runtime.committed()));
}

#[test]
fn operation_overflow_poisons_the_whole_transaction() {
    let limits = capacity().with_operations(1);
    let mut runtime = UiRuntime::new(construction(), limits).unwrap();
    let root = runtime.committed().root();
    let mut poisoned = runtime.begin_transaction();
    poisoned
        .set_property(root, WIDTH, PropertyValue::ScalarI32(200))
        .unwrap();
    let first_error = poisoned
        .set_property(root, WIDTH, PropertyValue::ScalarI32(300))
        .unwrap_err();
    let repeated = poisoned
        .set_property(root, WIDTH, PropertyValue::ScalarI32(400))
        .unwrap_err();
    assert_eq!(first_error, repeated);
    assert_eq!(first_error.operation_index(), Some(1));

    let mut winner = runtime.begin_transaction();
    winner
        .set_property(root, WIDTH, PropertyValue::ScalarI32(250))
        .unwrap();
    drop(runtime.commit(winner).unwrap());
    let committed_winner = runtime.committed();

    let error = runtime.commit(poisoned).unwrap_err();
    assert_eq!(
        error.kind(),
        TransactionErrorKind::CapacityExceeded(CapacityKind::Operations)
    );
    assert_eq!(error.operation_index(), Some(1));
    assert!(committed_winner.shares_state_with(&runtime.committed()));
    assert_eq!(
        runtime.committed().generation(),
        committed_winner.generation()
    );
    assert_eq!(
        runtime.committed().property(root, WIDTH),
        Some(&PropertyValue::ScalarI32(250))
    );
}

#[test]
fn retained_generations_block_until_every_old_handle_is_released() {
    let limits = capacity().with_retained_generations(1);
    let mut runtime = UiRuntime::new(construction(), limits).unwrap();
    let generation_zero = runtime.committed();
    let root = generation_zero.root();
    let mut first = runtime.begin_transaction();
    first
        .set_property(root, WIDTH, PropertyValue::ScalarI32(200))
        .unwrap();
    let first_receipt = runtime.commit(first).unwrap();
    let committed_first = runtime.committed();

    let mut blocked = runtime.begin_transaction();
    blocked
        .set_property(root, WIDTH, PropertyValue::ScalarI32(300))
        .unwrap();
    let error = runtime.commit(blocked).unwrap_err();
    assert_eq!(
        error.kind(),
        TransactionErrorKind::CapacityExceeded(CapacityKind::RetainedGenerations)
    );
    assert_eq!(error.operation_index(), None);
    assert!(committed_first.shares_state_with(&runtime.committed()));
    assert_eq!(
        runtime.committed().generation(),
        committed_first.generation()
    );
    assert_eq!(
        runtime.committed().property(root, WIDTH),
        Some(&PropertyValue::ScalarI32(200))
    );

    drop(first_receipt);
    let mut still_blocked = runtime.begin_transaction();
    still_blocked
        .set_property(root, WIDTH, PropertyValue::ScalarI32(300))
        .unwrap();
    let error = runtime.commit(still_blocked).unwrap_err();
    assert_eq!(
        error.kind(),
        TransactionErrorKind::CapacityExceeded(CapacityKind::RetainedGenerations)
    );
    assert_eq!(error.operation_index(), None);
    assert!(committed_first.shares_state_with(&runtime.committed()));

    drop(generation_zero);
    let mut retry = runtime.begin_transaction();
    retry
        .set_property(root, WIDTH, PropertyValue::ScalarI32(300))
        .unwrap();
    drop(
        runtime
            .commit(retry)
            .expect("released generation should unblock"),
    );
}

#[test]
fn a_receipt_alone_retains_its_previous_generation() {
    let limits = capacity().with_retained_generations(1);
    let mut runtime = UiRuntime::new(construction(), limits).unwrap();
    let root = runtime.committed().root();
    let mut first = runtime.begin_transaction();
    first
        .set_property(root, WIDTH, PropertyValue::ScalarI32(200))
        .unwrap();
    let receipt = runtime.commit(first).unwrap();
    let committed_first = runtime.committed();

    let mut blocked = runtime.begin_transaction();
    blocked
        .set_property(root, WIDTH, PropertyValue::ScalarI32(300))
        .unwrap();
    let error = runtime.commit(blocked).unwrap_err();
    assert_eq!(
        error.kind(),
        TransactionErrorKind::CapacityExceeded(CapacityKind::RetainedGenerations)
    );
    assert_eq!(error.operation_index(), None);
    assert!(committed_first.shares_state_with(&runtime.committed()));
    assert_eq!(
        runtime.committed().generation(),
        committed_first.generation()
    );
    assert_eq!(
        runtime.committed().property(root, WIDTH),
        Some(&PropertyValue::ScalarI32(200))
    );

    drop(receipt);
    let mut retry = runtime.begin_transaction();
    retry
        .set_property(root, WIDTH, PropertyValue::ScalarI32(300))
        .unwrap();
    drop(runtime.commit(retry).unwrap());
}

#[test]
fn old_transaction_bases_participate_in_the_retained_bound() {
    let limits = capacity().with_retained_generations(1);
    let mut runtime = UiRuntime::new(construction(), limits).unwrap();
    let root = runtime.committed().root();
    let old_transaction = runtime.begin_transaction();
    let mut first = runtime.begin_transaction();
    first
        .set_property(root, WIDTH, PropertyValue::ScalarI32(200))
        .unwrap();
    drop(runtime.commit(first).unwrap());
    let committed_first = runtime.committed();

    let mut blocked = runtime.begin_transaction();
    blocked
        .set_property(root, WIDTH, PropertyValue::ScalarI32(300))
        .unwrap();
    let error = runtime.commit(blocked).unwrap_err();
    assert_eq!(
        error.kind(),
        TransactionErrorKind::CapacityExceeded(CapacityKind::RetainedGenerations)
    );
    assert_eq!(error.operation_index(), None);
    assert!(committed_first.shares_state_with(&runtime.committed()));
    assert_eq!(
        runtime.committed().generation(),
        committed_first.generation()
    );

    drop(old_transaction);
    let mut retry = runtime.begin_transaction();
    retry
        .set_property(root, WIDTH, PropertyValue::ScalarI32(300))
        .unwrap();
    drop(runtime.commit(retry).unwrap());
}

#[test]
fn noops_do_not_consume_retained_generation_capacity() {
    let limits = capacity().with_retained_generations(0);
    let mut runtime = UiRuntime::new(construction(), limits).unwrap();
    let before = runtime.committed();
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(before.root(), WIDTH, PropertyValue::ScalarI32(120))
        .unwrap();

    let receipt = runtime.commit(transaction).unwrap();

    assert!(receipt.is_empty());
    assert!(before.shares_state_with(&runtime.committed()));
}
