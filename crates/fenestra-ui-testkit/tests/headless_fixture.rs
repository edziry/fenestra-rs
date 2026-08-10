#[path = "headless/fixture_support.rs"]
mod support;

use fenestra_ui_ir::prototype::{
    ChildFactory, InputPolicy, InvalidationClass, InvalidationSet, PropertyValue, ValueType,
};
use fenestra_ui_runtime::prototype::{
    HeadlessProjectionCapacity, HeadlessSurface, RuntimeCapacity,
};
use fenestra_ui_testkit::prototype::HeadlessFixtureV1;

use support::{
    COLOR, COMPONENT, CONTAINER_TEMPLATE, CONTROL_TEMPLATE, HEIGHT, INPUT, ITEM_TEMPLATE,
    ITEMS_REGION, ROOT_TEMPLATE, VISIBLE, WIDTH,
};

fn invalidation(classes: &[InvalidationClass]) -> InvalidationSet {
    classes.iter().fold(InvalidationSet::NONE, |set, class| {
        set.union(InvalidationSet::from_class(*class))
    })
}

fn dimension_invalidation() -> InvalidationSet {
    invalidation(&[
        InvalidationClass::Layout,
        InvalidationClass::Semantics,
        InvalidationClass::HitTest,
        InvalidationClass::Paint,
        InvalidationClass::Composition,
    ])
}

fn visibility_invalidation() -> InvalidationSet {
    invalidation(&[
        InvalidationClass::Semantics,
        InvalidationClass::HitTest,
        InvalidationClass::Paint,
    ])
}

fn region_invalidation() -> InvalidationSet {
    invalidation(&[
        InvalidationClass::Structure,
        InvalidationClass::Layout,
        InvalidationClass::Semantics,
        InvalidationClass::HitTest,
        InvalidationClass::Paint,
        InvalidationClass::Composition,
    ])
}

