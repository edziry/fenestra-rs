mod support;

use fenestra_ui_ir::prototype::{PropertyValue, ValidatedStyleProgram};
use fenestra_ui_runtime::prototype::{
    HeadlessProjectionErrorKind, HeadlessProjectionLimitKind, HeadlessProjectionSpec,
    HeadlessSurface, RuntimeInitializationErrorKind, UiRuntime,
};

use support::headless::{
    COLOR, HEIGHT, INPUT, ITEM, ROOT, VISIBLE, WIDTH, empty_style, runtime_capacity,
};
use support::headless_spec::{
    HeadlessSpecBuilder, HeterogeneousHeadlessBuilder, MISSING_PROPERTY, MISSING_TEMPLATE, surface,
};

fn initialization_error(
    style: ValidatedStyleProgram,
    spec: HeadlessProjectionSpec,
    surface: HeadlessSurface,
) -> RuntimeInitializationErrorKind {
    UiRuntime::new_headless(style, spec, surface, runtime_capacity())
        .err()
        .expect("headless initialization should fail")
        .kind()
}

fn assert_headless_error(
    style: ValidatedStyleProgram,
    spec: HeadlessProjectionSpec,
    initial_surface: HeadlessSurface,
    expected: HeadlessProjectionErrorKind,
) {
    assert_eq!(
        initialization_error(style, spec, initial_surface),
        RuntimeInitializationErrorKind::Headless(expected)
    );
}

#[test]
fn projection_limit_kinds_have_fixed_diagnostic_order() {
    assert_eq!(
        HeadlessProjectionLimitKind::ALL,
        [
            HeadlessProjectionLimitKind::ComputedStyles,
            HeadlessProjectionLimitKind::Geometry,
            HeadlessProjectionLimitKind::Semantics,
            HeadlessProjectionLimitKind::HitRegions,
            HeadlessProjectionLimitKind::SceneRectangles,
        ]
    );
}

#[test]
fn valid_spec_accepts_a_heterogeneous_static_descendant() {
    let style = HeterogeneousHeadlessBuilder::new().empty_style();
    let runtime = UiRuntime::new_headless(
        style,
        HeadlessSpecBuilder::new().build(),
        surface(),
        runtime_capacity(),
    )
    .expect("heterogeneous role declarations should initialize");
    let committed = runtime.committed();

    assert_eq!(committed.node_count(), 2);
    assert_eq!(
        committed
            .headless_projection()
            .expect("headless projection should exist")
            .computed_style_count(),
        2
    );
}

#[test]
fn missing_roles_or_targets_use_one_closed_error() {
    let cases = [
        (
            empty_style(),
            HeadlessSpecBuilder::new()
                .with_width(MISSING_PROPERTY)
                .build(),
        ),
        (
            empty_style(),
            HeadlessSpecBuilder::new()
                .with_semantic_template(MISSING_TEMPLATE)
                .build(),
        ),
        (
            HeterogeneousHeadlessBuilder::new()
                .without_target_property(INPUT)
                .empty_style(),
            HeadlessSpecBuilder::new().build(),
        ),
    ];

    for (style, spec) in cases {
        assert_headless_error(
            style,
            spec,
            surface(),
            HeadlessProjectionErrorKind::MissingSpecificationTargetOrProperty,
        );
    }
}

#[test]
fn role_types_are_checked_on_every_reachable_template() {
    let wrong_fixed_roles = [
        HeadlessSpecBuilder::new().with_width(COLOR).build(),
        HeadlessSpecBuilder::new().with_height(VISIBLE).build(),
        HeadlessSpecBuilder::new().with_color(WIDTH).build(),
        HeadlessSpecBuilder::new().with_visible(INPUT).build(),
        HeadlessSpecBuilder::new().with_input(HEIGHT).build(),
    ];
    for spec in wrong_fixed_roles {
        assert_headless_error(
            empty_style(),
            spec,
            surface(),
            HeadlessProjectionErrorKind::PropertyTypeMismatch,
        );
    }

    assert_headless_error(
        HeterogeneousHeadlessBuilder::new()
            .with_target_default(WIDTH, PropertyValue::Bool(true))
            .empty_style(),
        HeadlessSpecBuilder::new().build(),
        surface(),
        HeadlessProjectionErrorKind::PropertyTypeMismatch,
    );
}

