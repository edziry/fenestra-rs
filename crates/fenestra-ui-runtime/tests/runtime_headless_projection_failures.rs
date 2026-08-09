#[path = "support/headless_projection.rs"]
mod headless_projection;
mod support;

use fenestra_ui_ir::prototype::{InputPolicy, PropertyId, PropertyValue, TemplateNodeId};
use fenestra_ui_runtime::prototype::{
    HeadlessProjectionCapacity, HeadlessProjectionErrorKind, HeadlessProjectionLimitKind,
    HeadlessSurface, RuntimeInitializationErrorKind, TransactionErrorKind,
};

use headless_projection::{nodes, runtime, try_runtime};
use support::headless::{
    CONTROL, HEIGHT, INPUT, INSERTED_KEY, ITEM, ROOT, ROOT_WIDTH, SCHEMA_WIDTH, WIDTH, exact_style,
    exact_style_with,
};

fn capacity(
    computed: usize,
    geometry: usize,
    semantics: usize,
    hit_regions: usize,
    scene_rectangles: usize,
) -> HeadlessProjectionCapacity {
    HeadlessProjectionCapacity::new(computed, geometry, semantics, hit_regions, scene_rectangles)
}

fn initialization_error(
    assignments: Vec<(TemplateNodeId, PropertyId, PropertyValue)>,
    capacity: HeadlessProjectionCapacity,
) -> RuntimeInitializationErrorKind {
    initialization_error_at(assignments, HeadlessSurface::new(120, 90), capacity)
}

fn initialization_error_at(
    assignments: Vec<(TemplateNodeId, PropertyId, PropertyValue)>,
    surface: HeadlessSurface,
    capacity: HeadlessProjectionCapacity,
) -> RuntimeInitializationErrorKind {
    try_runtime(exact_style_with(assignments), surface, capacity)
        .err()
        .expect("invalid projection should reject initialization")
        .kind()
}

fn headless_initialization_error(
    kind: HeadlessProjectionErrorKind,
) -> RuntimeInitializationErrorKind {
    RuntimeInitializationErrorKind::Headless(kind)
}

fn headless_transaction_error(kind: HeadlessProjectionErrorKind) -> TransactionErrorKind {
    TransactionErrorKind::Headless(kind)
}

#[test]
fn negative_width_or_height_rejects_initial_projection() {
    for (template, property) in [(ROOT, WIDTH), (ITEM, HEIGHT)] {
        assert_eq!(
            initialization_error(
                vec![(template, property, PropertyValue::ScalarI32(-1))],
                capacity(5, 5, 1, 3, 5),
            ),
            headless_initialization_error(HeadlessProjectionErrorKind::NegativeGeometry)
        );
    }
}

#[test]
fn fixed_record_limits_precede_negative_geometry() {
    let negative = vec![(ITEM, WIDTH, PropertyValue::ScalarI32(-1))];
    let cases = [
        (
            capacity(4, 4, 0, 0, 0),
            HeadlessProjectionLimitKind::ComputedStyles,
        ),
        (
            capacity(5, 4, 0, 0, 0),
            HeadlessProjectionLimitKind::Geometry,
        ),
    ];

    for (capacity, expected) in cases {
        assert_eq!(
            initialization_error(negative.clone(), capacity),
            headless_initialization_error(HeadlessProjectionErrorKind::CapacityExceeded(expected))
        );
    }
}

#[test]
fn global_negative_scan_precedes_earlier_layout_overflow() {
    let mut runtime = runtime(HeadlessSurface::new(120, 90));
    let before = runtime.committed();
    let nodes = nodes(&before);
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(nodes.root, WIDTH, PropertyValue::ScalarI32(ROOT_WIDTH))
        .expect("leading no-op should stage");
    transaction
        .set_property(nodes.control, HEIGHT, PropertyValue::ScalarI32(i32::MAX))
        .expect("overflowing control height should stage");
    transaction
        .set_property(nodes.second, WIDTH, PropertyValue::ScalarI32(-1))
        .expect("later negative width should stage");

    let error = runtime
        .commit(transaction)
        .expect_err("global negative scan should precede earlier layout overflow");
    let after = runtime.committed();

    assert_eq!(
        error.kind(),
        headless_transaction_error(HeadlessProjectionErrorKind::NegativeGeometry)
    );
    assert_eq!(error.operation_index(), Some(2));
    assert!(before.shares_state_with(&after));
    assert_eq!(after.generation(), before.generation());
}

#[test]
fn negative_geometry_precedes_derived_capacity_failures() {
    for surface in [HeadlessSurface::new(120, 90), HeadlessSurface::new(0, 0)] {
        assert_eq!(
            initialization_error_at(
                vec![(ITEM, WIDTH, PropertyValue::ScalarI32(-1))],
                surface,
                capacity(5, 5, 0, 0, 0),
            ),
            headless_initialization_error(HeadlessProjectionErrorKind::NegativeGeometry)
        );
    }
}

