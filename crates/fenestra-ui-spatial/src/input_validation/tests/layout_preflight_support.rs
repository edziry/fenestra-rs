use std::error::Error;

use fenestra_ui_layout::prototype::{LayoutAxisV1, LayoutDimensionV1, LayoutPaddingV1};

use super::fixture::RawInputFixture;
use super::island_support::{fixture, free_with, island_limits, layout_with, root_with};
use crate::error::{
    SpatialContainerErrorKindV2, SpatialErrorLocationV2, SpatialInputErrorKindV2,
    SpatialLayoutDimensionErrorKindV2,
};
use crate::limits::SpatialLimitsV2;
use crate::model::SpatialViewportV2;
use crate::resolve_error::{
    SpatialLayoutErrorKindV2, SpatialResolveErrorKindV2, SpatialResolveErrorV2,
};
use crate::topology::{SpatialContainerV2, SpatialNodeV2};

pub(super) const VIEWPORT: SpatialViewportV2 = SpatialViewportV2::new(20, 20);

pub(super) fn input(nodes: Vec<SpatialNodeV2>) -> RawInputFixture {
    fixture(nodes)
}

pub(super) fn root(container: SpatialContainerV2) -> SpatialNodeV2 {
    root_with(container)
}

pub(super) fn free(
    key: u32,
    parent: u32,
    width: i32,
    height: i32,
    container: SpatialContainerV2,
) -> SpatialNodeV2 {
    free_with(key, parent, width, height, container)
}

pub(super) fn layout(
    key: u32,
    parent: u32,
    width: LayoutDimensionV1,
    height: LayoutDimensionV1,
    container: SpatialContainerV2,
) -> SpatialNodeV2 {
    layout_with(key, parent, width, height, container)
}

pub(super) const fn fixed(value: i32) -> LayoutDimensionV1 {
    LayoutDimensionV1::new(value, value, value)
}

pub(super) const fn dimension(minimum: i32, preferred: i32, maximum: i32) -> LayoutDimensionV1 {
    LayoutDimensionV1::new(minimum, preferred, maximum)
}

pub(super) const fn padding(left: i32, right: i32, top: i32, bottom: i32) -> LayoutPaddingV1 {
    LayoutPaddingV1::new(left, right, top, bottom)
}

pub(super) const fn container(padding: LayoutPaddingV1, gap: i32) -> SpatialContainerV2 {
    container_on(LayoutAxisV1::Column, padding, gap)
}

pub(super) const fn container_on(
    axis: LayoutAxisV1,
    padding: LayoutPaddingV1,
    gap: i32,
) -> SpatialContainerV2 {
    SpatialContainerV2::new(axis, padding, gap)
}

pub(super) const fn valid_container() -> SpatialContainerV2 {
    container(padding(0, 0, 0, 0), 0)
}

pub(super) fn limits(islands: usize, per_island: usize, total: usize) -> SpatialLimitsV2 {
    island_limits(islands, per_island, total)
}

pub(super) fn preflight(
    fixture: &RawInputFixture,
    viewport: SpatialViewportV2,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    prepare_layout_preflight!(fixture, viewport, limits).map(|_| ())
}

pub(super) fn expect_valid(result: Result<(), SpatialResolveErrorV2>) {
    if let Err(error) = result {
        panic!("expected layout preflight success, got {error:?}");
    }
}

pub(super) fn expect_input<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialInputErrorKindV2,
    location: SpatialErrorLocationV2,
) {
    let error = match result {
        Ok(_) => panic!("expected layout preflight input failure"),
        Err(error) => error,
    };
    assert_error(
        error,
        SpatialResolveErrorKindV2::Input(kind),
        location,
        "input",
    );
}

pub(super) fn expect_bridge(error: SpatialResolveErrorV2, location: SpatialErrorLocationV2) {
    assert_error(
        error,
        SpatialResolveErrorKindV2::Layout(SpatialLayoutErrorKindV2::BridgeInvariant),
        location,
        "layout",
    );
}

fn assert_error(
    error: SpatialResolveErrorV2,
    kind: SpatialResolveErrorKindV2,
    location: SpatialErrorLocationV2,
    label: &str,
) {
    assert_eq!(error.kind(), kind);
    assert_eq!(error.location(), location);
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
    assert_eq!(error.to_string(), format!("spatial-resolve-error({label})"));
    assert_eq!(
        format!("{error:?}"),
        format!("SpatialResolveErrorV2(spatial-resolve-error({label}))")
    );
    assert!(Error::source(&error).is_none());
}

pub(super) const fn negative_constraint(
    extent: fenestra_ui_layout::prototype::LayoutExtentV1,
    field: fenestra_ui_layout::prototype::LayoutConstraintFieldV1,
) -> SpatialInputErrorKindV2 {
    SpatialInputErrorKindV2::InvalidLayoutDimensions(
        SpatialLayoutDimensionErrorKindV2::NegativeConstraint { extent, field },
    )
}

pub(super) const fn invalid_container(
    kind: SpatialContainerErrorKindV2,
) -> SpatialInputErrorKindV2 {
    SpatialInputErrorKindV2::InvalidContainer(kind)
}
