mod support;

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{
    ManifestEntry, MutationRecordView, TransactionErrorKind, UiRuntime,
};

use support::{
    EMPTY_LIST, ITEM_BODY, KEY, LIST, NESTED_KEY, NESTED_LIST, SECOND_KEY, VALUE, capacity,
    construction, layout, paint, structure, structure_and_layout,
};

#[test]
fn insertion_expands_the_factory_at_one_local_index() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let before = runtime.committed();
    let root = before.root();
    let list = before.fragment(root, LIST).unwrap();
    let first = before.keyed_member(list, KEY).unwrap();
    let second = before.keyed_member(list, SECOND_KEY).unwrap();
    let mut transaction = runtime.begin_transaction();
    transaction.insert_keyed(list, 9, 1).unwrap();

    let receipt = runtime.commit(transaction).unwrap();
    let after = runtime.committed();
    let inserted = after.keyed_member(list, 9).unwrap();
    let nested = after.fragment(inserted, NESTED_LIST).unwrap();
    let nested_member = after.keyed_member(nested, NESTED_KEY).unwrap();

    assert_eq!(
        after.keyed_members(list).unwrap().collect::<Vec<_>>(),
        vec![(KEY, first), (9, inserted), (SECOND_KEY, second)]
    );
    assert_eq!(after.template(inserted), Some(ITEM_BODY));
    assert_eq!(
        after.children(root).unwrap(),
        &[before.children(root).unwrap()[0], first, inserted, second]
    );
    let mutations = receipt.mutations().collect::<Vec<_>>();
    assert_eq!(mutations.len(), 1);
    let MutationRecordView::KeyInserted(insertion) = mutations[0] else {
        panic!("insert should emit a keyed insertion");
    };
    assert_eq!(insertion.fragment(), list);
    assert_eq!(insertion.key(), 9);
    assert_eq!(insertion.root(), inserted);
    assert_eq!(insertion.final_index(), 1);
    assert_eq!(
        insertion.created().collect::<Vec<_>>(),
        vec![
            ManifestEntry::Node(inserted),
            ManifestEntry::Fragment(nested),
            ManifestEntry::Node(nested_member),
        ]
    );
    assert_eq!(receipt.invalidation(), structure_and_layout());
}

#[test]
fn move_preserves_identity_and_uses_a_final_local_index() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let before = runtime.committed();
    let root = before.root();
    let list = before.fragment(root, LIST).unwrap();
    let first = before.keyed_member(list, KEY).unwrap();
    let second = before.keyed_member(list, SECOND_KEY).unwrap();
    let mut transaction = runtime.begin_transaction();
    transaction.move_keyed(list, SECOND_KEY, 0).unwrap();

    let receipt = runtime.commit(transaction).unwrap();
    let after = runtime.committed();

    assert_eq!(
        after.keyed_members(list).unwrap().collect::<Vec<_>>(),
        vec![(SECOND_KEY, second), (KEY, first)]
    );
    let mut mutations = receipt.mutations();
    assert_eq!(mutations.len(), 1);
    let Some(MutationRecordView::KeyMoved(movement)) = mutations.next() else {
        panic!("move should emit one keyed movement");
    };
    assert_eq!(movement.fragment(), list);
    assert_eq!(movement.key(), SECOND_KEY);
    assert_eq!(movement.root(), second);
    assert_eq!(movement.old_index(), 1);
    assert_eq!(movement.final_index(), 0);
    assert_eq!(receipt.invalidation(), structure_and_layout());
}

#[test]
fn keyed_update_uses_the_property_change_lane() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let committed = runtime.committed();
    let list = committed.fragment(committed.root(), LIST).unwrap();
    let member = committed.keyed_member(list, KEY).unwrap();
    let mut transaction = runtime.begin_transaction();
    transaction
        .update_keyed(list, KEY, VALUE, PropertyValue::ScalarI32(44))
        .unwrap();
    transaction
        .set_property(member, VALUE, PropertyValue::ScalarI32(55))
        .unwrap();

    let receipt = runtime.commit(transaction).unwrap();

    let mut mutations = receipt.mutations();
    assert_eq!(mutations.len(), 1);
    let Some(MutationRecordView::PropertyChanged(change)) = mutations.next() else {
        panic!("keyed update should emit a property change");
    };
    assert_eq!(change.node(), member);
    assert_eq!(change.property(), VALUE);
    assert_eq!(change.old_value(), &PropertyValue::ScalarI32(10));
    assert_eq!(change.new_value(), &PropertyValue::ScalarI32(55));
    assert_eq!(
        runtime.committed().property(member, VALUE),
        Some(&PropertyValue::ScalarI32(55))
    );
    assert_eq!(receipt.invalidation(), layout().union(paint()));
}

