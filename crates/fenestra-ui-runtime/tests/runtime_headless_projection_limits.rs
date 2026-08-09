#[path = "support/headless_projection.rs"]
mod headless_projection;
mod support;

use fenestra_ui_runtime::prototype::{
    HeadlessProjectionCapacity, HeadlessProjectionErrorKind, HeadlessProjectionLimitKind,
    HeadlessSurface, RuntimeInitializationErrorKind,
};

use headless_projection::{BASELINE_PROJECTION_CAPACITY, try_runtime};
use support::headless::exact_style;

fn capacity(
    computed: usize,
    geometry: usize,
    semantics: usize,
    hit_regions: usize,
    scene_rectangles: usize,
) -> HeadlessProjectionCapacity {
    HeadlessProjectionCapacity::new(computed, geometry, semantics, hit_regions, scene_rectangles)
}

fn initialization_error(capacity: HeadlessProjectionCapacity) -> RuntimeInitializationErrorKind {
    try_runtime(exact_style(), HeadlessSurface::new(120, 90), capacity)
        .err()
        .expect("projection capacity should reject initialization")
        .kind()
}

fn limit_error(limit: HeadlessProjectionLimitKind) -> RuntimeInitializationErrorKind {
    RuntimeInitializationErrorKind::Headless(HeadlessProjectionErrorKind::CapacityExceeded(limit))
}

#[test]
fn baseline_projection_capacities_are_inclusive() {
    let runtime = try_runtime(
        exact_style(),
        HeadlessSurface::new(120, 90),
        BASELINE_PROJECTION_CAPACITY,
    )
    .expect("all exact projection ceilings should initialize");
    let committed = runtime.committed();
    let projection = committed
        .headless_projection()
        .expect("headless projection should exist");

    assert_eq!(projection.computed_style_count(), 5);
    assert_eq!(projection.geometry_count(), 5);
    assert_eq!(projection.semantic_count(), 1);
    assert_eq!(projection.hit_region_count(), 3);
    assert_eq!(projection.scene_rectangle_count(), 5);
}

#[test]
fn every_projection_capacity_rejects_one_record_below_baseline() {
    let cases = [
        (
            capacity(4, 5, 1, 3, 5),
            HeadlessProjectionLimitKind::ComputedStyles,
        ),
        (
            capacity(5, 4, 1, 3, 5),
            HeadlessProjectionLimitKind::Geometry,
        ),
        (
            capacity(5, 5, 0, 3, 5),
            HeadlessProjectionLimitKind::Semantics,
        ),
        (
            capacity(5, 5, 1, 2, 5),
            HeadlessProjectionLimitKind::HitRegions,
        ),
        (
            capacity(5, 5, 1, 3, 4),
            HeadlessProjectionLimitKind::SceneRectangles,
        ),
    ];

    for (capacity, expected) in cases {
        assert_eq!(initialization_error(capacity), limit_error(expected));
    }
}

#[test]
fn simultaneous_projection_limits_follow_applicable_order() {
    let cases = [
        (
            capacity(4, 4, 0, 2, 4),
            HeadlessProjectionLimitKind::ComputedStyles,
        ),
        (
            capacity(5, 4, 0, 2, 4),
            HeadlessProjectionLimitKind::Geometry,
        ),
        (
            capacity(5, 5, 0, 2, 4),
            HeadlessProjectionLimitKind::Semantics,
        ),
        (
            capacity(5, 5, 1, 2, 4),
            HeadlessProjectionLimitKind::HitRegions,
        ),
        (
            capacity(5, 5, 1, 3, 4),
            HeadlessProjectionLimitKind::SceneRectangles,
        ),
    ];

    for (capacity, expected) in cases {
        assert_eq!(initialization_error(capacity), limit_error(expected));
    }
}

#[test]
fn empty_surface_accepts_zero_derived_projection_capacities() {
    let runtime = try_runtime(
        exact_style(),
        HeadlessSurface::new(0, 0),
        capacity(5, 5, 0, 0, 0),
    )
    .expect("empty clips should produce no derived records");
    let committed = runtime.committed();
    let projection = committed
        .headless_projection()
        .expect("headless projection should exist");

    assert_eq!(projection.computed_style_count(), 5);
    assert_eq!(projection.geometry_count(), 5);
    assert_eq!(projection.semantic_count(), 0);
    assert_eq!(projection.hit_region_count(), 0);
    assert_eq!(projection.scene_rectangle_count(), 0);
}
