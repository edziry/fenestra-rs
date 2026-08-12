use fenestra_ui_ir::prototype::{
    ChildSlot, ComponentSchema, ComponentTypeId, ConstructionProgram, InitialKey, InitialProperty,
    InputPolicy, InvalidationClass, InvalidationSet, PropertyId, PropertySchema, PropertyValue,
    SUPPORTED_CONSTRUCTION_FORMAT, SUPPORTED_SCHEMA_FORMAT, SUPPORTED_STYLE_FORMAT, SchemaManifest,
    SchemaNamespace, SchemaRevision, StructuralRegion, StructuralRegionId, StyleAssignment,
    StyleProgram, TemplateNode, TemplateNodeId, ValueType,
};

use super::value::span;

const NAMESPACE: SchemaNamespace = SchemaNamespace::new(13_013);
const REVISION: SchemaRevision = SchemaRevision::new(2);

pub(super) fn schema() -> SchemaManifest {
    SchemaManifest::new(
        SUPPORTED_SCHEMA_FORMAT,
        NAMESPACE,
        REVISION,
        vec![ComponentSchema::new(
            ComponentTypeId::new(0),
            vec![
                property(
                    0,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(80),
                    layout_all(),
                    3,
                ),
                property(
                    1,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(48),
                    layout_all(),
                    4,
                ),
                property(
                    2,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(2),
                    layout_visual(),
                    5,
                ),
                property(
                    3,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(1),
                    non_layout(),
                    6,
                ),
                property(
                    4,
                    ValueType::Rgba8,
                    PropertyValue::Rgba8([64, 48, 32, 255]),
                    paint(),
                    7,
                ),
                property(
                    5,
                    ValueType::Rgba8,
                    PropertyValue::Rgba8([16, 64, 96, 192]),
                    paint(),
                    8,
                ),
                property(6, ValueType::Bool, PropertyValue::Bool(true), visible(), 9),
                property(
                    7,
                    ValueType::InputPolicy,
                    PropertyValue::InputPolicy(InputPolicy::Ignore),
                    one(InvalidationClass::HitTest),
                    10,
                ),
            ],
            span(2),
        )],
        span(1),
    )
}

pub(super) fn construction() -> ConstructionProgram {
    ConstructionProgram::new(
        SUPPORTED_CONSTRUCTION_FORMAT,
        NAMESPACE,
        REVISION,
        vec![
            template_node(
                0,
                vec![
                    i32_initial(0, 180, 13),
                    i32_initial(1, 120, 14),
                    rgba_initial(4, [96, 72, 48, 255], 15),
                ],
                vec![
                    ChildSlot::static_node(TemplateNodeId::new(1), span(16)),
                    ChildSlot::static_node(TemplateNodeId::new(5), span(17)),
                ],
                12,
            ),
            template_node(
                1,
                vec![
                    i32_initial(0, 80, 19),
                    i32_initial(1, 60, 20),
                    i32_initial(3, 1, 21),
                ],
                vec![ChildSlot::static_node(TemplateNodeId::new(2), span(22))],
                18,
            ),
            template_node(
                2,
                vec![
                    i32_initial(0, 40, 24),
                    i32_initial(1, 30, 25),
                    i32_initial(3, 1, 26),
                ],
                vec![
                    ChildSlot::static_node(TemplateNodeId::new(3), span(27)),
                    ChildSlot::region(StructuralRegionId::new(0), span(28)),
                ],
                23,
            ),
            template_node(
                3,
                vec![i32_initial(0, 12, 30), i32_initial(1, 10, 31)],
                vec![],
                29,
            ),
            template_node(
                4,
                vec![
                    i32_initial(0, 16, 33),
                    i32_initial(1, 12, 34),
                    rgba_initial(4, [40, 80, 120, 192], 35),
                    InitialProperty::new(
                        PropertyId::new(7),
                        PropertyValue::InputPolicy(InputPolicy::Accept),
                        span(36),
                    ),
                ],
                vec![],
                32,
            ),
            template_node(
                5,
                vec![i32_initial(0, 50, 38), i32_initial(1, 40, 39)],
                vec![ChildSlot::static_node(TemplateNodeId::new(6), span(40))],
                37,
            ),
            template_node(
                6,
                vec![i32_initial(0, 20, 42), i32_initial(1, 16, 43)],
                vec![],
                41,
            ),
        ],
        vec![StructuralRegion::new(
            StructuralRegionId::new(0),
            TemplateNodeId::new(2),
            TemplateNodeId::new(4),
            vec![InitialKey::new(10, span(45)), InitialKey::new(20, span(46))],
            region_invalidation(),
            span(44),
        )],
        span(11),
    )
}