#[test]
fn removal_retires_nested_state_in_postorder() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let before = runtime.committed();
    let root = before.root();
    let list = before.fragment(root, LIST).unwrap();
    let member = before.keyed_member(list, KEY).unwrap();
    let nested = before.fragment(member, NESTED_LIST).unwrap();
    let nested_member = before.keyed_member(nested, NESTED_KEY).unwrap();
    let mut transaction = runtime.begin_transaction();
    transaction.remove_keyed(list, KEY).unwrap();

    let receipt = runtime.commit(transaction).unwrap();
    let after = runtime.committed();

    assert_eq!(
        after.keyed_members(list).unwrap().collect::<Vec<_>>(),
        vec![(SECOND_KEY, before.keyed_member(list, SECOND_KEY).unwrap())]
    );
    assert_eq!(after.template(member), None);
    assert_eq!(after.keyed_member(nested, NESTED_KEY), None);
    let mut mutations = receipt.mutations();
    assert_eq!(mutations.len(), 1);
    let Some(MutationRecordView::KeyRemoved(removal)) = mutations.next() else {
        panic!("remove should emit one keyed retirement");
    };
    assert_eq!(removal.fragment(), list);
    assert_eq!(removal.key(), KEY);
    assert_eq!(removal.root(), member);
    assert_eq!(removal.old_index(), 0);
    assert_eq!(
        removal.retired().collect::<Vec<_>>(),
        vec![
            ManifestEntry::Node(nested_member),
            ManifestEntry::Fragment(nested),
            ManifestEntry::Node(member),
        ]
    );
}

#[test]
fn reinserting_a_removed_key_creates_a_new_identity() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let committed = runtime.committed();
    let list = committed.fragment(committed.root(), LIST).unwrap();
    let retired = committed.keyed_member(list, KEY).unwrap();
    let retired_nested = committed.fragment(retired, NESTED_LIST).unwrap();
    let retired_nested_member = committed.keyed_member(retired_nested, NESTED_KEY).unwrap();
    let mut transaction = runtime.begin_transaction();
    transaction.remove_keyed(list, KEY).unwrap();
    transaction.insert_keyed(list, KEY, 1).unwrap();

    let receipt = runtime.commit(transaction).unwrap();
    let after = runtime.committed();
    let replacement = after.keyed_member(list, KEY).unwrap();
    let replacement_nested = after.fragment(replacement, NESTED_LIST).unwrap();
    let replacement_nested_member = after.keyed_member(replacement_nested, NESTED_KEY).unwrap();

    assert_ne!(retired, replacement);
    assert_ne!(retired_nested, replacement_nested);
    assert_ne!(retired_nested_member, replacement_nested_member);
    assert_eq!(after.template(retired), None);
    assert_eq!(after.keyed_member(retired_nested, NESTED_KEY), None);
    assert_eq!(
        after
            .keyed_members(list)
            .unwrap()
            .map(|item| item.0)
            .collect::<Vec<_>>(),
        vec![SECOND_KEY, KEY]
    );
    let mutations = receipt.mutations().collect::<Vec<_>>();
    assert_eq!(mutations.len(), 2);
    let MutationRecordView::KeyRemoved(removal) = mutations[0] else {
        panic!("first transition should retire the old member");
    };
    let MutationRecordView::KeyInserted(insertion) = mutations[1] else {
        panic!("second transition should create the replacement");
    };
    assert_eq!(
        removal.retired().collect::<Vec<_>>(),
        vec![
            ManifestEntry::Node(retired_nested_member),
            ManifestEntry::Fragment(retired_nested),
            ManifestEntry::Node(retired),
        ]
    );
    assert_eq!(
        insertion.created().collect::<Vec<_>>(),
        vec![
            ManifestEntry::Node(replacement),
            ManifestEntry::Fragment(replacement_nested),
            ManifestEntry::Node(replacement_nested_member),
        ]
    );
}

