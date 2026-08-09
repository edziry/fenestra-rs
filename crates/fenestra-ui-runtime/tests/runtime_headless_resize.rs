#[path = "support/headless_projection.rs"]
mod headless_projection;
mod support;

use fenestra_ui_ir::prototype::{InvalidationClass, InvalidationSet, PropertyValue};
use fenestra_ui_runtime::prototype::{
    CapacityKind, HeadlessPoint, HeadlessProjectionErrorKind, HeadlessSurface,
    HeadlessSurfaceChangeView, MutationRecordView, TransactionErrorKind, UiRuntime,
};

use headless_projection::{nodes, rect, runtime};
use support::headless::{HEIGHT, WIDTH, construction, runtime_capacity};
use support::headless_projection_state::{ProjectionRecords, capture_projection};

fn resize_invalidation() -> InvalidationSet {
    [
        InvalidationClass::Surface,
        InvalidationClass::Layout,
        InvalidationClass::HitTest,
        InvalidationClass::Semantics,
        InvalidationClass::Paint,
        InvalidationClass::Composition,
    ]
    .into_iter()
    .fold(InvalidationSet::NONE, |set, class| {
        set.union(InvalidationSet::from_class(class))
    })
}

fn assert_surface_change(
    change: HeadlessSurfaceChangeView<'_>,
    old_surface: HeadlessSurface,
    new_surface: HeadlessSurface,
) {
    assert_eq!(change.old_surface(), old_surface);
    assert_eq!(change.new_surface(), new_surface);
}

fn clip_expected(expected: &mut ProjectionRecords, fixture: headless_projection::ProjectionNodes) {
    expected.geometry_mut(fixture.root).clip = rect(0, 0, 25, 15);
    expected.geometry_mut(fixture.container).clip = rect(0, 0, 25, 15);
    expected.geometry_mut(fixture.control).clip = rect(0, 0, 25, 10);
    expected.geometry_mut(fixture.first).clip = rect(0, 10, 25, 5);
    expected.geometry_mut(fixture.second).clip = rect(0, 22, 25, 0);
    expected.hit_mut(fixture.control).clip = rect(0, 0, 25, 10);
    expected.hit_mut(fixture.first).clip = rect(0, 10, 25, 5);
    expected.hits.retain(|record| record.node != fixture.second);
    expected.scene_mut(fixture.root).rectangle = rect(0, 0, 25, 15);
    expected.scene_mut(fixture.container).rectangle = rect(0, 0, 25, 15);
    expected.scene_mut(fixture.control).rectangle = rect(0, 0, 25, 10);
    expected.scene_mut(fixture.first).rectangle = rect(0, 10, 25, 5);
    expected
        .scenes
        .retain(|record| record.node != fixture.second);
}

#[test]
fn resize_publishes_one_typed_record_and_retains_the_old_projection() {
    let mut runtime = runtime(HeadlessSurface::new(120, 90));
    let before = runtime.committed();
    let fixture = nodes(&before);
    let retained = capture_projection(&before);
    let raw_bounds = retained
        .geometry
        .iter()
        .map(|record| (record.node, record.bounds))
        .collect::<Vec<_>>();
    let resized = HeadlessSurface::new(25, 15);
    let mut transaction = runtime.begin_transaction();
    transaction
        .resize_headless(resized)
        .expect("headless resize should stage");

    let receipt = runtime
        .commit(transaction)
        .expect("valid headless resize should publish");
    let after = runtime.committed();
    let mut expected = retained.clone();
    expected.generation = after.generation();
    expected.surface = resized;
    clip_expected(&mut expected, fixture);

    assert_eq!(receipt.generation(), after.generation());
    assert_eq!(after.generation().get(), before.generation().get() + 1);
    assert_eq!(capture_projection(&after), expected);
    assert_eq!(capture_projection(&before), retained);
    assert_eq!(
        capture_projection(&after)
            .geometry
            .iter()
            .map(|record| (record.node, record.bounds))
            .collect::<Vec<_>>(),
        raw_bounds
    );
    assert_eq!(
        before
            .headless_projection()
            .expect("retained projection should exist")
            .hit_test(HeadlessPoint::new(5, 22)),
        Some(fixture.second)
    );
    assert_eq!(
        after
            .headless_projection()
            .expect("resized projection should exist")
            .hit_test(HeadlessPoint::new(5, 22)),
        None
    );
    assert!(!before.shares_state_with(&after));
}

