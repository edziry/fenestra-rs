#[path = "runtime_layout_engine/calls.rs"]
mod calls;
#[path = "runtime_layout_engine/failures.rs"]
mod failures;
#[path = "support/headless_projection.rs"]
mod headless_projection;
#[path = "runtime_layout_engine/support.rs"]
mod layout_support;
#[path = "runtime_layout_engine/limits.rs"]
mod limits;
#[path = "runtime_layout_engine/output.rs"]
mod output;
mod support;

use std::panic::{RefUnwindSafe, UnwindSafe};

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_layout::prototype::LayoutViewportV1;
use fenestra_ui_runtime::prototype::UiRuntime;

use layout_support::{dimension, initialize_with_spy, node, only_input};
use support::headless::{
    CONTAINER, CONTROL, ROOT, WIDTH, exact_style, exact_style_with, runtime_capacity,
};
use support::headless_spec::{HeadlessSpecBuilder, surface};

fn assert_runtime_auto_traits<T>()
where
    T: Send + Sync + Unpin + UnwindSafe + RefUnwindSafe,
{
}

#[test]
fn injected_layout_engine_preserves_runtime_auto_traits() {
    assert_runtime_auto_traits::<UiRuntime>();
}

#[test]
fn initialization_invokes_injected_engine_once_with_exact_mapped_input() {
    let (_runtime, state) = initialize_with_spy(exact_style());
    let captured = only_input(&state);

    assert_eq!(captured.viewport, LayoutViewportV1::new(120, 90));
    assert_eq!(
        captured.nodes,
        vec![
            node(0, None, dimension(100, 100), dimension(80, 80)),
            node(1, Some(0), dimension(80, 100), dimension(50, 50)),
            node(2, Some(1), dimension(30, 80), dimension(10, 10)),
            node(3, Some(1), dimension(40, 80), dimension(12, 12)),
            node(4, Some(1), dimension(40, 80), dimension(12, 12)),
        ]
    );
}

#[test]
fn child_width_maximum_uses_the_resolved_parent_width() {
    let style = exact_style_with(vec![
        (ROOT, WIDTH, PropertyValue::ScalarI32(50)),
        (CONTAINER, WIDTH, PropertyValue::ScalarI32(80)),
        (CONTROL, WIDTH, PropertyValue::ScalarI32(70)),
    ]);
    let (_runtime, state) = initialize_with_spy(style);
    let captured = only_input(&state);

    assert_eq!(captured.nodes.len(), 5);
    assert_eq!(captured.nodes[0].style().width(), dimension(50, 50));
    assert_eq!(captured.nodes[1].style().width(), dimension(80, 50));
    assert_eq!(captured.nodes[2].style().width(), dimension(70, 50));
}

#[test]
fn default_headless_constructor_remains_the_baseline_control() {
    let runtime = UiRuntime::new_headless(
        exact_style(),
        HeadlessSpecBuilder::new().build(),
        surface(),
        runtime_capacity(),
    )
    .expect("default headless runtime should initialize");

    assert_eq!(
        runtime
            .committed()
            .headless_projection()
            .expect("headless projection should exist")
            .geometry_count(),
        5
    );
}