#[test]
fn equal_keys_are_local_and_foreign_fragments_fail_closed() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let before = runtime.committed();
    let root = before.root();
    let list = before.fragment(root, LIST).unwrap();
    let empty = before.fragment(root, EMPTY_LIST).unwrap();
    let list_members = before.keyed_members(list).unwrap().collect::<Vec<_>>();
    let mut local = runtime.begin_transaction();
    local.insert_keyed(empty, KEY, 0).unwrap();
    let receipt = runtime.commit(local).unwrap();
    let after = runtime.committed();
    let empty_root = after.keyed_member(empty, KEY).unwrap();

    assert_eq!(
        after.keyed_members(list).unwrap().collect::<Vec<_>>(),
        list_members
    );
    assert_eq!(
        after.keyed_members(empty).unwrap().collect::<Vec<_>>(),
        vec![(KEY, empty_root)]
    );
    assert_ne!(after.keyed_member(list, KEY), Some(empty_root));
    assert_eq!(
        after.children(root).unwrap(),
        &[
            before.children(root).unwrap()[0],
            list_members[0].1,
            list_members[1].1,
            empty_root,
        ]
    );
    assert_eq!(receipt.invalidation(), structure().union(paint()));

    let mut duplicate = runtime.begin_transaction();
    duplicate.insert_keyed(list, KEY, 0).unwrap();
    let error = runtime.commit(duplicate).unwrap_err();
    assert_eq!(error.kind(), TransactionErrorKind::DuplicateKey);
    assert_eq!(error.operation_index(), Some(0));

    let foreign_runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let foreign_fragment = foreign_runtime
        .committed()
        .fragment(foreign_runtime.committed().root(), LIST)
        .unwrap();
    let mut foreign = runtime.begin_transaction();
    foreign.remove_keyed(foreign_fragment, KEY).unwrap();
    let error = runtime.commit(foreign).unwrap_err();
    assert_eq!(error.kind(), TransactionErrorKind::MissingFragment);
    assert_eq!(error.operation_index(), Some(0));
}

#[test]
fn keyed_index_errors_roll_back_prior_operations() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let before = runtime.committed();
    let list = before.fragment(before.root(), LIST).unwrap();
    let mut transaction = runtime.begin_transaction();
    transaction.move_keyed(list, SECOND_KEY, 0).unwrap();
    transaction.insert_keyed(list, 99, usize::MAX).unwrap();

    let error = runtime.commit(transaction).unwrap_err();

    assert_eq!(error.kind(), TransactionErrorKind::IndexOutOfBounds);
    assert_eq!(error.operation_index(), Some(1));
    assert!(before.shares_state_with(&runtime.committed()));
}

#[test]
fn same_position_move_is_a_true_noop() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let before = runtime.committed();
    let list = before.fragment(before.root(), LIST).unwrap();
    let mut transaction = runtime.begin_transaction();
    transaction.move_keyed(list, KEY, 0).unwrap();

    let receipt = runtime.commit(transaction).unwrap();

    assert!(receipt.is_empty());
    assert_eq!(receipt.generation(), before.generation());
    assert!(before.shares_state_with(&runtime.committed()));
}

#[test]
fn structural_round_trip_preserves_every_lifecycle_record() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let before = runtime.committed();
    let list = before.fragment(before.root(), LIST).unwrap();
    let existing = before.keyed_members(list).unwrap().collect::<Vec<_>>();
    let mut transaction = runtime.begin_transaction();
    transaction.insert_keyed(list, 99, 2).unwrap();
    transaction
        .update_keyed(list, 99, VALUE, PropertyValue::ScalarI32(77))
        .unwrap();
    transaction.move_keyed(list, 99, 0).unwrap();
    transaction.remove_keyed(list, 99).unwrap();

    let receipt = runtime.commit(transaction).unwrap();
    let after = runtime.committed();
    let mutations = receipt.mutations().collect::<Vec<_>>();

    assert_eq!(receipt.generation().get(), before.generation().get() + 1);
    assert_eq!(mutations.len(), 4);
    let MutationRecordView::KeyInserted(insertion) = mutations[0] else {
        panic!("first record should be insertion");
    };
    let MutationRecordView::PropertyChanged(change) = mutations[1] else {
        panic!("second record should retain the property transition");
    };
    let MutationRecordView::KeyMoved(movement) = mutations[2] else {
        panic!("third record should be movement");
    };
    let MutationRecordView::KeyRemoved(removal) = mutations[3] else {
        panic!("fourth record should be retirement");
    };
    assert_eq!(change.node(), insertion.root());
    assert_eq!(movement.root(), insertion.root());
    assert_eq!(removal.root(), insertion.root());
    let created = insertion.created().collect::<Vec<_>>();
    let retired = removal.retired().collect::<Vec<_>>();
    assert_eq!(created.len(), retired.len());
    assert_eq!(created[0], ManifestEntry::Node(insertion.root()));
    assert_eq!(retired.last(), Some(&ManifestEntry::Node(insertion.root())));
    assert_eq!(
        after.keyed_members(list).unwrap().collect::<Vec<_>>(),
        existing
    );
    assert_eq!(
        receipt.invalidation(),
        structure().union(layout()).union(paint())
    );
}
