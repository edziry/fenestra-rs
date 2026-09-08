use fenestra_ui_ir::prototype::{
    ChildFactory, InvalidationClass, PropertyValue, SpatialBindingV2, SpatialPlacementRecipeV2,
    SpatialShapeGeometryV2, TemplateNodeId,
};

use super::*;
use crate::{Color, Dimension, Style};

#[test]
fn generated_identity_preserves_preorder_and_construction_ownership() {
    let view = View::new(
        "example",
        Element::row("root")
            .child(Element::column("left").child(Element::rect("nested")))
            .child(Element::rect("right")),
    );
    let lowered = lower(&view, Limits::default()).expect("valid nested view");
    assert_eq!(lowered.names, ["root", "left", "nested", "right"]);
    let construction = lowered.program.style().construction();
    let root = construction.root_factory();
    let children = root
        .children()
        .map(|child| match child {
            ChildFactory::Static { template, .. } => template.id().get(),
            ChildFactory::Region { .. } => panic!("static views have no regions"),
        })
        .collect::<Vec<_>>();
    assert_eq!(children, [1, 3]);
    for (index, declaration) in lowered.program.program().nodes().iter().enumerate() {
        assert_eq!(declaration.template().value().get() as usize, index);
        assert!(
            construction
                .template(*declaration.template().value())
                .is_some()
        );
    }
}

#[test]
fn dimensions_remain_property_bound_in_placement_and_coverage() {
    let view = View::new(
        "example",
        Element::rect("target").style(
            Style::new()
                .width(37)
                .height(19)
                .background(Color::rgba8(20, 40, 60, 128)),
        ),
    );
    let lowered = lower(&view, Limits::default()).expect("valid rectangle");
    let declaration = &lowered.program.program().nodes()[0];
    let SpatialPlacementRecipeV2::Layout(placement) = declaration.placement() else {
        panic!("static elements participate in layout");
    };
    let SpatialShapeGeometryV2::Rect { width, height, .. } = declaration.shapes()[0].geometry()
    else {
        panic!("elements use rectangle coverage");
    };
    assert_eq!(*width.value(), SpatialBindingV2::Property(WIDTH));
    assert_eq!(*height.value(), SpatialBindingV2::Property(HEIGHT));
    for (dimension, property) in [(placement.width(), WIDTH), (placement.height(), HEIGHT)] {
        assert_eq!(
            *dimension.minimum().value(),
            SpatialBindingV2::Property(property)
        );
        assert_eq!(
            *dimension.preferred().value(),
            SpatialBindingV2::Property(property)
        );
        assert_eq!(
            *dimension.maximum().value(),
            SpatialBindingV2::Property(property)
        );
    }
    let template = lowered.program.style().construction().root_factory();
    assert_eq!(
        template.effective_value(WIDTH),
        Some(&PropertyValue::ScalarI32(37))
    );
    assert_eq!(
        template.effective_value(HEIGHT),
        Some(&PropertyValue::ScalarI32(19))
    );
    assert_eq!(
        template.effective_value(BACKGROUND),
        Some(&PropertyValue::Rgba8([20, 40, 60, 128]))
    );
}

#[test]
fn schema_records_downstream_work_for_each_typed_property() {
    let view = View::new("example", Element::rect("target"));
    let lowered = lower(&view, Limits::default()).expect("valid rectangle");
    let template = lowered
        .program
        .style()
        .construction()
        .template(TemplateNodeId::new(0))
        .unwrap();
    let component = template.component();
    for property in [WIDTH, HEIGHT, PADDING, GAP] {
        let invalidation = component.property(property).unwrap().invalidation();
        for class in [
            InvalidationClass::Layout,
            InvalidationClass::Paint,
            InvalidationClass::HitTest,
        ] {
            assert!(invalidation.contains(class));
        }
    }
    assert_eq!(
        component
            .property(BACKGROUND)
            .unwrap()
            .invalidation()
            .iter()
            .collect::<Vec<_>>(),
        [InvalidationClass::Paint]
    );
    assert_eq!(
        component
            .property(INPUT)
            .unwrap()
            .invalidation()
            .iter()
            .collect::<Vec<_>>(),
        [InvalidationClass::HitTest]
    );
}

fn lower(view: &View, limits: Limits) -> Result<Lowered, Error> {
    let flat = prepare(view, limits)?;
    let sizes = flat
        .nodes
        .iter()
        .map(|node| {
            let (Dimension::Px(width), Dimension::Px(height)) =
                (node.element.style.width, node.element.style.height)
            else {
                panic!("fixed lowering fixture");
            };
            Size::new(width as u32, height as u32)
        })
        .collect::<Vec<_>>();
    let styles = flat
        .nodes
        .iter()
        .map(|node| node.element.style)
        .collect::<Vec<_>>();
    lower_prepared(&flat, &sizes, &styles, limits)
}
