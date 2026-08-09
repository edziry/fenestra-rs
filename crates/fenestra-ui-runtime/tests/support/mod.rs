#![allow(dead_code)]

pub mod headless;
pub mod headless_projection_state;
pub mod headless_spec;
pub mod model;
pub mod multiplicity;

use fenestra_ui_ir::prototype::{
    ChildSlot, ComponentSchema, ComponentTypeId, ConstructionProgram, InitialKey, InitialProperty,
    InvalidationClass, InvalidationSet, PropertyId, PropertySchema, PropertyValue,
    SUPPORTED_CONSTRUCTION_FORMAT, SUPPORTED_SCHEMA_FORMAT, SchemaManifest, SchemaNamespace,
    SchemaRevision, SourceSpan, StructuralRegion, StructuralRegionId, TemplateNode, TemplateNodeId,
    ValidatedConstruction, ValidationLimits, ValueType, validate_construction, validate_schema,
};
use fenestra_ui_runtime::prototype::RuntimeCapacity;

pub const PANEL: ComponentTypeId = ComponentTypeId::new(0);
pub const ITEM: ComponentTypeId = ComponentTypeId::new(1);

pub const WIDTH: PropertyId = PropertyId::new(0);
pub const VISIBLE: PropertyId = PropertyId::new(1);
pub const VALUE: PropertyId = PropertyId::new(0);

pub const ROOT: TemplateNodeId = TemplateNodeId::new(0);
pub const STATIC_CHILD: TemplateNodeId = TemplateNodeId::new(1);
pub const ITEM_BODY: TemplateNodeId = TemplateNodeId::new(2);
pub const EMPTY_BODY: TemplateNodeId = TemplateNodeId::new(3);
pub const NESTED_BODY: TemplateNodeId = TemplateNodeId::new(4);

pub const LIST: StructuralRegionId = StructuralRegionId::new(0);
pub const EMPTY_LIST: StructuralRegionId = StructuralRegionId::new(1);
pub const NESTED_LIST: StructuralRegionId = StructuralRegionId::new(2);

pub const KEY: u64 = 7;
pub const SECOND_KEY: u64 = 8;
pub const NESTED_KEY: u64 = 70;

pub fn layout() -> InvalidationSet {
    InvalidationSet::from_class(InvalidationClass::Layout)
}

pub fn paint() -> InvalidationSet {
    InvalidationSet::from_class(InvalidationClass::Paint)
}

pub fn structure() -> InvalidationSet {
    InvalidationSet::from_class(InvalidationClass::Structure)
}

pub fn structure_and_layout() -> InvalidationSet {
    structure().union(layout())
}

pub fn capacity() -> RuntimeCapacity {
    RuntimeCapacity::new(32, 64, 32, 32, 64, 4)
}

pub fn construction() -> ValidatedConstruction {
    let synthetic = SourceSpan::synthetic();
    let namespace = SchemaNamespace::new(41);
    let revision = SchemaRevision::new(3);
    let manifest = SchemaManifest::new(
        SUPPORTED_SCHEMA_FORMAT,
        namespace,
        revision,
        vec![
            ComponentSchema::new(
                PANEL,
                vec![
                    PropertySchema::new(
                        WIDTH,
                        ValueType::ScalarI32,
                        PropertyValue::ScalarI32(100),
                        layout(),
                        synthetic,
                    ),
                    PropertySchema::new(
                        VISIBLE,
                        ValueType::Bool,
                        PropertyValue::Bool(true),
                        paint(),
                        synthetic,
                    ),
                ],
                synthetic,
            ),
            ComponentSchema::new(
                ITEM,
                vec![PropertySchema::new(
                    VALUE,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(0),
                    layout().union(paint()),
                    synthetic,
                )],
                synthetic,
            ),
        ],
        synthetic,
    );
    let program = ConstructionProgram::new(
        SUPPORTED_CONSTRUCTION_FORMAT,
        namespace,
        revision,
        vec![
            TemplateNode::new(
                ROOT,
                PANEL,
                vec![InitialProperty::new(
                    WIDTH,
                    PropertyValue::ScalarI32(120),
                    synthetic,
                )],
                vec![
                    ChildSlot::static_node(STATIC_CHILD, synthetic),
                    ChildSlot::region(LIST, synthetic),
                    ChildSlot::region(EMPTY_LIST, synthetic),
                ],
                synthetic,
            ),
            TemplateNode::new(STATIC_CHILD, PANEL, Vec::new(), Vec::new(), synthetic),
            TemplateNode::new(
                ITEM_BODY,
                ITEM,
                vec![InitialProperty::new(
                    VALUE,
                    PropertyValue::ScalarI32(10),
                    synthetic,
                )],
                vec![ChildSlot::region(NESTED_LIST, synthetic)],
                synthetic,
            ),
            TemplateNode::new(EMPTY_BODY, ITEM, Vec::new(), Vec::new(), synthetic),
            TemplateNode::new(
                NESTED_BODY,
                ITEM,
                vec![InitialProperty::new(
                    VALUE,
                    PropertyValue::ScalarI32(30),
                    synthetic,
                )],
                Vec::new(),
                synthetic,
            ),
        ],
        vec![
            StructuralRegion::new(
                LIST,
                ROOT,
                ITEM_BODY,
                vec![
                    InitialKey::new(KEY, synthetic),
                    InitialKey::new(SECOND_KEY, synthetic),
                ],
                structure_and_layout(),
                synthetic,
            ),
            StructuralRegion::new(
                EMPTY_LIST,
                ROOT,
                EMPTY_BODY,
                Vec::new(),
                structure().union(paint()),
                synthetic,
            ),
            StructuralRegion::new(
                NESTED_LIST,
                ITEM_BODY,
                NESTED_BODY,
                vec![InitialKey::new(NESTED_KEY, synthetic)],
                structure(),
                synthetic,
            ),
        ],
        synthetic,
    );
    let limits = ValidationLimits::new(8, 16, 16, 16, 32, 16, 16, 16, 64);
    let schema = validate_schema(manifest, limits).expect("fixture schema should validate");
    validate_construction(&schema, program, limits).expect("fixture program should validate")
}
