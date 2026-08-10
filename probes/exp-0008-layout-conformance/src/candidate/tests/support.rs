use fenestra_ui_layout::prototype::{
    LayoutAxisV1, LayoutDimensionV1, LayoutErrorLocationV1, LayoutNodeKeyV1, LayoutNodeV1,
    LayoutPaddingV1, LayoutRecordV1, LayoutStyleV1, LayoutViewportV1,
};

use crate::candidate::{
    CandidateProfileErrorFieldV1, CandidateProfileErrorKindV1, CandidateProfileErrorV1,
    CandidateRawRecordV1,
};

pub(super) const fn dimension(minimum: i32, preferred: i32, maximum: i32) -> LayoutDimensionV1 {
    LayoutDimensionV1::new(minimum, preferred, maximum)
}

pub(super) const fn fixed(value: i32) -> LayoutDimensionV1 {
    dimension(value, value, value)
}

pub(super) const fn padding(left: i32, right: i32, top: i32, bottom: i32) -> LayoutPaddingV1 {
    LayoutPaddingV1::new(left, right, top, bottom)
}

pub(super) const fn node(
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

pub(super) const fn fixed_node(
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

pub(super) const fn viewport(width: i32, height: i32) -> LayoutViewportV1 {
    LayoutViewportV1::new(width, height)
}

pub(super) const fn raw(key: u32, x: f32, y: f32, width: f32, height: f32) -> CandidateRawRecordV1 {
    CandidateRawRecordV1::new(LayoutNodeKeyV1::new(key), x, y, width, height)
}

pub(super) fn assert_profile_error<T>(
    result: Result<T, CandidateProfileErrorV1>,
    kind: CandidateProfileErrorKindV1,
    field: CandidateProfileErrorFieldV1,
    location: LayoutErrorLocationV1,
) {
    let error = result
        .err()
        .expect("candidate profile validation must fail");
    assert_eq!(error.kind(), kind);
    assert_eq!(error.field(), field);
    assert_eq!(error.location(), location);
}

pub(super) fn bounds(record: LayoutRecordV1) -> [i32; 4] {
    let bounds = record.bounds();
    [bounds.x(), bounds.y(), bounds.width(), bounds.height()]
}
