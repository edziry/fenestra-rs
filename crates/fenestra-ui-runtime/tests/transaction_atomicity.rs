mod support;

use std::panic::{AssertUnwindSafe, catch_unwind};

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{TransactionErrorKind, UiRuntime};

use support::{KEY, LIST, NESTED_KEY, NESTED_LIST, SECOND_KEY, WIDTH, capacity, construction};

#[test]
fn create_and_retire_are_rolled_back_after_a_late_error() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let before = runtime.committed();
    let root = before.root();
    let list = before.fragment(root, LIST).unwrap();
    let original_member = before.keyed_member(list, KEY).unwrap();
    let original_nested = before.fragment(original_member, NESTED_LIST).unwrap();
    let original_nested_member = before.keyed_member(original_nested, NESTED_KEY).unwrap();
    let counts = (
        before.node_count(),
        before.fragment_count(),
        before.property_slot_count(),
    );
    let mut transaction = runtime.begin_transaction();
    transaction.insert_keyed(list, 99, 2).unwrap();
    transaction.remove_keyed(list, KEY).unwrap();
    transaction.insert_keyed(list, SECOND_KEY, 0).unwrap();

    let error = runtime.commit(transaction).unwrap_err();

    assert_eq!(error.kind(), TransactionErrorKind::DuplicateKey);
    assert_eq!(error.operation_index(), Some(2));
    let after = runtime.committed();
    assert!(before.shares_state_with(&after));
    assert_eq!(after.keyed_member(list, 99), None);
    assert_eq!(after.keyed_member(list, KEY), Some(original_member));
    assert_eq!(
        after.fragment(original_member, NESTED_LIST),
        Some(original_nested)
    );
    assert_eq!(
        after.keyed_member(original_nested, NESTED_KEY),
        Some(original_nested_member)
    );
    assert_eq!(
        (
            after.node_count(),
            after.fragment_count(),
            after.property_slot_count(),
        ),
        counts
    );
}

#[test]
fn caller_panic_while_staging_discards_the_detached_plan() {
    let runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let before = runtime.committed();
    let root = before.root();

    let panic = catch_unwind(AssertUnwindSafe(|| {
        let mut transaction = runtime.begin_transaction();
        transaction
            .set_property(root, WIDTH, PropertyValue::ScalarI32(500))
            .unwrap();
        panic!("caller panic during staging");
    }));

    assert!(panic.is_err());
    let after = runtime.committed();
    assert!(before.shares_state_with(&after));
    assert_eq!(after.generation(), before.generation());
    assert_eq!(
        after.property(root, WIDTH),
        Some(&PropertyValue::ScalarI32(120))
    );
}
