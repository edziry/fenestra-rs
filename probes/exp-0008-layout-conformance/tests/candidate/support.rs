use fenestra_ui_layout::prototype::{
    LayoutAxisV1, LayoutDimensionV1, LayoutEngineErrorKindV1, LayoutErrorKindV1,
    LayoutErrorLocationV1, LayoutInputV1, LayoutNodeKeyV1, LayoutNodeV1, LayoutPaddingV1,
    LayoutStyleV1, LayoutViewportV1, REGISTERED_LAYOUT_LIMITS_V1, ReferenceStackEngineV1,
    compute_layout_v1,
};

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

pub const fn viewport(width: i32, height: i32) -> LayoutViewportV1 {
    LayoutViewportV1::new(width, height)
}

pub fn assert_candidate_rejection(
    name: &str,
    engine: &impl fenestra_ui_layout::prototype::LayoutEngineV1,
    viewport: LayoutViewportV1,
    nodes: &[LayoutNodeV1],
    location: LayoutErrorLocationV1,
) {
    let input = LayoutInputV1::new(viewport, nodes);
    compute_layout_v1(
        &ReferenceStackEngineV1::new(),
        input,
        REGISTERED_LAYOUT_LIMITS_V1,
    )
    .unwrap_or_else(|error| panic!("{name}: input must remain core-valid: {error:?}"));

    let error = compute_layout_v1(engine, input, REGISTERED_LAYOUT_LIMITS_V1)
        .expect_err("core-valid value above the candidate profile must be rejected");
    assert_eq!(
        error.kind(),
        LayoutErrorKindV1::Engine(LayoutEngineErrorKindV1::RejectedInput),
        "{name}: wrong neutral rejection"
    );
    assert_eq!(
        error.location(),
        location,
        "{name}: wrong rejection location"
    );
}
