mod support;

use fenestra_ui_ir::prototype::{PropertyId, PropertyValue};
use fenestra_ui_runtime::prototype::{
    MutationRecordView, TransactionError, TransactionErrorKind, UiRuntime,
};

use support::{KEY, LIST, VALUE, WIDTH, capacity, construction};

fn assert_operation_error(error: TransactionError, expected: TransactionErrorKind) {
    assert_eq!(error.kind(), expected);
    assert_eq!(error.operation_index(), Some(0));
}

#[test]
fn direct_errors_follow_node_property_type_priority() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let root = runtime.committed().root();
    let foreign = UiRuntime::new(construction(), capacity()).unwrap();
    let mut missing_node = runtime.begin_transaction();
    missing_node
        .set_property(
            foreign.committed().root(),
            PropertyId::new(u32::MAX),
            PropertyValue::Bool(false),
        )
        .unwrap();
    assert_operation_error(
        runtime.commit(missing_node).unwrap_err(),
        TransactionErrorKind::MissingNode,
    );

    let mut unknown_property = runtime.begin_transaction();
    unknown_property
        .set_property(root, PropertyId::new(u32::MAX), PropertyValue::Bool(false))
        .unwrap();
    assert_operation_error(
        runtime.commit(unknown_property).unwrap_err(),
        TransactionErrorKind::UnknownProperty,
    );

    let mut wrong_type = runtime.begin_transaction();
    wrong_type
        .set_property(root, WIDTH, PropertyValue::Bool(false))
        .unwrap();
    assert_operation_error(
        runtime.commit(wrong_type).unwrap_err(),
        TransactionErrorKind::PropertyTypeMismatch,
    );
}

#[test]
fn keyed_errors_follow_fragment_key_property_type_priority() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let committed = runtime.committed();
    let list = committed.fragment(committed.root(), LIST).unwrap();
    let missing_key = u64::MAX;

    let mut update_missing = runtime.begin_transaction();
    update_missing
        .update_keyed(
            list,
            missing_key,
            PropertyId::new(u32::MAX),
            PropertyValue::Bool(false),
        )
        .unwrap();
    assert_operation_error(
        runtime.commit(update_missing).unwrap_err(),
        TransactionErrorKind::MissingKey,
    );

    let mut update_unknown = runtime.begin_transaction();
    update_unknown
        .update_keyed(
            list,
            KEY,
            PropertyId::new(u32::MAX),
            PropertyValue::Bool(false),
        )
        .unwrap();
    assert_operation_error(
        runtime.commit(update_unknown).unwrap_err(),
        TransactionErrorKind::UnknownProperty,
    );

    let mut update_type = runtime.begin_transaction();
    update_type
        .update_keyed(list, KEY, VALUE, PropertyValue::Bool(false))
        .unwrap();
    assert_operation_error(
        runtime.commit(update_type).unwrap_err(),
        TransactionErrorKind::PropertyTypeMismatch,
    );
}

#[test]
fn duplicate_and_missing_keys_win_before_indices() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let committed = runtime.committed();
    let list = committed.fragment(committed.root(), LIST).unwrap();
    let mut duplicate = runtime.begin_transaction();
    duplicate.insert_keyed(list, KEY, usize::MAX).unwrap();
    assert_operation_error(
        runtime.commit(duplicate).unwrap_err(),
        TransactionErrorKind::DuplicateKey,
    );

    let mut missing = runtime.begin_transaction();
    missing.move_keyed(list, u64::MAX, usize::MAX).unwrap();
    assert_operation_error(
        runtime.commit(missing).unwrap_err(),
        TransactionErrorKind::MissingKey,
    );
}

#[test]
fn keyed_lanes_report_missing_fragments_before_other_fields() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let foreign = UiRuntime::new(construction(), capacity()).unwrap();
    let committed = foreign.committed();
    let fragment = committed.fragment(committed.root(), LIST).unwrap();

    let mut insert = runtime.begin_transaction();
    insert.insert_keyed(fragment, KEY, usize::MAX).unwrap();
    assert_operation_error(
        runtime.commit(insert).unwrap_err(),
        TransactionErrorKind::MissingFragment,
    );

    let mut movement = runtime.begin_transaction();
    movement.move_keyed(fragment, u64::MAX, usize::MAX).unwrap();
    assert_operation_error(
        runtime.commit(movement).unwrap_err(),
        TransactionErrorKind::MissingFragment,
    );

    let mut update = runtime.begin_transaction();
    update
        .update_keyed(
            fragment,
            u64::MAX,
            PropertyId::new(u32::MAX),
            PropertyValue::Bool(false),
        )
        .unwrap();
    assert_operation_error(
        runtime.commit(update).unwrap_err(),
        TransactionErrorKind::MissingFragment,
    );

    let mut removal = runtime.begin_transaction();
    removal.remove_keyed(fragment, u64::MAX).unwrap();
    assert_operation_error(
        runtime.commit(removal).unwrap_err(),
        TransactionErrorKind::MissingFragment,
    );
}

