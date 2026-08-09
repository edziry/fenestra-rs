mod support;

use fenestra_ui_ir::prototype::{PropertyValue, TemplateNodeId};
use fenestra_ui_runtime::prototype::{MutationRecordView, TransactionErrorKind, UiRuntime};

use support::{
    EMPTY_LIST, ITEM, ITEM_BODY, KEY, LIST, NESTED_BODY, NESTED_KEY, NESTED_LIST, PANEL, ROOT,
    SECOND_KEY, STATIC_CHILD, VALUE, VISIBLE, WIDTH, capacity, construction, layout,
};

#[test]
fn initialization_materializes_the_validated_construction() {
    let runtime = UiRuntime::new(construction(), capacity()).expect("runtime should initialize");
    let committed = runtime.committed();
    let root = committed.root();

    assert_eq!(committed.generation().get(), 0);
    assert_eq!(committed.node_count(), 6);
    assert_eq!(committed.fragment_count(), 4);
    assert_eq!(committed.property_slot_count(), 8);
    assert_eq!(committed.template(root), Some(ROOT));
    assert_eq!(committed.component(root), Some(PANEL));
    assert_eq!(
        committed.property(root, WIDTH),
        Some(&PropertyValue::ScalarI32(120))
    );
    assert_eq!(
        committed.property(root, VISIBLE),
        Some(&PropertyValue::Bool(true))
    );

    let static_child = committed.children(root).expect("root should be live")[0];
    assert_eq!(committed.template(static_child), Some(STATIC_CHILD));
    assert_eq!(
        committed.property(static_child, WIDTH),
        Some(&PropertyValue::ScalarI32(100))
    );

    let list = committed
        .fragment(root, LIST)
        .expect("root list should be instantiated");
    let empty = committed
        .fragment(root, EMPTY_LIST)
        .expect("empty list should still own a fragment");
    assert_eq!(
        committed.keyed_members(list).unwrap().collect::<Vec<_>>(),
        vec![
            (KEY, committed.children(root).unwrap()[1]),
            (SECOND_KEY, committed.children(root).unwrap()[2]),
        ]
    );
    assert_eq!(committed.keyed_members(empty).unwrap().count(), 0);

    let member = committed
        .keyed_member(list, KEY)
        .expect("key should resolve");
    assert_eq!(committed.template(member), Some(ITEM_BODY));
    assert_eq!(committed.component(member), Some(ITEM));
    assert_eq!(
        committed.property(member, VALUE),
        Some(&PropertyValue::ScalarI32(10))
    );
    let nested = committed
        .fragment(member, NESTED_LIST)
        .expect("nested region should have its own identity");
    let nested_member = committed
        .keyed_member(nested, NESTED_KEY)
        .expect("nested key should resolve");
    assert_eq!(committed.template(nested_member), Some(NESTED_BODY));
    assert_eq!(committed.children(member), Some(&[nested_member][..]));

    let second_member = committed
        .keyed_member(list, SECOND_KEY)
        .expect("second key should resolve");
    let second_nested = committed
        .fragment(second_member, NESTED_LIST)
        .expect("each repeated node should own a distinct nested fragment");
    assert_ne!(member, second_member);
    assert_eq!(committed.template(second_member), Some(ITEM_BODY));
    assert_ne!(nested, second_nested);
}