#[test]
fn registered_fixture_exposes_the_exact_style_specification_and_bounds() {
    let fixture = HeadlessFixtureV1::build().expect("registered headless fixture should validate");
    let style = fixture.style();
    let construction = style.construction();
    let spec = fixture.spec();

    assert_eq!(construction.root_factory().id(), ROOT_TEMPLATE);
    assert!(construction.template(CONTAINER_TEMPLATE).is_some());
    assert!(construction.template(CONTROL_TEMPLATE).is_some());
    assert!(construction.template(ITEM_TEMPLATE).is_some());
    assert!(construction.region(support::ITEMS_REGION).is_some());
    assert_eq!(
        style
            .assignments()
            .map(|assignment| (
                assignment.target().id(),
                assignment.property().id(),
                assignment.replacement().clone(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                CONTROL_TEMPLATE,
                COLOR,
                PropertyValue::Rgba8([10, 20, 30, 255]),
            ),
            (
                ITEM_TEMPLATE,
                COLOR,
                PropertyValue::Rgba8([80, 90, 100, 255]),
            ),
        ]
    );
    assert_eq!(
        style
            .assignment(CONTROL_TEMPLATE, COLOR)
            .expect("control color assignment should exist")
            .replacement(),
        &PropertyValue::Rgba8([10, 20, 30, 255])
    );
    assert_eq!(
        style
            .assignment(ITEM_TEMPLATE, COLOR)
            .expect("item color assignment should exist")
            .replacement(),
        &PropertyValue::Rgba8([80, 90, 100, 255])
    );
    assert!(style.assignment(ROOT_TEMPLATE, COLOR).is_none());
    assert!(style.assignment(CONTAINER_TEMPLATE, COLOR).is_none());

    assert_eq!(spec.width(), WIDTH);
    assert_eq!(spec.height(), HEIGHT);
    assert_eq!(spec.color(), COLOR);
    assert_eq!(spec.visible(), VISIBLE);
    assert_eq!(spec.input(), INPUT);
    assert_eq!(spec.semantic_template(), CONTROL_TEMPLATE);
    assert_eq!(spec.semantic_label(), 1);
    assert_eq!(
        spec.capacity(),
        HeadlessProjectionCapacity::new(8, 8, 1, 8, 8)
    );
    assert_eq!(fixture.surface(), HeadlessSurface::new(120, 90));
    assert_eq!(
        fixture.runtime_capacity(),
        RuntimeCapacity::new(8, 8, 8, 2, 40, 3)
    );
    let observer_limits = fixture.harness_limits();
    assert_eq!(observer_limits.transactions(), 16);
    assert_eq!(observer_limits.operations_per_transaction(), 8);
    assert_eq!(observer_limits.operations(), 128);
    assert_eq!(observer_limits.live_memberships(), 5);
    assert_eq!(observer_limits.path_depth(), 3);
    assert_eq!(observer_limits.normalized_nodes(), 8);
    assert_eq!(observer_limits.normalized_fragments(), 2);
    assert_eq!(observer_limits.normalized_properties(), 40);
    assert_eq!(observer_limits.applicable_actions(), 64);
    assert_eq!(observer_limits.trace_bytes(), 20_480);
}

#[test]
fn registered_fixture_pins_the_complete_schema_contract() {
    let fixture = HeadlessFixtureV1::build().expect("registered headless fixture should validate");
    let construction = fixture.style().construction();
    let component = construction
        .schema()
        .component(COMPONENT)
        .expect("fixture component should exist");

    assert_eq!(
        component
            .properties()
            .map(|property| (
                property.id(),
                property.value_type(),
                property.default().clone(),
                property.invalidation(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                WIDTH,
                ValueType::ScalarI32,
                PropertyValue::ScalarI32(40),
                dimension_invalidation(),
            ),
            (
                HEIGHT,
                ValueType::ScalarI32,
                PropertyValue::ScalarI32(10),
                dimension_invalidation(),
            ),
            (
                COLOR,
                ValueType::Rgba8,
                PropertyValue::Rgba8([32, 32, 32, 255]),
                InvalidationSet::from_class(InvalidationClass::Paint),
            ),
            (
                VISIBLE,
                ValueType::Bool,
                PropertyValue::Bool(true),
                visibility_invalidation(),
            ),
            (
                INPUT,
                ValueType::InputPolicy,
                PropertyValue::InputPolicy(InputPolicy::Ignore),
                InvalidationSet::from_class(InvalidationClass::HitTest),
            ),
        ]
    );
}

#[test]
fn registered_fixture_pins_every_template_child_and_region_declaration() {
    let fixture = HeadlessFixtureV1::build().expect("registered headless fixture should validate");
    let construction = fixture.style().construction();

    let initial = |template| {
        construction
            .template(template)
            .expect("fixture template should exist")
            .initial_properties()
            .map(|property| (property.property().id(), property.value().clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        initial(ROOT_TEMPLATE),
        vec![
            (WIDTH, PropertyValue::ScalarI32(100)),
            (HEIGHT, PropertyValue::ScalarI32(80)),
            (COLOR, PropertyValue::Rgba8([1, 1, 1, 255])),
        ]
    );
    assert_eq!(
        initial(CONTAINER_TEMPLATE),
        vec![
            (WIDTH, PropertyValue::ScalarI32(80)),
            (HEIGHT, PropertyValue::ScalarI32(50)),
            (COLOR, PropertyValue::Rgba8([2, 2, 2, 255])),
        ]
    );
    assert_eq!(
        initial(CONTROL_TEMPLATE),
        vec![
            (WIDTH, PropertyValue::ScalarI32(30)),
            (COLOR, PropertyValue::Rgba8([3, 3, 3, 255])),
            (INPUT, PropertyValue::InputPolicy(InputPolicy::Accept),),
        ]
    );
    assert_eq!(
        initial(ITEM_TEMPLATE),
        vec![
            (HEIGHT, PropertyValue::ScalarI32(12)),
            (COLOR, PropertyValue::Rgba8([4, 4, 4, 255])),
            (INPUT, PropertyValue::InputPolicy(InputPolicy::Accept),),
        ]
    );
    for template in [
        ROOT_TEMPLATE,
        CONTAINER_TEMPLATE,
        CONTROL_TEMPLATE,
        ITEM_TEMPLATE,
    ] {
        assert_eq!(
            construction
                .template(template)
                .expect("fixture template should exist")
                .component()
                .id(),
            COMPONENT
        );
    }

    let root_children = construction
        .template(ROOT_TEMPLATE)
        .expect("root template should exist")
        .children()
        .map(|child| match child {
            ChildFactory::Static { template, .. } => ("static", template.id()),
            ChildFactory::Region { region, .. } => ("region", region.repeat_body().id()),
        })
        .collect::<Vec<_>>();
    assert_eq!(root_children, vec![("static", CONTAINER_TEMPLATE)]);

    let container_children = construction
        .template(CONTAINER_TEMPLATE)
        .expect("container template should exist")
        .children()
        .map(|child| match child {
            ChildFactory::Static { template, .. } => ("static", template.id()),
            ChildFactory::Region { region, .. } => ("region", region.repeat_body().id()),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        container_children,
        vec![("static", CONTROL_TEMPLATE), ("region", ITEM_TEMPLATE)]
    );
    assert_eq!(
        construction
            .template(CONTROL_TEMPLATE)
            .expect("control template should exist")
            .children()
            .count(),
        0
    );
    assert_eq!(
        construction
            .template(ITEM_TEMPLATE)
            .expect("item template should exist")
            .children()
            .count(),
        0
    );

    let region = construction
        .region(ITEMS_REGION)
        .expect("item region should exist");
    assert_eq!(region.id(), ITEMS_REGION);
    assert_eq!(region.owner().id(), CONTAINER_TEMPLATE);
    assert_eq!(region.repeat_body().id(), ITEM_TEMPLATE);
    assert_eq!(
        region
            .initial_keys()
            .map(|key| key.value())
            .collect::<Vec<_>>(),
        vec![10, 20]
    );
    assert_eq!(region.invalidation(), region_invalidation());
}

#[test]
fn construction_fallbacks_remain_distinct_from_exact_style() {
    let fixture = HeadlessFixtureV1::build().expect("registered headless fixture should validate");
    let construction = fixture.style().construction();

    assert_eq!(
        construction
            .template(ROOT_TEMPLATE)
            .expect("root template should exist")
            .effective_value(WIDTH),
        Some(&PropertyValue::ScalarI32(100))
    );
    assert_eq!(
        construction
            .template(CONTAINER_TEMPLATE)
            .expect("container template should exist")
            .effective_value(HEIGHT),
        Some(&PropertyValue::ScalarI32(50))
    );
    assert_eq!(
        construction
            .template(CONTROL_TEMPLATE)
            .expect("control template should exist")
            .effective_value(COLOR),
        Some(&PropertyValue::Rgba8([3, 3, 3, 255]))
    );
    assert_eq!(
        construction
            .template(ITEM_TEMPLATE)
            .expect("item template should exist")
            .effective_value(INPUT),
        Some(&PropertyValue::InputPolicy(InputPolicy::Accept))
    );
}
