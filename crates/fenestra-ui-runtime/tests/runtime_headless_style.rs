mod support;

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{
    HeadlessProjectionCapacity, HeadlessProjectionSpec, HeadlessSurface, UiRuntime,
};

use support::headless::{
    COLOR, CONTAINER, CONTROL_STYLE_COLOR, DIRECT_COLOR, FIRST_KEY, HEIGHT, INPUT, INSERTED_KEY,
    ITEM_STYLE_COLOR, ITEMS, ROOT_WIDTH, SEMANTIC_LABEL, SURFACE_HEIGHT, SURFACE_WIDTH, VISIBLE,
    WIDTH, construction, empty_style, exact_style, runtime_capacity,
};

fn projection_spec() -> HeadlessProjectionSpec {
    HeadlessProjectionSpec::new(
        WIDTH,
        HEIGHT,
        COLOR,
        VISIBLE,
        INPUT,
        support::headless::CONTROL,
        SEMANTIC_LABEL,
        HeadlessProjectionCapacity::new(8, 8, 1, 8, 8),
    )
}

fn headless_runtime(style: fenestra_ui_ir::prototype::ValidatedStyleProgram) -> UiRuntime {
    UiRuntime::new_headless(
        style,
        projection_spec(),
        HeadlessSurface::new(SURFACE_WIDTH, SURFACE_HEIGHT),
        runtime_capacity(),
    )
    .expect("headless runtime should initialize")
}

#[test]
fn construction_initial_is_preserved_without_an_exact_style_assignment() {
    let runtime = headless_runtime(empty_style());
    let committed = runtime.committed();
    let root = committed.root();
    let projection = committed
        .headless_projection()
        .expect("headless runtime should expose its projection");
    let computed = projection
        .computed_style(root)
        .expect("root computed style should exist");

    assert_eq!(
        committed.property(root, WIDTH),
        Some(&PropertyValue::ScalarI32(ROOT_WIDTH))
    );
    assert_eq!(
        computed.property(WIDTH),
        Some(&PropertyValue::ScalarI32(ROOT_WIDTH))
    );
}

#[test]
fn exact_control_and_item_assignments_initialize_computed_style() {
    let runtime = headless_runtime(exact_style());
    let committed = runtime.committed();
    let root = committed.root();
    let container = committed.children(root).expect("root should be live")[0];
    let control = committed
        .children(container)
        .expect("container should be live")[0];
    let items = committed
        .fragment(container, ITEMS)
        .expect("item region should exist");
    let item = committed
        .keyed_member(items, FIRST_KEY)
        .expect("first item should exist");
    let projection = committed
        .headless_projection()
        .expect("headless runtime should expose its projection");

    assert_eq!(committed.template(container), Some(CONTAINER));
    assert_eq!(
        committed.property(control, COLOR),
        Some(&PropertyValue::Rgba8(CONTROL_STYLE_COLOR))
    );
    assert_eq!(
        committed.property(item, COLOR),
        Some(&PropertyValue::Rgba8(ITEM_STYLE_COLOR))
    );
    assert_eq!(
        projection
            .computed_style(control)
            .expect("control computed style should exist")
            .property(COLOR),
        Some(&PropertyValue::Rgba8(CONTROL_STYLE_COLOR))
    );
    assert_eq!(
        projection
            .computed_style(item)
            .expect("item computed style should exist")
            .property(COLOR),
        Some(&PropertyValue::Rgba8(ITEM_STYLE_COLOR))
    );
}

#[test]
fn direct_mutation_replaces_the_materialized_style_value() {
    let mut runtime = headless_runtime(exact_style());
    let before = runtime.committed();
    let container = before.children(before.root()).expect("root should be live")[0];
    let control = before
        .children(container)
        .expect("container should be live")[0];
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(control, COLOR, PropertyValue::Rgba8(DIRECT_COLOR))
        .expect("direct color should stage");

    runtime
        .commit(transaction)
        .expect("direct color should publish");
    let committed = runtime.committed();
    let projection = committed
        .headless_projection()
        .expect("headless runtime should expose its projection");

    assert_eq!(
        committed.property(control, COLOR),
        Some(&PropertyValue::Rgba8(DIRECT_COLOR))
    );
    assert_eq!(
        projection
            .computed_style(control)
            .expect("control computed style should exist")
            .property(COLOR),
        Some(&PropertyValue::Rgba8(DIRECT_COLOR))
    );
}

#[test]
fn keyed_insert_materializes_the_repeat_body_style() {
    let mut runtime = headless_runtime(exact_style());
    let before = runtime.committed();
    let container = before.children(before.root()).expect("root should be live")[0];
    let items = before
        .fragment(container, ITEMS)
        .expect("item region should exist");
    let mut transaction = runtime.begin_transaction();
    transaction
        .insert_keyed(items, INSERTED_KEY, 1)
        .expect("item insert should stage");

    runtime
        .commit(transaction)
        .expect("item insert should publish");
    let committed = runtime.committed();
    let inserted = committed
        .keyed_member(items, INSERTED_KEY)
        .expect("inserted item should exist");
    let projection = committed
        .headless_projection()
        .expect("headless runtime should expose its projection");

    assert_eq!(
        committed.property(inserted, COLOR),
        Some(&PropertyValue::Rgba8(ITEM_STYLE_COLOR))
    );
    assert_eq!(
        projection
            .computed_style(inserted)
            .expect("inserted item computed style should exist")
            .property(COLOR),
        Some(&PropertyValue::Rgba8(ITEM_STYLE_COLOR))
    );
}

#[test]
fn ordinary_runtime_snapshot_has_no_headless_projection() {
    let runtime = UiRuntime::new(construction(), runtime_capacity())
        .expect("ordinary runtime should initialize");

    assert!(runtime.committed().headless_projection().is_none());
}