#[test]
fn move_index_and_remove_key_errors_keep_operation_context() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let committed = runtime.committed();
    let list = committed.fragment(committed.root(), LIST).unwrap();

    let mut movement = runtime.begin_transaction();
    movement.move_keyed(list, KEY, usize::MAX).unwrap();
    assert_operation_error(
        runtime.commit(movement).unwrap_err(),
        TransactionErrorKind::IndexOutOfBounds,
    );

    let mut removal = runtime.begin_transaction();
    removal.remove_keyed(list, u64::MAX).unwrap();
    assert_operation_error(
        runtime.commit(removal).unwrap_err(),
        TransactionErrorKind::MissingKey,
    );
}

#[test]
fn stale_base_wins_before_invalid_operations() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let root = runtime.committed().root();
    let mut stale = runtime.begin_transaction();
    stale
        .set_property(root, PropertyId::new(u32::MAX), PropertyValue::Bool(false))
        .unwrap();
    let mut winner = runtime.begin_transaction();
    winner
        .set_property(root, WIDTH, PropertyValue::ScalarI32(200))
        .unwrap();
    drop(runtime.commit(winner).unwrap());

    let error = runtime.commit(stale).unwrap_err();

    assert_eq!(error.kind(), TransactionErrorKind::StaleBase);
    assert_eq!(error.operation_index(), None);
}

#[test]
fn the_first_invalid_operation_wins_within_one_sequence() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let before = runtime.committed();
    let root = before.root();
    let list = before.fragment(root, LIST).unwrap();
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(root, PropertyId::new(u32::MAX), PropertyValue::Bool(false))
        .unwrap();
    transaction.remove_keyed(list, u64::MAX).unwrap();

    assert_operation_error(
        runtime.commit(transaction).unwrap_err(),
        TransactionErrorKind::UnknownProperty,
    );
    assert!(before.shares_state_with(&runtime.committed()));
}

#[test]
fn semantic_errors_win_before_apply_time_capacity() {
    let limits = capacity().with_live_nodes(6);
    let mut runtime = UiRuntime::new(construction(), limits).unwrap();
    let committed = runtime.committed();
    let list = committed.fragment(committed.root(), LIST).unwrap();
    let mut duplicate = runtime.begin_transaction();
    duplicate.insert_keyed(list, KEY, 2).unwrap();

    assert_operation_error(
        runtime.commit(duplicate).unwrap_err(),
        TransactionErrorKind::DuplicateKey,
    );
}

#[test]
fn diagnostics_keep_values_and_physical_identities_private() {
    let mut runtime = UiRuntime::new(construction(), capacity()).unwrap();
    let committed = runtime.committed();
    let list = committed.fragment(committed.root(), LIST).unwrap();
    let mut transaction = runtime.begin_transaction();
    transaction
        .update_keyed(list, KEY, VALUE, PropertyValue::Bool(false))
        .unwrap();
    let error = runtime.commit(transaction).unwrap_err();
    let debug = format!("{error:?}");
    let display = error.to_string();

    assert!(!debug.contains("false"));
    assert!(!display.contains("false"));
    assert_eq!(format!("{list:?}"), "FragmentId(..)");
    assert_eq!(error.operation_index(), Some(0));

    let root = runtime.committed().root();
    let mut property = runtime.begin_transaction();
    property
        .set_property(root, WIDTH, PropertyValue::ScalarI32(321))
        .unwrap();
    let property_receipt = runtime.commit(property).unwrap();
    let property_record = property_receipt.mutations().next().unwrap();
    let committed_debug = format!("{:?}", runtime.committed());
    let receipt_debug = format!("{property_receipt:?}");
    let record_debug = format!("{property_record:?}");
    for output in [&committed_debug, &receipt_debug, &record_debug] {
        assert!(!output.contains("321"));
        assert!(!output.contains("arena"));
        assert!(!output.contains("domain"));
        assert!(!output.contains("slot"));
    }

    let mut insertion = runtime.begin_transaction();
    insertion.insert_keyed(list, 99, 2).unwrap();
    let insertion_receipt = runtime.commit(insertion).unwrap();
    let Some(MutationRecordView::KeyInserted(insertion)) = insertion_receipt.mutations().next()
    else {
        panic!("insert should retain its typed view");
    };
    let manifest_debug = format!("{:?}", insertion.created().collect::<Vec<_>>());
    assert!(!manifest_debug.contains("arena"));
    assert!(!manifest_debug.contains("domain"));
    assert!(!manifest_debug.contains("slot"));
    assert_eq!(format!("{root:?}"), "NodeId(..)");
}
