use fenestra_ui_ir::prototype::{
    InputPolicy, PropertyId, SpatialBindingV2, SpatialBrushContentV2, SpatialCoverageRecipeV2,
    SpatialFieldV2, SpatialFillRuleV2, SpatialNodeDeclarationV2, SpatialPaintRecipeV2,
    SpatialPathVerbRecipeV2, SpatialShapeGeometryV2,
};

use crate::support;

const S: i64 = 65_536;

#[test]
fn every_geometry_brush_and_item_branch_resolves_in_owner_scope() {
    let compiled = support::compile_fen(support::FIXTURE);
    let nodes = compiled.spatial().nodes();
    let scene = &nodes[0];
    assert_eq!(
        scene
            .shapes()
            .iter()
            .map(|shape| shape.symbol().value().get())
            .collect::<Vec<_>>(),
        [0, 1, 2, 3]
    );
    assert_scene_shapes(scene);
    assert_scene_brushes(scene);
    assert_scene_clips(scene);
    assert_scene_items(scene);
    assert_tile_content(&nodes[4]);
}

fn assert_scene_shapes(node: &SpatialNodeDeclarationV2) {
    let [rect, circle, polygon, path] = node.shapes() else {
        panic!("scene shape table");
    };
    let SpatialShapeGeometryV2::Rect {
        origin,
        width,
        height,
    } = rect.geometry()
    else {
        panic!("rect");
    };
    assert_eq!(
        bindings([origin.x(), origin.y(), *width, *height]),
        [lit(0), lit(0), property(0), property(1)]
    );
    let SpatialShapeGeometryV2::Circle { center, radius } = circle.geometry() else {
        panic!("circle");
    };
    assert_eq!(
        bindings([center.x(), center.y(), *radius]),
        [lit(5 * S), lit(5 * S), lit(2 * S)]
    );
    let SpatialShapeGeometryV2::Polygon { points } = polygon.geometry() else {
        panic!("polygon");
    };
    assert_eq!(points.len(), 3);
    assert_eq!(
        bindings([points[2].point().x(), points[2].point().y()]),
        [lit(5 * S), lit(8 * S)]
    );
    let SpatialShapeGeometryV2::Path { verbs } = path.geometry() else {
        panic!("path");
    };
    assert_eq!(verbs.len(), 5);
    assert!(matches!(verbs[0], SpatialPathVerbRecipeV2::MoveTo { .. }));
    assert!(matches!(verbs[1], SpatialPathVerbRecipeV2::LineTo { .. }));
    assert!(matches!(
        verbs[2],
        SpatialPathVerbRecipeV2::QuadraticTo { .. }
    ));
    assert!(matches!(verbs[3], SpatialPathVerbRecipeV2::CubicTo { .. }));
    assert!(matches!(verbs[4], SpatialPathVerbRecipeV2::Close { .. }));
}

fn assert_scene_brushes(node: &SpatialNodeDeclarationV2) {
    assert_eq!(
        node.brushes()
            .iter()
            .map(|brush| brush.symbol().value().get())
            .collect::<Vec<_>>(),
        [0, 1]
    );
    let SpatialBrushContentV2::Solid { color } = node.brushes()[0].content() else {
        panic!("solid");
    };
    assert_eq!(
        *color.value(),
        SpatialBindingV2::Property(PropertyId::new(4))
    );
    let SpatialBrushContentV2::LinearGradient { start, end, stops } = node.brushes()[1].content()
    else {
        panic!("gradient");
    };
    assert_eq!(
        bindings([start.x(), start.y(), end.x(), end.y()]),
        [lit(0), lit(0), lit(10 * S), lit(0)]
    );
    assert_eq!(
        stops
            .iter()
            .map(|stop| *stop.offset().value())
            .collect::<Vec<_>>(),
        [0, 32_768, 65_535]
    );
    assert_eq!(
        *stops[0].color().value(),
        SpatialBindingV2::Property(PropertyId::new(5))
    );
    assert_eq!(
        *stops[1].color().value(),
        SpatialBindingV2::Literal([128, 64, 32, 255])
    );
}

