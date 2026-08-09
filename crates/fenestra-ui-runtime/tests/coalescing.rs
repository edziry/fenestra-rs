mod support;

use fenestra_ui_ir::prototype::{InvalidationClass, PropertyValue};
use fenestra_ui_runtime::prototype::{MutationRecordView, UiRuntime};

use support::{LIST, SECOND_KEY, VISIBLE, WIDTH, capacity, construction};

#[test]
fn repeated_writes_keep_their_first_log_position_and_final_value() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let committed = runtime.committed();
    let root = committed.root();
    let list = committed.fragment(root, LIST).unwrap();
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(root, WIDTH, PropertyValue::ScalarI32(130))
        .unwrap();
    transaction.move_keyed(list, SECOND_KEY, 0).unwrap();
    transaction
        .set_property(root, WIDTH, PropertyValue::ScalarI32(140))
        .unwrap();

    let receipt = runtime.commit(transaction).unwrap();
    let mutations = receipt.mutations().collect::<Vec<_>>();

    assert_eq!(mutations.len(), 2);
    let MutationRecordView::PropertyChanged(change) = mutations[0] else {
        panic!("coalesced property should retain the first position");
    };
    assert!(matches!(mutations[1], MutationRecordView::KeyMoved(_)));
    assert_eq!(change.old_value(), &PropertyValue::ScalarI32(120));
    assert_eq!(change.new_value(), &PropertyValue::ScalarI32(140));
    assert_eq!(
        runtime.committed().property(root, WIDTH),
        Some(&PropertyValue::ScalarI32(140))
    );
}

#[test]
fn a_write_round_trip_disappears_without_losing_structural_order() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let committed = runtime.committed();
    let root = committed.root();
    let list = committed.fragment(root, LIST).unwrap();
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(root, VISIBLE, PropertyValue::Bool(false))
        .unwrap();
    transaction.move_keyed(list, SECOND_KEY, 0).unwrap();
    transaction
        .set_property(root, VISIBLE, PropertyValue::Bool(true))
        .unwrap();

    let receipt = runtime.commit(transaction).unwrap();

    assert_eq!(receipt.mutations().len(), 1);
    assert!(matches!(
        receipt.mutations().next(),
        Some(MutationRecordView::KeyMoved(_))
    ));
    assert!(!receipt.invalidation().contains(InvalidationClass::Paint));
}

#[test]
fn a_pure_write_round_trip_is_a_true_noop() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let before = runtime.committed();
    let root = before.root();
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(root, WIDTH, PropertyValue::ScalarI32(130))
        .unwrap();
    transaction
        .set_property(root, WIDTH, PropertyValue::ScalarI32(120))
        .unwrap();

    let receipt = runtime.commit(transaction).unwrap();

    assert!(receipt.is_empty());
    assert!(receipt.invalidation().is_empty());
    assert!(before.shares_state_with(&runtime.committed()));
}

#[test]
fn a_noop_does_not_make_a_sibling_transaction_stale() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let root = runtime.committed().root();
    let mut noop = runtime.begin_transaction();
    let mut sibling = runtime.begin_transaction();
    noop.set_property(root, WIDTH, PropertyValue::ScalarI32(120))
        .unwrap();
    sibling
        .set_property(root, WIDTH, PropertyValue::ScalarI32(200))
        .unwrap();

    assert!(runtime.commit(noop).unwrap().is_empty());
    let receipt = runtime.commit(sibling).unwrap();

    assert_eq!(receipt.generation().get(), 1);
    assert_eq!(
        runtime.committed().property(root, WIDTH),
        Some(&PropertyValue::ScalarI32(200))
    );
}