#[test]
fn resize_emits_one_typed_mutation_and_complete_invalidation() {
    let mut runtime = runtime(HeadlessSurface::new(120, 90));
    let before = runtime.committed();
    let resized = HeadlessSurface::new(90, 70);
    let mut transaction = runtime.begin_transaction();
    transaction
        .resize_headless(resized)
        .expect("headless resize should stage");

    let receipt = runtime
        .commit(transaction)
        .expect("valid resize should publish");
    let after = runtime.committed();
    let mut mutations = receipt.mutations();

    assert_eq!(receipt.generation(), after.generation());
    assert_eq!(after.generation().get(), before.generation().get() + 1);
    assert_eq!(receipt.invalidation(), resize_invalidation());
    assert_eq!(mutations.len(), 1);
    let mutation = mutations.next();
    assert_eq!(format!("{:?}", mutation), "Some(HeadlessSurfaceChanged)");
    let Some(MutationRecordView::HeadlessSurfaceChanged(change)) = mutation else {
        panic!("resize should emit one headless surface change");
    };
    assert_surface_change(change, HeadlessSurface::new(120, 90), resized);
}

#[test]
fn resize_to_the_committed_surface_is_an_exact_noop() {
    let mut runtime = runtime(HeadlessSurface::new(120, 90));
    let before = runtime.committed();
    let projection = capture_projection(&before);
    let mut transaction = runtime.begin_transaction();
    transaction
        .resize_headless(HeadlessSurface::new(120, 90))
        .expect("same surface should stage");

    let receipt = runtime
        .commit(transaction)
        .expect("same surface should remain valid");
    let after = runtime.committed();

    assert!(receipt.is_empty());
    assert!(receipt.invalidation().is_empty());
    assert_eq!(receipt.generation(), before.generation());
    assert!(before.shares_state_with(&after));
    assert_eq!(capture_projection(&after), projection);
}

#[test]
fn resize_participates_in_the_existing_operation_ceiling() {
    let mut runtime = runtime(HeadlessSurface::new(120, 90));
    let before = runtime.committed();
    let root = before.root();
    let mut transaction = runtime.begin_transaction();
    for _ in 0..8 {
        transaction
            .set_property(root, WIDTH, PropertyValue::ScalarI32(100))
            .expect("operation at the inclusive ceiling should stage");
    }

    let staged = transaction
        .resize_headless(HeadlessSurface::new(90, 70))
        .expect_err("ninth operation should poison the transaction");

    assert_eq!(
        staged.kind(),
        TransactionErrorKind::CapacityExceeded(CapacityKind::Operations)
    );
    assert_eq!(staged.operation_index(), Some(8));
    let committed = runtime
        .commit(transaction)
        .expect_err("poisoned resize transaction should reject commit");
    assert_eq!(committed, staged);
    assert!(before.shares_state_with(&runtime.committed()));
}

#[test]
fn ordinary_runtime_rejects_resize_at_the_original_operation_index() {
    let mut runtime = UiRuntime::new(construction(), runtime_capacity())
        .expect("ordinary runtime should initialize");
    let before = runtime.committed();
    let root = before.root();
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(root, WIDTH, PropertyValue::ScalarI32(84))
        .expect("prior property should stage");
    transaction
        .resize_headless(HeadlessSurface::new(90, 70))
        .expect("unavailable resize should fail only at commit");

    let error = runtime
        .commit(transaction)
        .expect_err("ordinary runtime cannot publish a headless surface");
    let after = runtime.committed();

    assert_eq!(error.kind(), TransactionErrorKind::HeadlessUnavailable);
    assert_eq!(error.operation_index(), Some(1));
    assert!(before.shares_state_with(&after));
    assert_eq!(after.generation(), before.generation());
    assert_eq!(
        after.property(root, WIDTH),
        Some(&PropertyValue::ScalarI32(100))
    );
    assert!(after.headless_projection().is_none());
}