fn assert_scene_clips(node: &SpatialNodeDeclarationV2) {
    assert_eq!(
        node.clips()
            .iter()
            .map(|clip| clip.symbol().value().get())
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(node.clips()[0].parent(), None);
    let parent = node.clips()[1].parent().expect("inner clip parent");
    assert_eq!(
        (parent.owner().value().get(), parent.clip().value().get()),
        (0, 0)
    );
    assert_eq!(node.clips()[1].fill_rule(), SpatialFillRuleV2::EvenOdd);
}

fn assert_scene_items(node: &SpatialNodeDeclarationV2) {
    let [fill, stroke, image] = node.paint_items() else {
        panic!("scene paint table");
    };
    let SpatialPaintRecipeV2::CoveragePaint {
        coverage: SpatialCoverageRecipeV2::Fill { shape, rule },
        brush,
        opacity,
        clip,
        ..
    } = fill
    else {
        panic!("fill paint");
    };
    assert_eq!(
        (
            shape.value().get(),
            *rule,
            brush.value().get(),
            *opacity.value()
        ),
        (0, SpatialFillRuleV2::NonZero, 0, 255)
    );
    assert_eq!(clip.expect("fill clip").clip().value().get(), 1);
    let SpatialPaintRecipeV2::CoveragePaint {
        coverage: SpatialCoverageRecipeV2::RoundStroke { shape, width },
        brush,
        opacity,
        clip,
        ..
    } = stroke
    else {
        panic!("stroke paint");
    };
    assert_eq!(
        (
            shape.value().get(),
            *width.value(),
            brush.value().get(),
            *opacity.value(),
            clip.is_none()
        ),
        (3, property(2), 1, 200, true)
    );
    let SpatialPaintRecipeV2::ImagePaint {
        image,
        source_x,
        source_y,
        source_width,
        source_height,
        destination_origin,
        destination_width,
        destination_height,
        opacity,
        clip,
        ..
    } = image
    else {
        panic!("image paint");
    };
    assert_eq!(
        (
            image.value().get(),
            *source_x.value(),
            *source_y.value(),
            *source_width.value(),
            *source_height.value()
        ),
        (0, 0, 0, 2, 2)
    );
    assert_eq!(
        bindings([
            destination_origin.x(),
            destination_origin.y(),
            *destination_width,
            *destination_height
        ]),
        [lit(2 * S), lit(2 * S), lit(8 * S), lit(8 * S)]
    );
    assert_eq!(
        (
            *opacity.value(),
            clip.expect("image clip").clip().value().get()
        ),
        (192, 0)
    );
    assert_eq!(node.hit_items().len(), 3);
    assert_eq!(
        *node.hit_items()[0].input_policy().value(),
        SpatialBindingV2::Property(PropertyId::new(7))
    );
    assert_eq!(
        *node.hit_items()[1].input_policy().value(),
        SpatialBindingV2::Literal(InputPolicy::Accept)
    );
    assert_eq!(
        *node.hit_items()[2].input_policy().value(),
        SpatialBindingV2::Literal(InputPolicy::Ignore)
    );
    assert_eq!(node.semantic_items().len(), 2);
}

fn assert_tile_content(node: &SpatialNodeDeclarationV2) {
    assert_eq!(
        (
            node.shapes().len(),
            node.brushes().len(),
            node.clips().len()
        ),
        (1, 1, 1)
    );
    let parent = node.clips()[0].parent().expect("ancestor clip");
    assert_eq!(
        (parent.owner().value().get(), parent.clip().value().get()),
        (0, 0)
    );
    assert_eq!(
        (
            node.paint_items().len(),
            node.hit_items().len(),
            node.semantic_items().len()
        ),
        (1, 1, 1)
    );
    let local = node.semantic_items()[0]
        .clip()
        .expect("local semantic clip");
    assert_eq!(
        (local.owner().value().get(), local.clip().value().get()),
        (4, 0)
    );
}

fn bindings<const N: usize>(
    fields: [SpatialFieldV2<SpatialBindingV2<i64>>; N],
) -> [SpatialBindingV2<i64>; N] {
    fields.map(|field| *field.value())
}

const fn lit(value: i64) -> SpatialBindingV2<i64> {
    SpatialBindingV2::Literal(value)
}

const fn property(value: u32) -> SpatialBindingV2<i64> {
    SpatialBindingV2::Property(PropertyId::new(value))
}
