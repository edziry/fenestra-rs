use fenestra_ui_spatial::prototype::{
    SpatialCoverageV2, SpatialFillRuleV2, SpatialNodeKeyV2, SpatialShapeKeyV2,
};

use crate::*;

#[test]
fn borrowed_resource_input_preserves_all_three_tables_and_image_ownership() {
    let stops = [SpatialGradientStopV2::new(
        0,
        SpatialRgba8V2::new(1, 2, 3, 4),
    )];
    let brushes = [SpatialBrushV2::new(
        SpatialBrushKeyV2::new(0),
        SpatialBrushContentV2::Solid {
            color: stops[0].color(),
        },
    )];
    let images = [SpatialImageV2::new(
        SpatialImageKeyV2::new(0),
        1,
        1,
        4,
        vec![4, 3, 2, 1].into_boxed_slice(),
    )];

    {
        let input = SpatialResourceInputV2::new(&stops, &brushes, &images);
        assert_same_slice(input.gradient_stops(), &stops);
        assert_same_slice(input.brushes(), &brushes);
        assert_same_slice(input.images(), &images);
        assert_eq!(input.images()[0].bytes(), &[4, 3, 2, 1]);
    }
    assert_eq!(images[0].bytes(), &[4, 3, 2, 1]);
}

#[test]
fn borrowed_item_input_preserves_three_independent_tables() {
    let coverage = SpatialCoverageV2::Fill {
        shape: SpatialShapeKeyV2::new(0),
        rule: SpatialFillRuleV2::NonZero,
    };
    let paint_items = [SpatialPaintV2::new(
        SpatialNodeKeyV2::new(1),
        2,
        SpatialPaintContentV2::CoveragePaint {
            coverage,
            brush: SpatialBrushKeyV2::new(3),
            opacity: 4,
            clip: None,
        },
    )];
    let hit_items = [SpatialHitV2::new(
        SpatialNodeKeyV2::new(5),
        6,
        coverage,
        None,
        SpatialInputPolicyV2::Accept,
    )];
    let semantic_items = [SpatialSemanticGeometryV2::new(
        SpatialNodeKeyV2::new(7),
        8,
        SpatialShapeKeyV2::new(9),
        SpatialFillRuleV2::EvenOdd,
        None,
    )];

    let input = SpatialItemInputV2::new(&paint_items, &hit_items, &semantic_items);
    assert_same_slice(input.paint_items(), &paint_items);
    assert_same_slice(input.hit_items(), &hit_items);
    assert_same_slice(input.semantic_items(), &semantic_items);
}

#[test]
fn empty_raw_inputs_are_preserved_without_validation() {
    let resources = SpatialResourceInputV2::new(&[], &[], &[]);
    assert!(resources.gradient_stops().is_empty());
    assert!(resources.brushes().is_empty());
    assert!(resources.images().is_empty());

    let items = SpatialItemInputV2::new(&[], &[], &[]);
    assert!(items.paint_items().is_empty());
    assert!(items.hit_items().is_empty());
    assert!(items.semantic_items().is_empty());
}

fn assert_same_slice<T>(observed: &[T], expected: &[T]) {
    assert_eq!(observed.as_ptr(), expected.as_ptr());
    assert_eq!(observed.len(), expected.len());
}