#[test]
fn direct_update_publishes_one_immutable_generation() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let before = runtime.committed();
    let root = before.root();
    let children = before.children(root).unwrap().to_vec();
    let list = before.fragment(root, LIST).unwrap();
    let empty = before.fragment(root, EMPTY_LIST).unwrap();
    let members = before.keyed_members(list).unwrap().collect::<Vec<_>>();
    let nested = members
        .iter()
        .map(|(_, member)| {
            let fragment = before.fragment(*member, NESTED_LIST).unwrap();
            let nested_member = before.keyed_member(fragment, NESTED_KEY).unwrap();
            (*member, fragment, nested_member)
        })
        .collect::<Vec<_>>();
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(root, WIDTH, PropertyValue::ScalarI32(240))
        .unwrap();

    assert_eq!(
        runtime.committed().property(root, WIDTH),
        Some(&PropertyValue::ScalarI32(120))
    );
    let receipt = runtime.commit(transaction).expect("commit should publish");
    let after = runtime.committed();

    assert_eq!(receipt.generation().get(), 1);
    let mut mutations = receipt.mutations();
    assert_eq!(mutations.len(), 1);
    let Some(MutationRecordView::PropertyChanged(mutation)) = mutations.next() else {
        panic!("direct update should emit one property change");
    };
    assert_eq!(mutation.node(), root);
    assert_eq!(mutation.property(), WIDTH);
    assert_eq!(mutation.old_value(), &PropertyValue::ScalarI32(120));
    assert_eq!(mutation.new_value(), &PropertyValue::ScalarI32(240));
    assert_eq!(receipt.invalidation(), layout());
    assert_eq!(after.generation().get(), 1);
    assert_eq!(after.children(root), Some(children.as_slice()));
    assert_eq!(after.fragment(root, LIST), Some(list));
    assert_eq!(after.fragment(root, EMPTY_LIST), Some(empty));
    assert_eq!(
        after.keyed_members(list).unwrap().collect::<Vec<_>>(),
        members
    );
    for (member, fragment, nested_member) in nested {
        assert_eq!(after.fragment(member, NESTED_LIST), Some(fragment));
        assert_eq!(
            after.keyed_member(fragment, NESTED_KEY),
            Some(nested_member)
        );
    }
    assert_eq!(
        before.property(root, WIDTH),
        Some(&PropertyValue::ScalarI32(120))
    );
    assert_eq!(
        after.property(root, WIDTH),
        Some(&PropertyValue::ScalarI32(240))
    );
    assert!(!before.shares_state_with(&after));
}

#[test]
fn empty_and_same_value_transactions_are_true_noops() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let before = runtime.committed();

    let transaction = runtime.begin_transaction();
    let empty = runtime.commit(transaction).unwrap();
    assert!(empty.is_empty());
    assert_eq!(empty.generation(), before.generation());
    assert!(before.shares_state_with(&runtime.committed()));

    let root = before.root();
    let mut same_value = runtime.begin_transaction();
    same_value
        .set_property(root, WIDTH, PropertyValue::ScalarI32(120))
        .unwrap();
    let receipt = runtime.commit(same_value).unwrap();
    assert!(receipt.is_empty());
    assert!(before.shares_state_with(&runtime.committed()));
}

#[test]
fn a_late_failure_rolls_back_the_complete_draft() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let foreign = UiRuntime::new(construction(), capacity()).unwrap();
    let before = runtime.committed();
    let root = before.root();
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(root, WIDTH, PropertyValue::ScalarI32(300))
        .unwrap();
    transaction
        .set_property(
            foreign.committed().root(),
            WIDTH,
            PropertyValue::ScalarI32(400),
        )
        .unwrap();

    let error = runtime
        .commit(transaction)
        .expect_err("foreign node must fail");

    assert_eq!(error.kind(), TransactionErrorKind::MissingNode);
    assert_eq!(error.operation_index(), Some(1));
    assert!(before.shares_state_with(&runtime.committed()));
    assert_eq!(runtime.committed().generation().get(), 0);
    assert_eq!(
        runtime.committed().property(root, WIDTH),
        Some(&PropertyValue::ScalarI32(120))
    );
}

#[test]
fn stale_and_foreign_transactions_fail_before_operations() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let mut first = runtime.begin_transaction();
    let mut stale = runtime.begin_transaction();
    let root = runtime.committed().root();
    first
        .set_property(root, WIDTH, PropertyValue::ScalarI32(200))
        .unwrap();
    stale
        .set_property(root, WIDTH, PropertyValue::ScalarI32(300))
        .unwrap();
    runtime.commit(first).unwrap();

    let error = runtime.commit(stale).expect_err("old base should be stale");
    assert_eq!(error.kind(), TransactionErrorKind::StaleBase);
    assert_eq!(error.operation_index(), None);

    let other = UiRuntime::new(construction(), capacity()).unwrap();
    let foreign = other.begin_transaction();
    let error = runtime
        .commit(foreign)
        .expect_err("another runtime base should be foreign");
    assert_eq!(error.kind(), TransactionErrorKind::StaleBase);
    assert_eq!(error.operation_index(), None);
}

#[test]
fn template_ids_remain_data_not_runtime_identity() {
    let runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let committed = runtime.committed();
    let root = committed.root();

    assert_eq!(committed.template(root), Some(TemplateNodeId::new(0)));
    assert_ne!(committed.children(root).unwrap()[0], root);
}