#[test]
fn semantic_template_rejects_an_uninstantiated_region_body() {
    let style = HeterogeneousHeadlessBuilder::new()
        .target_under_empty_region()
        .empty_style();

    assert_headless_error(
        style,
        HeadlessSpecBuilder::new().build(),
        surface(),
        HeadlessProjectionErrorKind::InvalidSemanticTemplate,
    );
}

#[test]
fn zero_key_region_body_still_participates_in_role_validation() {
    let style = HeterogeneousHeadlessBuilder::new()
        .without_target_property(INPUT)
        .target_under_empty_region()
        .empty_style();

    assert_headless_error(
        style,
        HeadlessSpecBuilder::new()
            .with_semantic_template(ROOT)
            .build(),
        surface(),
        HeadlessProjectionErrorKind::MissingSpecificationTargetOrProperty,
    );
}

#[test]
fn surface_extents_are_non_negative() {
    for invalid in [
        HeadlessSurface::new(-1, surface().height()),
        HeadlessSurface::new(surface().width(), -1),
    ] {
        assert_headless_error(
            empty_style(),
            HeadlessSpecBuilder::new().build(),
            invalid,
            HeadlessProjectionErrorKind::InvalidSurface,
        );
    }

    UiRuntime::new_headless(
        empty_style(),
        HeadlessSpecBuilder::new().build(),
        HeadlessSurface::new(0, 0),
        runtime_capacity(),
    )
    .expect("zero surface extents should remain valid");
}

#[test]
fn specification_categories_have_global_priority() {
    let cap = 4;
    let invalid_surface = HeadlessSurface::new(-1, surface().height());
    let cases = [
        (
            HeadlessSpecBuilder::new()
                .with_width(COLOR)
                .with_input(MISSING_PROPERTY)
                .with_semantic_template(ITEM)
                .with_computed_capacity(cap)
                .build(),
            invalid_surface,
            HeadlessProjectionErrorKind::MissingSpecificationTargetOrProperty,
        ),
        (
            HeadlessSpecBuilder::new()
                .with_width(COLOR)
                .with_semantic_template(ITEM)
                .with_computed_capacity(cap)
                .build(),
            invalid_surface,
            HeadlessProjectionErrorKind::PropertyTypeMismatch,
        ),
        (
            HeadlessSpecBuilder::new()
                .with_semantic_template(ITEM)
                .with_computed_capacity(cap)
                .build(),
            invalid_surface,
            HeadlessProjectionErrorKind::InvalidSemanticTemplate,
        ),
        (
            HeadlessSpecBuilder::new()
                .with_computed_capacity(cap)
                .build(),
            invalid_surface,
            HeadlessProjectionErrorKind::InvalidSurface,
        ),
        (
            HeadlessSpecBuilder::new()
                .with_computed_capacity(cap)
                .build(),
            surface(),
            HeadlessProjectionErrorKind::CapacityExceeded(
                HeadlessProjectionLimitKind::ComputedStyles,
            ),
        ),
    ];

    for (spec, initial_surface, expected) in cases {
        assert_headless_error(empty_style(), spec, initial_surface, expected);
    }
}

#[test]
fn computed_style_capacity_is_inclusive_at_initialization() {
    let runtime = UiRuntime::new_headless(
        empty_style(),
        HeadlessSpecBuilder::new().with_computed_capacity(5).build(),
        surface(),
        runtime_capacity(),
    )
    .expect("the exact computed-style footprint should fit");
    assert_eq!(
        runtime
            .committed()
            .headless_projection()
            .expect("headless projection should exist")
            .computed_style_count(),
        5
    );

    assert_headless_error(
        empty_style(),
        HeadlessSpecBuilder::new().with_computed_capacity(4).build(),
        surface(),
        HeadlessProjectionErrorKind::CapacityExceeded(HeadlessProjectionLimitKind::ComputedStyles),
    );
}
