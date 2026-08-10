use fenestra_ui_ir::prototype::{
    ChildSlot, ComponentSchema, ComponentTypeId, ConstructionProgram, InitialKey, InitialProperty,
    InputPolicy, InvalidationClass, InvalidationSet, PropertyId, PropertySchema, PropertyValue,
    SUPPORTED_CONSTRUCTION_FORMAT, SUPPORTED_SCHEMA_FORMAT, SUPPORTED_STYLE_FORMAT, SchemaManifest,
    SchemaNamespace, SchemaRevision, StructuralRegion, StructuralRegionId, StyleAssignment,
    StyleProgram, TemplateNode, TemplateNodeId, ValueType,
};

use super::logical_span;

const COMPONENT: ComponentTypeId = ComponentTypeId::new(0);
const WIDTH: PropertyId = PropertyId::new(0);
const HEIGHT: PropertyId = PropertyId::new(1);
const COLOR: PropertyId = PropertyId::new(2);
const VISIBLE: PropertyId = PropertyId::new(3);
const INPUT: PropertyId = PropertyId::new(4);
const ROOT: TemplateNodeId = TemplateNodeId::new(0);
const CONTAINER: TemplateNodeId = TemplateNodeId::new(1);
const CONTROL: TemplateNodeId = TemplateNodeId::new(2);
const ITEM: TemplateNodeId = TemplateNodeId::new(3);
const ITEMS: StructuralRegionId = StructuralRegionId::new(0);
const NAMESPACE: SchemaNamespace = SchemaNamespace::new(8_001);
const REVISION: SchemaRevision = SchemaRevision::new(1);

pub fn expected_schema() -> SchemaManifest {
    SchemaManifest::new(
        SUPPORTED_SCHEMA_FORMAT,
        NAMESPACE,
        REVISION,
        vec![ComponentSchema::new(
            COMPONENT,
            vec![
                property(
                    WIDTH,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(40),
                    dimension_invalidation(),
                    3,
                ),
                property(
                    HEIGHT,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(10),
                    dimension_invalidation(),
                    4,
                ),
                property(
                    COLOR,
                    ValueType::Rgba8,
                    PropertyValue::Rgba8([32, 32, 32, 255]),
                    InvalidationSet::from_class(InvalidationClass::Paint),
                    5,
                ),
                property(
                    VISIBLE,
                    ValueType::Bool,
                    PropertyValue::Bool(true),
                    visibility_invalidation(),
                    6,
                ),
                property(
                    INPUT,
                    ValueType::InputPolicy,
                    PropertyValue::InputPolicy(InputPolicy::Ignore),
                    InvalidationSet::from_class(InvalidationClass::HitTest),
                    7,
                ),
            ],
            logical_span(2),
        )],
        logical_span(1),
    )
}

pub fn expected_construction() -> ConstructionProgram {
    ConstructionProgram::new(
        SUPPORTED_CONSTRUCTION_FORMAT,
        NAMESPACE,
        REVISION,
        vec![
            template(
                ROOT,
                vec![
                    initial(WIDTH, PropertyValue::ScalarI32(100), 10),
                    initial(HEIGHT, PropertyValue::ScalarI32(80), 11),
                    initial(COLOR, PropertyValue::Rgba8([1, 1, 1, 255]), 12),
                ],
                vec![ChildSlot::static_node(CONTAINER, logical_span(13))],
                9,
            ),
            template(
                CONTAINER,
                vec![
                    initial(WIDTH, PropertyValue::ScalarI32(80), 15),
                    initial(HEIGHT, PropertyValue::ScalarI32(50), 16),
                    initial(COLOR, PropertyValue::Rgba8([2, 2, 2, 255]), 17),
                ],
                vec![
                    ChildSlot::static_node(CONTROL, logical_span(18)),
                    ChildSlot::region(ITEMS, logical_span(19)),
                ],
                14,
            ),
            template(
                CONTROL,
                vec![
                    initial(WIDTH, PropertyValue::ScalarI32(30), 21),
                    initial(COLOR, PropertyValue::Rgba8([3, 3, 3, 255]), 22),
                    initial(INPUT, PropertyValue::InputPolicy(InputPolicy::Accept), 23),
                ],
                Vec::new(),
                20,
            ),
            template(
                ITEM,
                vec![
                    initial(HEIGHT, PropertyValue::ScalarI32(12), 25),
                    initial(COLOR, PropertyValue::Rgba8([4, 4, 4, 255]), 26),
                    initial(INPUT, PropertyValue::InputPolicy(InputPolicy::Accept), 27),
                ],
                Vec::new(),
                24,
            ),
        ],
        vec![StructuralRegion::new(
            ITEMS,
            CONTAINER,
            ITEM,
            vec![
                InitialKey::new(10, logical_span(29)),
                InitialKey::new(20, logical_span(30)),
            ],
            region_invalidation(),
            logical_span(28),
        )],
        logical_span(8),
    )
}

pub fn expected_style() -> StyleProgram {
    StyleProgram::new(
        SUPPORTED_STYLE_FORMAT,
        NAMESPACE,
        REVISION,
        vec![
            StyleAssignment::new(
                CONTROL,
                COLOR,
                PropertyValue::Rgba8([10, 20, 30, 255]),
                logical_span(32),
            ),
            StyleAssignment::new(
                ITEM,
                COLOR,
                PropertyValue::Rgba8([80, 90, 100, 255]),
                logical_span(33),
            ),
        ],
        logical_span(31),
    )
}

fn property(
    id: PropertyId,
    value_type: ValueType,
    default: PropertyValue,
    invalidation: InvalidationSet,
    ordinal: u32,
) -> PropertySchema {
    PropertySchema::new(id, value_type, default, invalidation, logical_span(ordinal))
}

fn template(
    id: TemplateNodeId,
    properties: Vec<InitialProperty>,
    children: Vec<ChildSlot>,
    ordinal: u32,
) -> TemplateNode {
    TemplateNode::new(id, COMPONENT, properties, children, logical_span(ordinal))
}

fn initial(id: PropertyId, value: PropertyValue, ordinal: u32) -> InitialProperty {
    InitialProperty::new(id, value, logical_span(ordinal))
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

fn invalidation(classes: &[InvalidationClass]) -> InvalidationSet {
    classes.iter().fold(InvalidationSet::NONE, |set, class| {
        set.union(InvalidationSet::from_class(*class))
    })
}
