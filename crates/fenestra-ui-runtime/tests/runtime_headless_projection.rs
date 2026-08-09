#[path = "support/headless_projection.rs"]
mod headless_projection;
mod support;

use fenestra_ui_ir::prototype::{InputPolicy, PropertyValue};
use fenestra_ui_runtime::prototype::{
    HeadlessSemanticAction, HeadlessSemanticRole, HeadlessSurface,
};

use headless_projection::{CONTAINER_COLOR, ProjectionNodes, ROOT_COLOR, nodes, rect, runtime};
use support::headless::{
    COLOR, CONTROL_STYLE_COLOR, HEIGHT, INPUT, INSERTED_KEY, ITEM_STYLE_COLOR, ROOT_WIDTH,
    SCHEMA_WIDTH, VISIBLE, WIDTH,
};

fn ordered_nodes(nodes: ProjectionNodes) -> Vec<fenestra_ui_runtime::prototype::NodeId> {
    vec![
        nodes.root,
        nodes.container,
        nodes.control,
        nodes.first,
        nodes.second,
    ]
}

#[test]
fn initial_projection_matches_every_manual_ordered_record() {
    let runtime = runtime(HeadlessSurface::new(120, 90));
    let committed = runtime.committed();
    let nodes = nodes(&committed);
    let projection = committed
        .headless_projection()
        .expect("headless projection should exist");

    assert_eq!(
        projection
            .computed_styles()
            .map(|style| style.node())
            .collect::<Vec<_>>(),
        ordered_nodes(nodes)
    );
    assert_eq!(
        projection
            .computed_styles()
            .map(|style| {
                (
                    style.node(),
                    style.property(WIDTH).cloned(),
                    style.property(HEIGHT).cloned(),
                    style.property(COLOR).cloned(),
                    style.property(VISIBLE).cloned(),
                    style.property(INPUT).cloned(),
                )
            })
            .collect::<Vec<_>>(),
        vec![
            (
                nodes.root,
                Some(PropertyValue::ScalarI32(ROOT_WIDTH)),
                Some(PropertyValue::ScalarI32(80)),
                Some(PropertyValue::Rgba8(ROOT_COLOR)),
                Some(PropertyValue::Bool(true)),
                Some(PropertyValue::InputPolicy(InputPolicy::Ignore)),
            ),
            (
                nodes.container,
                Some(PropertyValue::ScalarI32(80)),
                Some(PropertyValue::ScalarI32(50)),
                Some(PropertyValue::Rgba8(CONTAINER_COLOR)),
                Some(PropertyValue::Bool(true)),
                Some(PropertyValue::InputPolicy(InputPolicy::Ignore)),
            ),
            (
                nodes.control,
                Some(PropertyValue::ScalarI32(30)),
                Some(PropertyValue::ScalarI32(10)),
                Some(PropertyValue::Rgba8(CONTROL_STYLE_COLOR)),
                Some(PropertyValue::Bool(true)),
                Some(PropertyValue::InputPolicy(InputPolicy::Accept)),
            ),
            (
                nodes.first,
                Some(PropertyValue::ScalarI32(SCHEMA_WIDTH)),
                Some(PropertyValue::ScalarI32(12)),
                Some(PropertyValue::Rgba8(ITEM_STYLE_COLOR)),
                Some(PropertyValue::Bool(true)),
                Some(PropertyValue::InputPolicy(InputPolicy::Accept)),
            ),
            (
                nodes.second,
                Some(PropertyValue::ScalarI32(SCHEMA_WIDTH)),
                Some(PropertyValue::ScalarI32(12)),
                Some(PropertyValue::Rgba8(ITEM_STYLE_COLOR)),
                Some(PropertyValue::Bool(true)),
                Some(PropertyValue::InputPolicy(InputPolicy::Accept)),
            ),
        ]
    );
    assert_eq!(
        projection
            .geometries()
            .map(|geometry| (geometry.node(), geometry.bounds(), geometry.clip()))
            .collect::<Vec<_>>(),
        vec![
            (nodes.root, rect(0, 0, 100, 80), rect(0, 0, 100, 80)),
            (nodes.container, rect(0, 0, 80, 50), rect(0, 0, 80, 50),),
            (nodes.control, rect(0, 0, 30, 10), rect(0, 0, 30, 10)),
            (nodes.first, rect(0, 10, 40, 12), rect(0, 10, 40, 12)),
            (nodes.second, rect(0, 22, 40, 12), rect(0, 22, 40, 12)),
        ]
    );
    assert_eq!(
        projection
            .semantics()
            .map(|semantic| {
                (
                    semantic.node(),
                    semantic.role(),
                    semantic.label(),
                    semantic.action(),
                )
            })
            .collect::<Vec<_>>(),
        vec![(
            nodes.control,
            HeadlessSemanticRole::Control,
            1,
            HeadlessSemanticAction::Activate,
        )]
    );
    assert_eq!(
        projection
            .hit_regions()
            .map(|hit| (hit.node(), hit.clip()))
            .collect::<Vec<_>>(),
        vec![
            (nodes.control, rect(0, 0, 30, 10)),
            (nodes.first, rect(0, 10, 40, 12)),
            (nodes.second, rect(0, 22, 40, 12)),
        ]
    );
    assert_eq!(
        projection
            .scene_rectangles()
            .map(|scene| (scene.node(), scene.rectangle(), scene.color()))
            .collect::<Vec<_>>(),
        vec![
            (nodes.root, rect(0, 0, 100, 80), ROOT_COLOR),
            (nodes.container, rect(0, 0, 80, 50), CONTAINER_COLOR,),
            (nodes.control, rect(0, 0, 30, 10), CONTROL_STYLE_COLOR,),
            (nodes.first, rect(0, 10, 40, 12), ITEM_STYLE_COLOR,),
            (nodes.second, rect(0, 22, 40, 12), ITEM_STYLE_COLOR,),
        ]
    );
    assert_eq!(projection.computed_style_count(), 5);
    assert_eq!(projection.geometry_count(), 5);
    assert_eq!(projection.semantic_count(), 1);
    assert_eq!(projection.hit_region_count(), 3);
    assert_eq!(projection.scene_rectangle_count(), 5);
}

