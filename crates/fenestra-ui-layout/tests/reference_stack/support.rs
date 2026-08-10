#![allow(dead_code)]

use fenestra_ui_layout::prototype::{
    LayoutArithmeticOperationV1, LayoutAxisV1, LayoutDimensionV1, LayoutEngineErrorKindV1,
    LayoutErrorKindV1, LayoutErrorLocationV1, LayoutExtentV1, LayoutInputV1, LayoutLimitsV1,
    LayoutNodeKeyV1, LayoutNodeV1, LayoutPaddingV1, LayoutRecordV1, LayoutRectV1, LayoutStyleV1,
    LayoutViewportV1, REGISTERED_LAYOUT_LIMITS_V1, ReferenceStackEngineV1, compute_layout_v1,
};

pub const LIMITS: LayoutLimitsV1 = REGISTERED_LAYOUT_LIMITS_V1;

pub const fn dimension(minimum: i32, preferred: i32, maximum: i32) -> LayoutDimensionV1 {
    LayoutDimensionV1::new(minimum, preferred, maximum)
}

pub const fn fixed(value: i32) -> LayoutDimensionV1 {
    dimension(value, value, value)
}

pub const fn padding(left: i32, right: i32, top: i32, bottom: i32) -> LayoutPaddingV1 {
    LayoutPaddingV1::new(left, right, top, bottom)
}

pub const fn node(
    key: u32,
    parent: Option<u32>,
    axis: LayoutAxisV1,
    width: LayoutDimensionV1,
    height: LayoutDimensionV1,
    node_padding: LayoutPaddingV1,
    gap: i32,
) -> LayoutNodeV1 {
    LayoutNodeV1::new(
        LayoutNodeKeyV1::new(key),
        match parent {
            Some(parent) => Some(LayoutNodeKeyV1::new(parent)),
            None => None,
        },
        LayoutStyleV1::new(axis, width, height, node_padding, gap),
    )
}

pub const fn fixed_node(
    key: u32,
    parent: Option<u32>,
    axis: LayoutAxisV1,
    width: i32,
    height: i32,
) -> LayoutNodeV1 {
    node(
        key,
        parent,
        axis,
        fixed(width),
        fixed(height),
        padding(0, 0, 0, 0),
        0,
    )
}

pub const fn record(key: u32, x: i32, y: i32, width: i32, height: i32) -> LayoutRecordV1 {
    LayoutRecordV1::new(
        LayoutNodeKeyV1::new(key),
        LayoutRectV1::new(x, y, width, height),
    )
}

pub fn assert_reference_case(
    name: &str,
    viewport: LayoutViewportV1,
    nodes: &[LayoutNodeV1],
    limits: LayoutLimitsV1,
    expected: &[LayoutRecordV1],
) {
    let engine = ReferenceStackEngineV1::new();
    let input = LayoutInputV1::new(viewport, nodes);
    let input_before = input;
    let nodes_before = nodes.to_vec();

    let first = compute_layout_v1(&engine, input, limits)
        .unwrap_or_else(|error| panic!("{name}: first reference run failed: {error:?}"));
    assert_eq!(input, input_before, "{name}: first run mutated the input");
    assert_eq!(nodes, nodes_before, "{name}: first run mutated the nodes");
    assert_eq!(first.records(), expected, "{name}: unexpected first output");

    let second = compute_layout_v1(&engine, input, limits)
        .unwrap_or_else(|error| panic!("{name}: second reference run failed: {error:?}"));
    assert_eq!(input, input_before, "{name}: second run mutated the input");
    assert_eq!(nodes, nodes_before, "{name}: second run mutated the nodes");
    assert_eq!(
        second.records(),
        expected,
        "{name}: unexpected second output"
    );
    assert_eq!(
        second, first,
        "{name}: repeated runs were not deterministic"
    );
}

pub fn assert_reference_arithmetic_error(
    name: &str,
    viewport: LayoutViewportV1,
    nodes: &[LayoutNodeV1],
    operation: LayoutArithmeticOperationV1,
    extent: LayoutExtentV1,
    location: LayoutErrorLocationV1,
) {
    let engine = ReferenceStackEngineV1::new();
    let input = LayoutInputV1::new(viewport, nodes);
    let input_before = input;
    let nodes_before = nodes.to_vec();
    let expected_kind = LayoutErrorKindV1::Engine(LayoutEngineErrorKindV1::ArithmeticExhausted {
        operation,
        extent,
    });

    let first = compute_layout_v1(&engine, input, LIMITS)
        .expect_err("arithmetic exhaustion must not publish partial output");
    assert_eq!(
        first.kind(),
        expected_kind,
        "{name}: wrong first error kind"
    );
    assert_eq!(first.location(), location, "{name}: wrong first location");
    assert_eq!(input, input_before, "{name}: first run mutated the input");
    assert_eq!(nodes, nodes_before, "{name}: first run mutated the nodes");

    let second = compute_layout_v1(&engine, input, LIMITS)
        .expect_err("repeated arithmetic exhaustion must not publish output");
    assert_eq!(
        second.kind(),
        expected_kind,
        "{name}: wrong second error kind"
    );
    assert_eq!(second.location(), location, "{name}: wrong second location");
    assert_eq!(
        second, first,
        "{name}: repeated errors were not deterministic"
    );
    assert_eq!(input, input_before, "{name}: second run mutated the input");
    assert_eq!(nodes, nodes_before, "{name}: second run mutated the nodes");
}

pub const fn viewport(width: i32, height: i32) -> LayoutViewportV1 {
    LayoutViewportV1::new(width, height)
}

pub const fn input_node(index: u32) -> LayoutErrorLocationV1 {
    LayoutErrorLocationV1::InputNode { index }
}
