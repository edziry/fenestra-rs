mod support;

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{
    CommittedRuntimeSnapshot, FragmentId, HeadlessProjectionErrorKind, HeadlessProjectionLimitKind,
    NodeId, TransactionErrorKind, UiRuntime,
};

use support::headless::{
    COLOR, DIRECT_COLOR, FIRST_KEY, INSERTED_KEY, ITEM_STYLE_COLOR, ITEMS, exact_style,
    runtime_capacity,
};
use support::headless_spec::{HeadlessSpecBuilder, surface};

const FOURTH_KEY: u64 = 40;

fn runtime(computed_styles: usize) -> UiRuntime {
    UiRuntime::new_headless(
        exact_style(),
        HeadlessSpecBuilder::new()
            .with_computed_capacity(computed_styles)
            .build(),
        surface(),
        runtime_capacity(),
    )
    .expect("headless runtime should initialize")
}

fn control_and_items(committed: &CommittedRuntimeSnapshot) -> (NodeId, FragmentId) {
    let root = committed.root();
    let container = committed.children(root).expect("root should be live")[0];
    let control = committed
        .children(container)
        .expect("container should be live")[0];
    let items = committed
        .fragment(container, ITEMS)
        .expect("item region should exist");
    (control, items)
}

fn computed_capacity_error() -> TransactionErrorKind {
    TransactionErrorKind::Headless(HeadlessProjectionErrorKind::CapacityExceeded(
        HeadlessProjectionLimitKind::ComputedStyles,
    ))
}

#[test]
fn effective_property_commit_rebuilds_at_exact_computed_capacity() {
    let mut runtime = runtime(5);
    let before = runtime.committed();
    let (control, _) = control_and_items(&before);
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(control, COLOR, PropertyValue::Rgba8(DIRECT_COLOR))
        .expect("direct color should stage");

    let receipt = runtime
        .commit(transaction)
        .expect("same-size computed rebuild should publish");
    let after = runtime.committed();

    assert!(!receipt.is_empty());
    assert!(!before.shares_state_with(&after));
    assert_eq!(after.generation().get(), 1);
    assert_eq!(
        after
            .headless_projection()
            .expect("headless projection should exist")
            .computed_style(control)
            .expect("control computed style should exist")
            .property(COLOR),
        Some(&PropertyValue::Rgba8(DIRECT_COLOR))
    );
}

#[test]
fn headless_noop_at_exact_capacity_preserves_exact_state() {
    let mut runtime = runtime(5);
    let before = runtime.committed();
    let (control, _) = control_and_items(&before);
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(
            control,
            COLOR,
            PropertyValue::Rgba8(support::headless::CONTROL_STYLE_COLOR),
        )
        .expect("same color should stage");

    let receipt = runtime
        .commit(transaction)
        .expect("same color should remain a no-op");
    let after = runtime.committed();

    assert!(receipt.is_empty());
    assert!(before.shares_state_with(&after));
    assert_eq!(after.generation().get(), 0);
}

#[test]
fn computed_capacity_failure_rolls_back_prior_property_change() {
    let mut runtime = runtime(5);
    let before = runtime.committed();
    let (control, items) = control_and_items(&before);
    let counts = (
        before.node_count(),
        before.fragment_count(),
        before.property_slot_count(),
    );
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(control, COLOR, PropertyValue::Rgba8(DIRECT_COLOR))
        .expect("direct color should stage");
    transaction
        .insert_keyed(items, INSERTED_KEY, 2)
        .expect("item insert should stage");

    let error = runtime
        .commit(transaction)
        .expect_err("insert should exceed computed capacity");
    let after = runtime.committed();

    assert_eq!(error.kind(), computed_capacity_error());
    assert_eq!(error.operation_index(), Some(1));
    assert!(before.shares_state_with(&after));
    assert_eq!(after.generation(), before.generation());
    assert_eq!(
        (
            after.node_count(),
            after.fragment_count(),
            after.property_slot_count(),
        ),
        counts
    );
    assert_eq!(after.keyed_member(items, INSERTED_KEY), None);
    assert_eq!(
        after.property(control, COLOR),
        Some(&PropertyValue::Rgba8(
            support::headless::CONTROL_STYLE_COLOR,
        ))
    );
    assert_eq!(
        after
            .headless_projection()
            .expect("headless projection should exist")
            .computed_style(control)
            .expect("control computed style should exist")
            .property(COLOR),
        Some(&PropertyValue::Rgba8(
            support::headless::CONTROL_STYLE_COLOR,
        ))
    );
}

#[test]
fn computed_capacity_is_cumulative_across_insertions() {
    let mut runtime = runtime(6);
    let before = runtime.committed();
    let (_, items) = control_and_items(&before);
    let mut transaction = runtime.begin_transaction();
    transaction
        .insert_keyed(items, INSERTED_KEY, 2)
        .expect("first item insert should stage");
    transaction
        .insert_keyed(items, FOURTH_KEY, 3)
        .expect("second item insert should stage");

    let error = runtime
        .commit(transaction)
        .expect_err("second insert should exceed computed capacity");
    let after = runtime.committed();

    assert_eq!(error.kind(), computed_capacity_error());
    assert_eq!(error.operation_index(), Some(1));
    assert!(before.shares_state_with(&after));
    assert_eq!(after.keyed_member(items, INSERTED_KEY), None);
    assert_eq!(after.keyed_member(items, FOURTH_KEY), None);
}

#[test]
fn operation_semantics_precede_computed_capacity() {
    let mut runtime = runtime(5);
    let before = runtime.committed();
    let (_, items) = control_and_items(&before);
    let mut transaction = runtime.begin_transaction();
    transaction
        .insert_keyed(items, FIRST_KEY, usize::MAX)
        .expect("duplicate item insert should stage");

    let error = runtime
        .commit(transaction)
        .expect_err("duplicate key should be rejected first");

    assert_eq!(error.kind(), TransactionErrorKind::DuplicateKey);
    assert_eq!(error.operation_index(), Some(0));
    assert!(before.shares_state_with(&runtime.committed()));
    assert_eq!(
        runtime.committed().property(
            before.keyed_member(items, FIRST_KEY).expect("item exists"),
            COLOR,
        ),
        Some(&PropertyValue::Rgba8(ITEM_STYLE_COLOR))
    );
}