#[test]
fn final_negative_resize_wins_and_rolls_back_the_exact_allocation() {
    let mut runtime = runtime(HeadlessSurface::new(120, 90));
    let before = runtime.committed();
    let retained = capture_projection(&before);
    let mut transaction = runtime.begin_transaction();
    transaction
        .resize_headless(HeadlessSurface::new(120, 90))
        .expect("leading resize no-op should stage");
    transaction
        .resize_headless(HeadlessSurface::new(90, 70))
        .expect("valid intermediate resize should stage");
    transaction
        .resize_headless(HeadlessSurface::new(-1, 70))
        .expect("invalid final resize should stage for final validation");

    let error = runtime
        .commit(transaction)
        .expect_err("negative final surface should reject publication");
    let after = runtime.committed();

    assert_eq!(
        error.kind(),
        TransactionErrorKind::Headless(HeadlessProjectionErrorKind::InvalidSurface)
    );
    assert_eq!(error.operation_index(), Some(2));
    assert!(before.shares_state_with(&after));
    assert_eq!(after.generation(), before.generation());
    assert_eq!(capture_projection(&after), retained);
}

#[test]
fn final_surface_validation_precedes_negative_geometry() {
    let mut runtime = runtime(HeadlessSurface::new(120, 90));
    let before = runtime.committed();
    let fixture = nodes(&before);
    let retained = capture_projection(&before);
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(fixture.second, WIDTH, PropertyValue::ScalarI32(-1))
        .expect("negative width should stage");
    transaction
        .resize_headless(HeadlessSurface::new(-1, 70))
        .expect("invalid surface should stage for final validation");

    let error = runtime
        .commit(transaction)
        .expect_err("surface validity should precede negative geometry");
    let after = runtime.committed();

    assert_eq!(
        error.kind(),
        TransactionErrorKind::Headless(HeadlessProjectionErrorKind::InvalidSurface)
    );
    assert_eq!(error.operation_index(), Some(1));
    assert!(before.shares_state_with(&after));
    assert_eq!(capture_projection(&after), retained);
}

#[test]
fn invalid_intermediate_resize_is_ignored_when_the_final_surface_is_valid() {
    let mut runtime = runtime(HeadlessSurface::new(120, 90));
    let before = runtime.committed();
    let final_surface = HeadlessSurface::new(90, 70);
    let mut transaction = runtime.begin_transaction();
    transaction
        .resize_headless(HeadlessSurface::new(-1, 90))
        .expect("invalid intermediate resize should stage");
    transaction
        .resize_headless(final_surface)
        .expect("valid final resize should stage");

    let receipt = runtime
        .commit(transaction)
        .expect("only the final candidate surface should be validated");
    let after = runtime.committed();
    let mut mutations = receipt.mutations();

    assert_eq!(mutations.len(), 1);
    let Some(MutationRecordView::HeadlessSurfaceChanged(change)) = mutations.next() else {
        panic!("coalesced resize should emit one surface change");
    };
    assert_surface_change(change, HeadlessSurface::new(120, 90), final_surface);
    assert_eq!(
        after
            .headless_projection()
            .expect("headless projection should remain available")
            .surface(),
        final_surface
    );
    assert_eq!(receipt.invalidation(), resize_invalidation());
    assert_eq!(after.generation().get(), before.generation().get() + 1);
}

#[test]
fn later_layout_overflow_rolls_back_a_prior_resize() {
    let mut runtime = runtime(HeadlessSurface::new(120, 90));
    let before = runtime.committed();
    let fixture = nodes(&before);
    let retained = capture_projection(&before);
    let mut transaction = runtime.begin_transaction();
    transaction
        .resize_headless(HeadlessSurface::new(25, 15))
        .expect("valid resize should stage");
    transaction
        .set_property(fixture.control, HEIGHT, PropertyValue::ScalarI32(i32::MAX))
        .expect("overflowing height should stage");

    let error = runtime
        .commit(transaction)
        .expect_err("layout overflow should reject the complete draft");
    let after = runtime.committed();

    assert_eq!(
        error.kind(),
        TransactionErrorKind::Headless(HeadlessProjectionErrorKind::ArithmeticExhausted)
    );
    assert_eq!(error.operation_index(), None);
    assert!(before.shares_state_with(&after));
    assert_eq!(after.generation(), before.generation());
    assert_eq!(capture_projection(&after), retained);
    assert_eq!(
        after.property(fixture.control, HEIGHT),
        Some(&PropertyValue::ScalarI32(10))
    );
}
