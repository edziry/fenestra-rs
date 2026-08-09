use fenestra_ui_ir::prototype::{
    ChildFactory, ComponentTypeId, InputPolicy, InvalidationClass, InvalidationSet, PropertyId,
    PropertyValue, StructuralRegionId, TemplateNodeId, ValueType,
};
use fenestra_ui_testkit::prototype::RuntimeOracleFixtureV1;

#[test]
fn fixture_v1_pins_authored_schema_and_structure() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let construction = fixture.construction();
    let root = construction.root_factory();

    assert_eq!(root.id(), TemplateNodeId::new(0));
    assert_eq!(root.component().id(), ComponentTypeId::new(0));
    assert_eq!(
        root.initial_properties()
            .map(|property| (property.property().id(), property.value().clone()))
            .collect::<Vec<_>>(),
        [(PropertyId::new(0), PropertyValue::ScalarI32(120))]
    );
    assert_eq!(
        root.effective_value(PropertyId::new(0)),
        Some(&PropertyValue::ScalarI32(120))
    );
    assert_eq!(
        root.effective_value(PropertyId::new(1)),
        Some(&PropertyValue::Bool(true))
    );
    assert_eq!(
        root.effective_value(PropertyId::new(2)),
        Some(&PropertyValue::Rgba8([0, 0, 0, 255]))
    );
    assert_eq!(
        root.effective_value(PropertyId::new(3)),
        Some(&PropertyValue::InputPolicy(InputPolicy::Accept))
    );

    let children: Vec<_> = root.children().collect();
    assert_eq!(children.len(), 3);
    assert_static(children[0], 1, 1);
    assert_region(children[1], 0, 0, 2, &[7, 8]);
    assert_region(children[2], 2, 0, 5, &[7]);

    let item = construction
        .template(TemplateNodeId::new(2))
        .expect("item template should resolve");
    assert_eq!(item.component().id(), ComponentTypeId::new(2));
    assert_eq!(
        property_ids(item.component()),
        vec![PropertyId::new(0), PropertyId::new(1)]
    );
    let item_children: Vec<_> = item.children().collect();
    assert_eq!(item_children.len(), 2);
    assert_static(item_children[0], 3, 3);
    assert_region(item_children[1], 1, 2, 4, &[1]);

    let components: Vec<_> = (0..=5)
        .map(|template| {
            construction
                .template(TemplateNodeId::new(template))
                .expect("registered template should resolve")
                .component()
        })
        .collect();
    assert_property_schema(
        components[0],
        0,
        ValueType::ScalarI32,
        PropertyValue::ScalarI32(100),
        set(&[InvalidationClass::Layout, InvalidationClass::Paint]),
    );
    assert_property_schema(
        components[0],
        1,
        ValueType::Bool,
        PropertyValue::Bool(true),
        set(&[
            InvalidationClass::Semantics,
            InvalidationClass::HitTest,
            InvalidationClass::Paint,
        ]),
    );
    assert_property_schema(
        components[0],
        2,
        ValueType::Rgba8,
        PropertyValue::Rgba8([0, 0, 0, 255]),
        set(&[InvalidationClass::Paint]),
    );
    assert_property_schema(
        components[0],
        3,
        ValueType::InputPolicy,
        PropertyValue::InputPolicy(InputPolicy::Accept),
        set(&[InvalidationClass::HitTest]),
    );
    assert_property_schema(
        components[1],
        0,
        ValueType::Bool,
        PropertyValue::Bool(true),
        set(&[InvalidationClass::Semantics, InvalidationClass::Paint]),
    );
    assert_property_schema(
        components[2],
        0,
        ValueType::ScalarI32,
        PropertyValue::ScalarI32(10),
        set(&[
            InvalidationClass::Intrinsic,
            InvalidationClass::Layout,
            InvalidationClass::Paint,
        ]),
    );
    assert_property_schema(
        components[2],
        1,
        ValueType::Bool,
        PropertyValue::Bool(true),
        set(&[
            InvalidationClass::Semantics,
            InvalidationClass::HitTest,
            InvalidationClass::Paint,
        ]),
    );
    assert_property_schema(
        components[3],
        0,
        ValueType::Rgba8,
        PropertyValue::Rgba8([255; 4]),
        set(&[InvalidationClass::Paint]),
    );
    for (component, value) in [(components[4], 1), (components[5], 20)] {
        assert_property_schema(
            component,
            0,
            ValueType::ScalarI32,
            PropertyValue::ScalarI32(value),
            set(&[
                InvalidationClass::Intrinsic,
                InvalidationClass::Layout,
                InvalidationClass::Paint,
            ]),
        );
    }
}