#[test]
fn clipped_surface_preserves_bounds_and_uses_canonical_empty_clips() {
    let runtime = runtime(HeadlessSurface::new(25, 15));
    let committed = runtime.committed();
    let nodes = nodes(&committed);
    let projection = committed
        .headless_projection()
        .expect("headless projection should exist");

    let expected = [
        (nodes.root, rect(0, 0, 100, 80), rect(0, 0, 25, 15)),
        (nodes.container, rect(0, 0, 80, 50), rect(0, 0, 25, 15)),
        (nodes.control, rect(0, 0, 30, 10), rect(0, 0, 25, 10)),
        (nodes.first, rect(0, 10, 40, 12), rect(0, 10, 25, 5)),
        (nodes.second, rect(0, 22, 40, 12), rect(0, 22, 25, 0)),
    ];
    assert_eq!(
        projection
            .geometries()
            .map(|geometry| (geometry.node(), geometry.bounds(), geometry.clip()))
            .collect::<Vec<_>>(),
        expected.clone()
    );
    assert_eq!(
        projection
            .scene_rectangles()
            .map(|scene| (scene.node(), scene.rectangle(), scene.color()))
            .collect::<Vec<_>>(),
        vec![
            (nodes.root, rect(0, 0, 25, 15), ROOT_COLOR),
            (nodes.container, rect(0, 0, 25, 15), CONTAINER_COLOR),
            (nodes.control, rect(0, 0, 25, 10), CONTROL_STYLE_COLOR,),
            (nodes.first, rect(0, 10, 25, 5), ITEM_STYLE_COLOR),
        ]
    );
    assert_eq!(
        projection
            .hit_regions()
            .map(|hit| (hit.node(), hit.clip()))
            .collect::<Vec<_>>(),
        vec![
            (nodes.control, rect(0, 0, 25, 10)),
            (nodes.first, rect(0, 10, 25, 5)),
        ]
    );
    assert_eq!(projection.semantic_count(), 1);
}

#[test]
fn projection_view_debug_is_bounded_and_payload_free() {
    let runtime = runtime(HeadlessSurface::new(120, 90));
    let committed = runtime.committed();
    let projection = committed
        .headless_projection()
        .expect("headless projection should exist");
    let rendered = format!("{projection:?}");

    assert_eq!(
        rendered,
        "HeadlessProjectionView { generation: RuntimeGeneration(0), computed_style_count: 5, \
         geometry_count: 5, semantic_count: 1, hit_region_count: 3, \
         scene_rectangle_count: 5, .. }"
    );
}

#[test]
fn keyed_insertion_uses_authored_order_instead_of_arena_order() {
    let mut runtime = runtime(HeadlessSurface::new(120, 90));
    let before = runtime.committed();
    let nodes = nodes(&before);
    let mut transaction = runtime.begin_transaction();
    transaction
        .insert_keyed(nodes.items, INSERTED_KEY, 1)
        .expect("middle item insert should stage");
    runtime
        .commit(transaction)
        .expect("middle item insert should publish");

    let committed = runtime.committed();
    let inserted = committed
        .keyed_member(nodes.items, INSERTED_KEY)
        .expect("inserted item should exist");
    let projection = committed
        .headless_projection()
        .expect("headless projection should exist");
    let expected = vec![
        nodes.root,
        nodes.container,
        nodes.control,
        nodes.first,
        inserted,
        nodes.second,
    ];

    assert_eq!(
        projection
            .computed_styles()
            .map(|style| style.node())
            .collect::<Vec<_>>(),
        expected.clone()
    );
    assert_eq!(
        projection
            .geometries()
            .map(|geometry| geometry.node())
            .collect::<Vec<_>>(),
        expected.clone()
    );
    assert_eq!(
        projection
            .scene_rectangles()
            .map(|scene| scene.node())
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        projection
            .hit_regions()
            .map(|hit| hit.node())
            .collect::<Vec<_>>(),
        vec![nodes.control, nodes.first, inserted, nodes.second]
    );
}
