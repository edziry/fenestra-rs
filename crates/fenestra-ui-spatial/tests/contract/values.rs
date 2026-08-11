use super::super::*;

#[test]
fn topology_values_round_trip_without_runtime_or_candidate_types() {
    let one = SpatialScalarV2::new(1);
    let two = SpatialScalarV2::new(2);
    let three = SpatialScalarV2::new(3);
    let four = SpatialScalarV2::new(4);
    let five = SpatialScalarV2::new(5);
    let six = SpatialScalarV2::new(6);
    let point = SpatialPointV2::new(one, two);
    let offset = SpatialOffsetV2::new(three, four);
    let affine = Affine2V2::new(one, two, three, four, five, six);
    let transform = SpatialLocalTransformV2::new(affine, point);

    assert_eq!(one.raw(), 1);
    assert_eq!(point.x(), one);
    assert_eq!(point.y(), two);
    assert_eq!(offset.x(), three);
    assert_eq!(offset.y(), four);
    assert_eq!(affine.a(), one);
    assert_eq!(affine.b(), two);
    assert_eq!(affine.c(), three);
    assert_eq!(affine.d(), four);
    assert_eq!(affine.tx(), five);
    assert_eq!(affine.ty(), six);
    assert_eq!(transform.affine(), affine);
    assert_eq!(transform.origin(), point);

    let self_anchor = SpatialAnchorV2::new(
        SpatialAnchorComponentV2::Center,
        SpatialAnchorComponentV2::End,
    );
    let target_anchor = SpatialAnchorV2::new(
        SpatialAnchorComponentV2::Start,
        SpatialAnchorComponentV2::Center,
    );
    let free = SpatialFreePlacementV2::new(
        80,
        40,
        self_anchor,
        SpatialAnchorTargetV2::Node(SpatialNodeKeyV2::new(7)),
        target_anchor,
        offset,
        transform,
    );

    assert_eq!(self_anchor.horizontal(), SpatialAnchorComponentV2::Center);
    assert_eq!(self_anchor.vertical(), SpatialAnchorComponentV2::End);
    assert_eq!(free.width(), 80);
    assert_eq!(free.height(), 40);
    assert_eq!(free.self_anchor(), self_anchor);
    assert_eq!(
        free.target(),
        SpatialAnchorTargetV2::Node(SpatialNodeKeyV2::new(7))
    );
    assert_eq!(free.target_anchor(), target_anchor);
    assert_eq!(free.offset(), offset);
    assert_eq!(free.transform(), transform);

    let dimension = LayoutDimensionV1::new(10, 20, 30);
    let layout = SpatialLayoutPlacementV2::new(dimension, dimension, transform);
    assert_eq!(layout.width(), dimension);
    assert_eq!(layout.height(), dimension);
    assert_eq!(layout.transform(), transform);

    let padding = LayoutPaddingV1::new(1, 2, 3, 4);
    let container = SpatialContainerV2::new(LayoutAxisV1::Row, padding, 5);
    assert_eq!(container.axis(), LayoutAxisV1::Row);
    assert_eq!(container.padding(), padding);
    assert_eq!(container.gap(), 5);

    let root = SpatialNodeV2::new(
        SpatialNodeKeyV2::new(0),
        None,
        SpatialPlacementV2::Root,
        container,
    );
    let layout_node = SpatialNodeV2::new(
        SpatialNodeKeyV2::new(1),
        Some(SpatialNodeKeyV2::new(0)),
        SpatialPlacementV2::Layout(layout),
        container,
    );
    let free_node = SpatialNodeV2::new(
        SpatialNodeKeyV2::new(2),
        Some(SpatialNodeKeyV2::new(1)),
        SpatialPlacementV2::Free(free),
        container,
    );
    let viewport = SpatialViewportV2::new(1280, 720);
    let nodes = [root, layout_node, free_node];
    let input = SpatialTopologyInputV2::new(viewport, &nodes);

    assert_eq!(SpatialNodeKeyV2::new(9).get(), 9);
    assert_eq!(viewport.width(), 1280);
    assert_eq!(viewport.height(), 720);
    assert_eq!(root.key(), SpatialNodeKeyV2::new(0));
    assert_eq!(root.parent(), None);
    assert_eq!(root.placement(), SpatialPlacementV2::Root);
    assert_eq!(root.container(), container);
    assert_eq!(layout_node.parent(), Some(SpatialNodeKeyV2::new(0)));
    assert_eq!(layout_node.placement(), SpatialPlacementV2::Layout(layout));
    assert_eq!(free_node.placement(), SpatialPlacementV2::Free(free));
    assert_eq!(input.viewport(), viewport);
    assert_eq!(input.nodes(), nodes.as_slice());
}

#[test]
fn payload_enums_remain_exhaustively_matchable() {
    let target = SpatialAnchorTargetV2::Node(SpatialNodeKeyV2::new(41));
    let target_key = match target {
        SpatialAnchorTargetV2::Viewport | SpatialAnchorTargetV2::Parent => None,
        SpatialAnchorTargetV2::Node(key) => Some(key),
    };
    assert_eq!(target_key, Some(SpatialNodeKeyV2::new(41)));

    let placement = SpatialPlacementV2::Root;
    let kind = match placement {
        SpatialPlacementV2::Root => SpatialPlacementKindV2::Root,
        SpatialPlacementV2::Layout(_) => SpatialPlacementKindV2::Layout,
        SpatialPlacementV2::Free(_) => SpatialPlacementKindV2::Free,
    };
    assert_eq!(kind, SpatialPlacementKindV2::Root);
}