#[test]
fn geometry_capacity_precedes_negative_and_attributes_the_insert() {
    let mut runtime = try_runtime(
        exact_style(),
        HeadlessSurface::new(120, 90),
        capacity(6, 5, 1, 4, 6),
    )
    .expect("baseline should fit before insertion");
    let before = runtime.committed();
    let nodes = nodes(&before);
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(nodes.root, WIDTH, PropertyValue::ScalarI32(ROOT_WIDTH))
        .expect("leading no-op should stage");
    transaction
        .set_property(nodes.second, WIDTH, PropertyValue::ScalarI32(-1))
        .expect("prior negative width should stage");
    transaction
        .insert_keyed(nodes.items, INSERTED_KEY, 2)
        .expect("item insertion should stage");

    let error = runtime
        .commit(transaction)
        .expect_err("insert should exceed geometry capacity");
    let after = runtime.committed();

    assert_eq!(
        error.kind(),
        headless_transaction_error(HeadlessProjectionErrorKind::CapacityExceeded(
            HeadlessProjectionLimitKind::Geometry,
        ))
    );
    assert_eq!(error.operation_index(), Some(2));
    assert!(before.shares_state_with(&after));
    assert_eq!(after.generation(), before.generation());
    assert_eq!(after.keyed_member(nodes.items, INSERTED_KEY), None);
    assert_eq!(
        after.property(nodes.second, WIDTH),
        Some(&PropertyValue::ScalarI32(SCHEMA_WIDTH))
    );
}

#[test]
fn derived_hit_capacity_failure_rolls_back_accepting_root() {
    let mut runtime = try_runtime(
        exact_style(),
        HeadlessSurface::new(120, 90),
        capacity(5, 5, 1, 3, 5),
    )
    .expect("baseline should fit exact hit capacity");
    let before = runtime.committed();
    let nodes = nodes(&before);
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(
            nodes.root,
            INPUT,
            PropertyValue::InputPolicy(InputPolicy::Accept),
        )
        .expect("accepting root should stage");

    let error = runtime
        .commit(transaction)
        .expect_err("accepting root should exceed hit capacity");
    let after = runtime.committed();

    assert_eq!(
        error.kind(),
        headless_transaction_error(HeadlessProjectionErrorKind::CapacityExceeded(
            HeadlessProjectionLimitKind::HitRegions,
        ))
    );
    assert!(before.shares_state_with(&after));
    assert_eq!(after.generation(), before.generation());
    assert_eq!(
        after.property(nodes.root, INPUT),
        Some(&PropertyValue::InputPolicy(InputPolicy::Ignore))
    );
}

#[test]
fn arithmetic_exhaustion_precedes_derived_capacity_failures() {
    assert_eq!(
        initialization_error(
            vec![(CONTROL, HEIGHT, PropertyValue::ScalarI32(i32::MAX))],
            capacity(5, 5, 0, 0, 0),
        ),
        headless_initialization_error(HeadlessProjectionErrorKind::ArithmeticExhausted)
    );
}

#[test]
fn commit_arithmetic_exhaustion_precedes_eager_hit_capacity_failure() {
    let mut runtime = try_runtime(
        exact_style(),
        HeadlessSurface::new(120, 90),
        capacity(5, 5, 1, 3, 5),
    )
    .expect("baseline should fit exact hit capacity");
    let before = runtime.committed();
    let nodes = nodes(&before);
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(
            nodes.root,
            INPUT,
            PropertyValue::InputPolicy(InputPolicy::Accept),
        )
        .expect("accepting root should stage");
    transaction
        .set_property(nodes.control, HEIGHT, PropertyValue::ScalarI32(i32::MAX))
        .expect("overflowing control height should stage");

    let error = runtime
        .commit(transaction)
        .expect_err("arithmetic should precede the derived hit limit");
    let after = runtime.committed();

    assert_eq!(
        error.kind(),
        headless_transaction_error(HeadlessProjectionErrorKind::ArithmeticExhausted)
    );
    assert!(before.shares_state_with(&after));
    assert_eq!(after.generation(), before.generation());
    assert_eq!(
        after.property(nodes.root, INPUT),
        Some(&PropertyValue::InputPolicy(InputPolicy::Ignore))
    );
    assert_eq!(
        after.property(nodes.control, HEIGHT),
        Some(&PropertyValue::ScalarI32(10))
    );
}

#[test]
fn direct_negative_dimension_rolls_back_exact_committed_allocation() {
    let mut runtime = runtime(HeadlessSurface::new(120, 90));
    let before = runtime.committed();
    let nodes = nodes(&before);
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(nodes.first, HEIGHT, PropertyValue::ScalarI32(-1))
        .expect("negative height should stage as typed fixture data");

    let error = runtime
        .commit(transaction)
        .expect_err("negative geometry should reject publication");
    let after = runtime.committed();

    assert_eq!(
        error.kind(),
        headless_transaction_error(HeadlessProjectionErrorKind::NegativeGeometry)
    );
    assert_eq!(error.operation_index(), Some(0));
    assert!(before.shares_state_with(&after));
    assert_eq!(after.generation(), before.generation());
    assert_eq!(
        after.property(nodes.first, HEIGHT),
        Some(&PropertyValue::ScalarI32(12))
    );
}

#[test]
fn derived_arithmetic_failure_rolls_back_exact_committed_allocation() {
    let mut runtime = runtime(HeadlessSurface::new(120, 90));
    let before = runtime.committed();
    let nodes = nodes(&before);
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(nodes.control, HEIGHT, PropertyValue::ScalarI32(i32::MAX))
        .expect("large height should stage as typed fixture data");

    let error = runtime
        .commit(transaction)
        .expect_err("cursor arithmetic should reject publication");
    let after = runtime.committed();

    assert_eq!(
        error.kind(),
        headless_transaction_error(HeadlessProjectionErrorKind::ArithmeticExhausted)
    );
    assert!(before.shares_state_with(&after));
    assert_eq!(after.generation(), before.generation());
    assert_eq!(
        after.property(nodes.control, HEIGHT),
        Some(&PropertyValue::ScalarI32(10))
    );
}
