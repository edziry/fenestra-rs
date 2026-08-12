use fenestra_ui_ir::prototype::{
    ChildSlot, ComponentSchema, ComponentTypeId, ConstructionProgram, InitialKey, InitialProperty,
    InputPolicy, InvalidationClass, InvalidationSet, PropertyId, PropertySchema, PropertyValue,
    SUPPORTED_CONSTRUCTION_FORMAT, SUPPORTED_SCHEMA_FORMAT, SUPPORTED_STYLE_FORMAT, SchemaManifest,
    SchemaNamespace, SchemaRevision, StructuralRegion, StructuralRegionId, StyleProgram,
    TemplateNode, TemplateNodeId, ValueType,
};

use super::spatial;
use super::value::span;

const NAMESPACE: SchemaNamespace = SchemaNamespace::new(80_008);
const REVISION: SchemaRevision = SchemaRevision::new(2);

pub(super) fn raw() -> (
    SchemaManifest,
    ConstructionProgram,
    StyleProgram,
    fenestra_ui_ir::prototype::SpatialProgramV2,
) {
    (schema(), construction(), style(), spatial::program())
}

fn schema() -> SchemaManifest {
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
                ),
                property(
                    1,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(48),
                    layout_all(),
                ),
                property(
                    2,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(2),
                    layout_all(),
                ),
                property(
                    3,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(1),
                    one(InvalidationClass::Composition),
                ),
                property(
                    4,
                    ValueType::Rgba8,
                    PropertyValue::Rgba8([64, 48, 32, 255]),
                    one(InvalidationClass::Paint),
                ),
                property(
                    5,
                    ValueType::Rgba8,
                    PropertyValue::Rgba8([16, 64, 96, 192]),
                    one(InvalidationClass::Paint),
                ),
                property(
                    6,
                    ValueType::Bool,
                    PropertyValue::Bool(true),
                    one(InvalidationClass::Semantics),
                ),
                property(
                    7,
                    ValueType::InputPolicy,
                    PropertyValue::InputPolicy(InputPolicy::Ignore),
                    one(InvalidationClass::HitTest),
                ),
            ],
            span(2),
        )],
        span(1),
    )
}

fn construction() -> ConstructionProgram {
    ConstructionProgram::new(
        SUPPORTED_CONSTRUCTION_FORMAT,
        NAMESPACE,
        REVISION,
        vec![
            template(
                0,
                vec![
                    initial(0, PropertyValue::ScalarI32(180), 14),
                    initial(1, PropertyValue::ScalarI32(120), 15),
                    initial(4, PropertyValue::Rgba8([96, 72, 48, 255]), 16),
                ],
                vec![
                    ChildSlot::static_node(TemplateNodeId::new(1), span(17)),
                    ChildSlot::region(StructuralRegionId::new(0), span(18)),
                ],
                13,
            ),
            template(
                1,
                vec![
                    initial(0, PropertyValue::ScalarI32(40), 21),
                    initial(1, PropertyValue::ScalarI32(30), 22),
                ],
                Vec::new(),
                20,
            ),
            template(
                2,
                vec![
                    initial(0, PropertyValue::ScalarI32(16), 25),
                    initial(1, PropertyValue::ScalarI32(12), 26),
                    initial(4, PropertyValue::Rgba8([40, 80, 120, 192]), 27),
                    initial(7, PropertyValue::InputPolicy(InputPolicy::Accept), 28),
                ],
                Vec::new(),
                24,
            ),
        ],
        vec![StructuralRegion::new(
            StructuralRegionId::new(0),
            TemplateNodeId::new(0),
            TemplateNodeId::new(2),
            vec![InitialKey::new(10, span(31)), InitialKey::new(20, span(32))],
            region_invalidation(),
            span(30),
        )],
        span(12),
    )
}

fn style() -> StyleProgram {
    StyleProgram::new(
        SUPPORTED_STYLE_FORMAT,
        NAMESPACE,
        REVISION,
        Vec::new(),
        span(34),
    )
}

fn property(
    id: u32,
    ty: ValueType,
    default: PropertyValue,
    invalidation: InvalidationSet,
) -> PropertySchema {
    PropertySchema::new(PropertyId::new(id), ty, default, invalidation, span(3 + id))
}

fn template(
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

fn initial(property: u32, value: PropertyValue, anchor: u32) -> InitialProperty {
    InitialProperty::new(PropertyId::new(property), value, span(anchor))
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

const fn region_invalidation() -> InvalidationSet {
    one(InvalidationClass::Structure)
        .union(one(InvalidationClass::Layout))
        .union(one(InvalidationClass::Semantics))
        .union(one(InvalidationClass::HitTest))
        .union(one(InvalidationClass::Paint))
        .union(one(InvalidationClass::Composition))
}
