#![allow(dead_code)]

use fenestra_ui_ir::prototype::ValidatedStyleProgram;
use fenestra_ui_runtime::prototype::{
    CommittedRuntimeSnapshot, FragmentId, HeadlessProjectionCapacity, HeadlessRect,
    HeadlessSurface, NodeId, RuntimeInitializationError, UiRuntime,
};

use crate::support::headless::{FIRST_KEY, ITEMS, SECOND_KEY, exact_style, runtime_capacity};
use crate::support::headless_spec::HeadlessSpecBuilder;

pub const ROOT_COLOR: [u8; 4] = [1, 1, 1, 255];
pub const CONTAINER_COLOR: [u8; 4] = [2, 2, 2, 255];
pub const BASELINE_PROJECTION_CAPACITY: HeadlessProjectionCapacity =
    HeadlessProjectionCapacity::new(5, 5, 1, 3, 5);

#[derive(Clone, Copy)]
pub struct ProjectionNodes {
    pub root: NodeId,
    pub container: NodeId,
    pub control: NodeId,
    pub first: NodeId,
    pub second: NodeId,
    pub items: FragmentId,
}

pub fn runtime(surface: HeadlessSurface) -> UiRuntime {
    UiRuntime::new_headless(
        exact_style(),
        HeadlessSpecBuilder::new().build(),
        surface,
        runtime_capacity(),
    )
    .expect("headless projection fixture should initialize")
}

pub fn try_runtime(
    style: ValidatedStyleProgram,
    surface: HeadlessSurface,
    capacity: HeadlessProjectionCapacity,
) -> Result<UiRuntime, RuntimeInitializationError> {
    UiRuntime::new_headless(
        style,
        HeadlessSpecBuilder::new().with_capacity(capacity).build(),
        surface,
        runtime_capacity(),
    )
}

pub fn nodes(committed: &CommittedRuntimeSnapshot) -> ProjectionNodes {
    let root = committed.root();
    let container = committed.children(root).expect("root should be live")[0];
    let control = committed
        .children(container)
        .expect("container should be live")[0];
    let items = committed
        .fragment(container, ITEMS)
        .expect("item region should exist");
    let first = committed
        .keyed_member(items, FIRST_KEY)
        .expect("first item should exist");
    let second = committed
        .keyed_member(items, SECOND_KEY)
        .expect("second item should exist");
    ProjectionNodes {
        root,
        container,
        control,
        first,
        second,
        items,
    }
}

pub const fn rect(x: i32, y: i32, width: i32, height: i32) -> HeadlessRect {
    HeadlessRect::new(x, y, width, height)
}