#[test]
fn fixture_v1_pins_invalidation_contracts() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let construction = fixture.construction();
    let root = construction.root_factory();

    assert_eq!(
        invalidation(root.component(), 0),
        set(&[InvalidationClass::Layout, InvalidationClass::Paint])
    );
    assert_eq!(
        invalidation(root.component(), 1),
        set(&[
            InvalidationClass::Semantics,
            InvalidationClass::HitTest,
            InvalidationClass::Paint,
        ])
    );
    assert_eq!(
        invalidation(root.component(), 2),
        set(&[InvalidationClass::Paint])
    );
    assert_eq!(
        invalidation(root.component(), 3),
        set(&[InvalidationClass::HitTest])
    );

    for descriptor in [0, 1] {
        assert_eq!(
            construction
                .region(StructuralRegionId::new(descriptor))
                .expect("region should resolve")
                .invalidation(),
            set(&[
                InvalidationClass::Structure,
                InvalidationClass::Layout,
                InvalidationClass::Paint,
            ])
        );
    }
    assert_eq!(
        construction
            .region(StructuralRegionId::new(2))
            .expect("region should resolve")
            .invalidation(),
        set(&[InvalidationClass::Structure, InvalidationClass::Paint])
    );
}

fn property_ids(component: fenestra_ui_ir::prototype::ComponentSchemaView<'_>) -> Vec<PropertyId> {
    component
        .properties()
        .map(|property| property.id())
        .collect()
}

fn invalidation(
    component: fenestra_ui_ir::prototype::ComponentSchemaView<'_>,
    property: u32,
) -> InvalidationSet {
    component
        .property(PropertyId::new(property))
        .expect("property should resolve")
        .invalidation()
}

fn assert_property_schema(
    component: fenestra_ui_ir::prototype::ComponentSchemaView<'_>,
    id: u32,
    value_type: ValueType,
    default: PropertyValue,
    expected_invalidation: InvalidationSet,
) {
    let property = component
        .property(PropertyId::new(id))
        .expect("property should resolve");
    assert_eq!(property.value_type(), value_type);
    assert_eq!(property.default(), &default);
    assert_eq!(property.invalidation(), expected_invalidation);
}

fn set(classes: &[InvalidationClass]) -> InvalidationSet {
    classes
        .iter()
        .copied()
        .fold(InvalidationSet::NONE, |set, class| {
            set.union(InvalidationSet::from_class(class))
        })
}

fn assert_static(child: ChildFactory<'_>, template: u32, component: u32) {
    let ChildFactory::Static {
        template: found, ..
    } = child
    else {
        panic!("expected static child");
    };
    assert_eq!(found.id(), TemplateNodeId::new(template));
    assert_eq!(found.component().id(), ComponentTypeId::new(component));
}

fn assert_region(
    child: ChildFactory<'_>,
    descriptor: u32,
    owner: u32,
    repeat_body: u32,
    keys: &[u64],
) {
    let ChildFactory::Region { region, .. } = child else {
        panic!("expected region child");
    };
    assert_eq!(region.id(), StructuralRegionId::new(descriptor));
    assert_eq!(region.owner().id(), TemplateNodeId::new(owner));
    assert_eq!(region.repeat_body().id(), TemplateNodeId::new(repeat_body));
    assert_eq!(
        region
            .initial_keys()
            .map(|key| key.value())
            .collect::<Vec<_>>(),
        keys
    );
}
