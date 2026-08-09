#[path = "support/headless_projection.rs"]
mod headless_projection;
mod support;

use fenestra_ui_ir::prototype::{InputPolicy, PropertyValue};
use fenestra_ui_runtime::prototype::{HeadlessPoint, HeadlessSurface};

use headless_projection::{nodes, rect, runtime};
use support::headless::{HEIGHT, INPUT, VISIBLE};

#[test]
fn hidden_ancestor_filters_outputs_without_removing_geometry() {
    let mut runtime = runtime(HeadlessSurface::new(120, 90));
    let before = runtime.committed();
    let nodes = nodes(&before);
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(nodes.container, VISIBLE, PropertyValue::Bool(false))
        .expect("container visibility should stage");
    runtime
        .commit(transaction)
        .expect("container visibility should publish");

    let committed = runtime.committed();
    let projection = committed
        .headless_projection()
        .expect("headless projection should exist");
    assert_eq!(projection.geometry_count(), 5);
    assert_eq!(
        projection
            .geometry(nodes.control)
            .map(|geometry| (geometry.bounds(), geometry.clip())),
        Some((rect(0, 0, 30, 10), rect(0, 0, 30, 10)))
    );
    assert_eq!(
        projection
            .geometry(nodes.first)
            .map(|geometry| (geometry.bounds(), geometry.clip())),
        Some((rect(0, 10, 40, 12), rect(0, 10, 40, 12)))
    );
    assert_eq!(
        projection
            .geometry(nodes.second)
            .map(|geometry| (geometry.bounds(), geometry.clip())),
        Some((rect(0, 22, 40, 12), rect(0, 22, 40, 12)))
    );
    assert_eq!(projection.semantic_count(), 0);
    assert_eq!(projection.hit_region_count(), 0);
    assert_eq!(
        projection
            .scene_rectangles()
            .map(|scene| scene.node())
            .collect::<Vec<_>>(),
        vec![nodes.root]
    );
}

#[test]
fn zero_height_geometry_remains_and_does_not_advance_the_cursor() {
    let mut runtime = runtime(HeadlessSurface::new(120, 90));
    let before = runtime.committed();
    let nodes = nodes(&before);
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(nodes.first, HEIGHT, PropertyValue::ScalarI32(0))
        .expect("zero height should stage");
    runtime
        .commit(transaction)
        .expect("zero height should publish");

    let committed = runtime.committed();
    let projection = committed
        .headless_projection()
        .expect("headless projection should exist");
    let first = projection
        .geometry(nodes.first)
        .expect("first item geometry should remain");
    let second = projection
        .geometry(nodes.second)
        .expect("second item geometry should remain");

    assert_eq!(first.bounds(), rect(0, 10, 40, 0));
    assert_eq!(first.clip(), rect(0, 10, 40, 0));
    assert_eq!(second.bounds(), rect(0, 10, 40, 12));
    assert_eq!(
        projection
            .scene_rectangles()
            .map(|scene| scene.node())
            .collect::<Vec<_>>(),
        vec![nodes.root, nodes.container, nodes.control, nodes.second,]
    );
    assert_eq!(
        projection
            .hit_regions()
            .map(|hit| hit.node())
            .collect::<Vec<_>>(),
        vec![nodes.control, nodes.second]
    );
}

#[test]
fn hit_testing_is_reverse_ordered_and_half_open() {
    let mut runtime = runtime(HeadlessSurface::new(120, 90));
    let before = runtime.committed();
    let nodes = nodes(&before);
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(
            nodes.root,
            INPUT,
            PropertyValue::InputPolicy(InputPolicy::Accept),
        )
        .expect("root input policy should stage");
    transaction
        .set_property(
            nodes.container,
            INPUT,
            PropertyValue::InputPolicy(InputPolicy::Accept),
        )
        .expect("container input policy should stage");
    runtime
        .commit(transaction)
        .expect("input policy changes should publish");

    let committed = runtime.committed();
    let projection = committed
        .headless_projection()
        .expect("headless projection should exist");
    assert_eq!(
        projection
            .hit_regions()
            .map(|hit| hit.node())
            .collect::<Vec<_>>(),
        vec![
            nodes.root,
            nodes.container,
            nodes.control,
            nodes.first,
            nodes.second,
        ]
    );
    assert_eq!(
        projection.hit_test(HeadlessPoint::new(0, 0)),
        Some(nodes.control)
    );
    assert_eq!(
        projection.hit_test(HeadlessPoint::new(5, 5)),
        Some(nodes.control)
    );
    assert_eq!(
        projection.hit_test(HeadlessPoint::new(35, 5)),
        Some(nodes.container)
    );
    assert_eq!(
        projection.hit_test(HeadlessPoint::new(90, 5)),
        Some(nodes.root)
    );
    assert_eq!(projection.hit_test(HeadlessPoint::new(100, 5)), None);
    assert_eq!(projection.hit_test(HeadlessPoint::new(-1, 0)), None);
    assert_eq!(projection.hit_test(HeadlessPoint::new(0, -1)), None);
    assert_eq!(
        projection.hit_test(HeadlessPoint::new(5, 10)),
        Some(nodes.first)
    );
    assert_eq!(
        projection.hit_test(HeadlessPoint::new(5, 22)),
        Some(nodes.second)
    );
    assert_eq!(projection.hit_test(HeadlessPoint::new(5, 80)), None);
}
