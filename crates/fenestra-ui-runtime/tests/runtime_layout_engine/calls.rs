use std::sync::Arc;

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{
    HeadlessProjectionCapacity, HeadlessProjectionErrorKind, HeadlessProjectionLimitKind,
    HeadlessSurface, TransactionErrorKind,
};

use super::headless_projection::nodes;
use super::layout_support::{SpyEngineV1, SpyState, try_runtime_with_engine};
use super::support::headless::{
    COLOR, DIRECT_COLOR, HEIGHT, INSERTED_KEY, ROOT_WIDTH, WIDTH, exact_style, runtime_capacity,
};
use super::support::headless_spec::{HeadlessSpecBuilder, surface};

#[test]
fn engine_calls_only_follow_successful_preflights_and_effective_commits() {
    let state = Arc::new(SpyState::default());
    let spec = HeadlessSpecBuilder::new()
        .with_capacity(HeadlessProjectionCapacity::new(5, 5, 1, 8, 8))
        .build();
    let mut runtime = try_runtime_with_engine(
        exact_style(),
        spec,
        surface(),
        runtime_capacity().with_retained_generations(8),
        Box::new(SpyEngineV1::new(Arc::clone(&state))),
    )
    .expect("counted runtime should initialize");
    assert_eq!(state.calls(), 1);

    let initial = runtime.committed();
    let fixture = nodes(&initial);
    let mut no_op = runtime.begin_transaction();
    no_op
        .set_property(fixture.root, WIDTH, PropertyValue::ScalarI32(ROOT_WIDTH))
        .expect("true no-op should stage");
    assert!(
        runtime
            .commit(no_op)
            .expect("true no-op should commit")
            .is_empty()
    );
    assert_eq!(state.calls(), 1);

    let mut invalid_surface = runtime.begin_transaction();
    invalid_surface
        .resize_headless(HeadlessSurface::new(-1, 90))
        .expect("invalid final surface should stage");
    let error = runtime
        .commit(invalid_surface)
        .expect_err("invalid surface preflight should reject");
    assert_eq!(
        error.kind(),
        TransactionErrorKind::Headless(HeadlessProjectionErrorKind::InvalidSurface)
    );
    assert_eq!(error.operation_index(), Some(0));
    assert_eq!(state.calls(), 1);

    let mut fixed_capacity = runtime.begin_transaction();
    fixed_capacity
        .insert_keyed(fixture.items, INSERTED_KEY, 1)
        .expect("one-over keyed insertion should stage");
    let error = runtime
        .commit(fixed_capacity)
        .expect_err("fixed projection preflight should reject");
    assert_eq!(
        error.kind(),
        TransactionErrorKind::Headless(HeadlessProjectionErrorKind::CapacityExceeded(
            HeadlessProjectionLimitKind::ComputedStyles,
        ))
    );
    assert_eq!(error.operation_index(), Some(0));
    assert_eq!(state.calls(), 1);

    let mut negative = runtime.begin_transaction();
    negative
        .set_property(fixture.control, WIDTH, PropertyValue::ScalarI32(-1))
        .expect("negative geometry should stage");
    let error = runtime
        .commit(negative)
        .expect_err("global negative scan should reject");
    assert_eq!(
        error.kind(),
        TransactionErrorKind::Headless(HeadlessProjectionErrorKind::NegativeGeometry)
    );
    assert_eq!(error.operation_index(), Some(0));
    assert_eq!(state.calls(), 1);

    let mut layout = runtime.begin_transaction();
    layout
        .set_property(fixture.root, WIDTH, PropertyValue::ScalarI32(101))
        .expect("effective layout update should stage");
    runtime
        .commit(layout)
        .expect("effective layout update should commit");
    assert_eq!(state.calls(), 2);

    let mut paint = runtime.begin_transaction();
    paint
        .set_property(fixture.control, COLOR, PropertyValue::Rgba8(DIRECT_COLOR))
        .expect("effective paint update should stage");
    runtime
        .commit(paint)
        .expect("effective paint update should commit");
    assert_eq!(state.calls(), 3);

    let mut resize = runtime.begin_transaction();
    resize
        .resize_headless(HeadlessSurface::new(110, 85))
        .expect("effective resize should stage");
    runtime
        .commit(resize)
        .expect("effective resize should commit");
    assert_eq!(state.calls(), 4);
}

#[test]
fn every_effective_structural_commit_invokes_the_engine_once() {
    let state = Arc::new(SpyState::default());
    let mut runtime = try_runtime_with_engine(
        exact_style(),
        HeadlessSpecBuilder::new().build(),
        surface(),
        runtime_capacity().with_retained_generations(8),
        Box::new(SpyEngineV1::new(Arc::clone(&state))),
    )
    .expect("counted structural runtime should initialize");
    let initial = runtime.committed();
    let fixture = nodes(&initial);
    assert_eq!(state.calls(), 1);

    let mut insert = runtime.begin_transaction();
    insert
        .insert_keyed(fixture.items, INSERTED_KEY, 1)
        .expect("effective insertion should stage");
    runtime
        .commit(insert)
        .expect("effective insertion should commit");
    assert_eq!(state.calls(), 2);

    let mut movement = runtime.begin_transaction();
    movement
        .move_keyed(fixture.items, INSERTED_KEY, 2)
        .expect("effective movement should stage");
    runtime
        .commit(movement)
        .expect("effective movement should commit");
    assert_eq!(state.calls(), 3);

    let mut update = runtime.begin_transaction();
    update
        .update_keyed(
            fixture.items,
            INSERTED_KEY,
            HEIGHT,
            PropertyValue::ScalarI32(14),
        )
        .expect("effective keyed update should stage");
    runtime
        .commit(update)
        .expect("effective keyed update should commit");
    assert_eq!(state.calls(), 4);

    let mut removal = runtime.begin_transaction();
    removal
        .remove_keyed(fixture.items, INSERTED_KEY)
        .expect("effective removal should stage");
    runtime
        .commit(removal)
        .expect("effective removal should commit");
    assert_eq!(state.calls(), 5);
}
