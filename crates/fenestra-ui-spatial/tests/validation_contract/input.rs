use crate::*;

#[test]
fn aggregate_input_preserves_four_distinct_borrowed_views() {
    let nodes = [SpatialNodeV2::new(
        SpatialNodeKeyV2::new(0),
        None,
        SpatialPlacementV2::Root,
        SpatialContainerV2::new(LayoutAxisV1::Column, LayoutPaddingV1::new(1, 2, 3, 4), 5),
    )];
    let points = [SpatialPointV2::new(
        SpatialScalarV2::new(6),
        SpatialScalarV2::new(7),
    )];
    let stops = [SpatialGradientStopV2::new(
        8,
        SpatialRgba8V2::new(9, 10, 11, 12),
    )];
    let image_bytes = vec![13_u8, 14, 15, 16].into_boxed_slice();
    let image_bytes_ptr = image_bytes.as_ptr();
    let images = [SpatialImageV2::new(
        SpatialImageKeyV2::new(17),
        1,
        1,
        4,
        image_bytes,
    )];
    let hits = [SpatialHitV2::new(
        SpatialNodeKeyV2::new(18),
        19,
        SpatialCoverageV2::Fill {
            shape: SpatialShapeKeyV2::new(20),
            rule: SpatialFillRuleV2::EvenOdd,
        },
        None,
        SpatialInputPolicyV2::Ignore,
    )];

    let topology = SpatialTopologyInputV2::new(SpatialViewportV2::new(21, 22), &nodes);
    let geometry = SpatialGeometryInputV2::new(&points, &[], &[], &[], &[]);
    let resources = SpatialResourceInputV2::new(&stops, &[], &images);
    let items = SpatialItemInputV2::new(&[], &hits, &[]);
    let input = SpatialInputV2::new(topology, geometry, resources, items);

    assert_eq!(input.topology().viewport(), SpatialViewportV2::new(21, 22));
    assert_same_slice(input.topology().nodes(), &nodes);
    assert_same_slice(input.geometry().polygon_points(), &points);
    assert_same_slice(input.resources().gradient_stops(), &stops);
    assert_same_slice(input.resources().images(), &images);
    assert_eq!(
        input.resources().images()[0].bytes().as_ptr(),
        image_bytes_ptr
    );
    assert_eq!(input.resources().images()[0].bytes(), &[13, 14, 15, 16]);
    assert_same_slice(input.items().hit_items(), &hits);
}

#[test]
fn aggregate_input_accepts_four_empty_raw_views_without_validation() {
    let topology = SpatialTopologyInputV2::new(SpatialViewportV2::new(-1, -2), &[]);
    let geometry = SpatialGeometryInputV2::new(&[], &[], &[], &[], &[]);
    let resources = SpatialResourceInputV2::new(&[], &[], &[]);
    let items = SpatialItemInputV2::new(&[], &[], &[]);
    let input = SpatialInputV2::new(topology, geometry, resources, items);

    assert!(input.topology().nodes().is_empty());
    assert!(input.geometry().polygon_points().is_empty());
    assert!(input.geometry().path_verbs().is_empty());
    assert!(input.geometry().paths().is_empty());
    assert!(input.geometry().shapes().is_empty());
    assert!(input.geometry().clips().is_empty());
    assert!(input.resources().gradient_stops().is_empty());
    assert!(input.resources().brushes().is_empty());
    assert!(input.resources().images().is_empty());
    assert!(input.items().paint_items().is_empty());
    assert!(input.items().hit_items().is_empty());
    assert!(input.items().semantic_items().is_empty());
}

fn assert_same_slice<T>(actual: &[T], expected: &[T]) {
    assert_eq!(actual.as_ptr(), expected.as_ptr());
    assert_eq!(actual.len(), expected.len());
}