pub(super) fn style() -> StyleProgram {
    StyleProgram::new(
        SUPPORTED_STYLE_FORMAT,
        NAMESPACE,
        REVISION,
        vec![
            style_assignment(1, 4, PropertyValue::Rgba8([24, 48, 72, 255]), 48),
            style_assignment(4, 4, PropertyValue::Rgba8([80, 120, 160, 192]), 49),
            style_assignment(6, 7, PropertyValue::InputPolicy(InputPolicy::Accept), 50),
        ],
        span(47),
    )
}

fn property(
    id: u32,
    ty: ValueType,
    default: PropertyValue,
    invalidation: InvalidationSet,
    anchor: u32,
) -> PropertySchema {
    PropertySchema::new(PropertyId::new(id), ty, default, invalidation, span(anchor))
}

fn template_node(
    id: u32,
    initial: Vec<InitialProperty>,
    children: Vec<ChildSlot>,
    anchor: u32,
) -> TemplateNode {
    TemplateNode::new(
        TemplateNodeId::new(id),
        ComponentTypeId::new(0),
        initial,
        children,
        span(anchor),
    )
}

fn i32_initial(property: u32, value: i32, anchor: u32) -> InitialProperty {
    InitialProperty::new(
        PropertyId::new(property),
        PropertyValue::ScalarI32(value),
        span(anchor),
    )
}

fn rgba_initial(property: u32, value: [u8; 4], anchor: u32) -> InitialProperty {
    InitialProperty::new(
        PropertyId::new(property),
        PropertyValue::Rgba8(value),
        span(anchor),
    )
}

fn style_assignment(
    target: u32,
    property: u32,
    value: PropertyValue,
    anchor: u32,
) -> StyleAssignment {
    StyleAssignment::new(
        TemplateNodeId::new(target),
        PropertyId::new(property),
        value,
        span(anchor),
    )
}

const fn one(class: InvalidationClass) -> InvalidationSet {
    InvalidationSet::from_class(class)
}

const fn layout_all() -> InvalidationSet {
    one(InvalidationClass::Layout)
        .union(one(InvalidationClass::Semantics))
        .union(one(InvalidationClass::HitTest))
        .union(one(InvalidationClass::Paint))
        .union(one(InvalidationClass::Composition))
}

const fn layout_visual() -> InvalidationSet {
    one(InvalidationClass::Layout)
        .union(one(InvalidationClass::HitTest))
        .union(one(InvalidationClass::Paint))
        .union(one(InvalidationClass::Composition))
}

const fn non_layout() -> InvalidationSet {
    one(InvalidationClass::Semantics)
        .union(one(InvalidationClass::HitTest))
        .union(one(InvalidationClass::Paint))
        .union(one(InvalidationClass::Composition))
}

const fn visible() -> InvalidationSet {
    one(InvalidationClass::Semantics)
        .union(one(InvalidationClass::HitTest))
        .union(one(InvalidationClass::Paint))
}

const fn paint() -> InvalidationSet {
    one(InvalidationClass::Paint)
}

const fn region_invalidation() -> InvalidationSet {
    one(InvalidationClass::Structure)
        .union(one(InvalidationClass::Layout))
        .union(one(InvalidationClass::Semantics))
        .union(one(InvalidationClass::HitTest))
        .union(one(InvalidationClass::Paint))
        .union(one(InvalidationClass::Composition))
}
