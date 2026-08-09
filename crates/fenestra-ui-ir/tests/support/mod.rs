#![allow(dead_code)]

pub mod construction_faults;
mod limits;
pub mod malformed;

pub use fenestra_ui_ir::prototype::{SUPPORTED_CONSTRUCTION_FORMAT, SUPPORTED_SCHEMA_FORMAT};

use fenestra_ui_ir::prototype::{
    ChildSlot, ComponentSchema, ComponentTypeId, ConstructionFormatVersion, ConstructionProgram,
    InitialKey, InitialProperty, InputPolicy, InvalidationClass, InvalidationSet,
    IrValidationError, PropertyId, PropertySchema, PropertyValue, SchemaFormatVersion,
    SchemaManifest, SchemaNamespace, SchemaRevision, SourceId, SourceSpan, StructuralRegion,
    StructuralRegionId, TemplateNode, TemplateNodeId, ValidatedConstruction, ValidationLimits,
    ValueType, validate_construction, validate_schema,
};

pub const COMPONENT: ComponentTypeId = ComponentTypeId::new(0);
pub const PROPERTY: PropertyId = PropertyId::new(0);
pub const ROOT: TemplateNodeId = TemplateNodeId::new(0);
pub const REPEAT: TemplateNodeId = TemplateNodeId::new(1);
pub const REGION: StructuralRegionId = StructuralRegionId::new(0);
pub const SCHEMA_NAMESPACE: SchemaNamespace = SchemaNamespace::new(0);
pub const SCHEMA_REVISION: SchemaRevision = SchemaRevision::new(0);
pub const TEST_LIMITS: ValidationLimits =
    ValidationLimits::new(64, 64, 64, 64, 64, 64, 64, 64, 256);

pub fn span(index: u32) -> SourceSpan {
    SourceSpan::bytes(SourceId::new(0), index * 10, index * 10 + 5)
}

pub fn property(
    id: PropertyId,
    value_type: ValueType,
    default: PropertyValue,
    invalidation: InvalidationSet,
    source: SourceSpan,
) -> PropertySchema {
    PropertySchema::new(id, value_type, default, invalidation, source)
}

pub fn scalar_property(id: PropertyId, source: SourceSpan) -> PropertySchema {
    property(
        id,
        ValueType::ScalarI32,
        PropertyValue::ScalarI32(0),
        InvalidationSet::from_class(InvalidationClass::Layout),
        source,
    )
}

pub fn component(
    id: ComponentTypeId,
    properties: Vec<PropertySchema>,
    source: SourceSpan,
) -> ComponentSchema {
    ComponentSchema::new(id, properties, source)
}

pub fn manifest_with(
    format: SchemaFormatVersion,
    namespace: SchemaNamespace,
    revision: SchemaRevision,
    components: Vec<ComponentSchema>,
    source: SourceSpan,
) -> SchemaManifest {
    SchemaManifest::new(format, namespace, revision, components, source)
}

pub fn basic_manifest() -> SchemaManifest {
    manifest_with(
        SUPPORTED_SCHEMA_FORMAT,
        SCHEMA_NAMESPACE,
        SCHEMA_REVISION,
        vec![component(
            COMPONENT,
            vec![scalar_property(PROPERTY, span(2))],
            span(1),
        )],
        span(0),
    )
}

pub fn initial_property(
    property: PropertyId,
    value: PropertyValue,
    source: SourceSpan,
) -> InitialProperty {
    InitialProperty::new(property, value, source)
}

pub fn node(
    id: TemplateNodeId,
    component: ComponentTypeId,
    properties: Vec<InitialProperty>,
    children: Vec<ChildSlot>,
    source: SourceSpan,
) -> TemplateNode {
    TemplateNode::new(id, component, properties, children, source)
}

pub fn root(children: Vec<ChildSlot>) -> TemplateNode {
    node(ROOT, COMPONENT, Vec::new(), children, span(5))
}

pub fn repeat(properties: Vec<InitialProperty>) -> TemplateNode {
    node(REPEAT, COMPONENT, properties, Vec::new(), span(6))
}

pub fn region(
    id: StructuralRegionId,
    owner: TemplateNodeId,
    repeat_body: TemplateNodeId,
    keys: Vec<InitialKey>,
    source: SourceSpan,
) -> StructuralRegion {
    region_with_invalidation(
        id,
        owner,
        repeat_body,
        keys,
        InvalidationSet::from_class(InvalidationClass::Structure)
            .union(InvalidationSet::from_class(InvalidationClass::Layout))
            .union(InvalidationSet::from_class(InvalidationClass::Paint)),
        source,
    )
}

pub fn region_with_invalidation(
    id: StructuralRegionId,
    owner: TemplateNodeId,
    repeat_body: TemplateNodeId,
    keys: Vec<InitialKey>,
    invalidation: InvalidationSet,
    source: SourceSpan,
) -> StructuralRegion {
    StructuralRegion::new(id, owner, repeat_body, keys, invalidation, source)
}

pub fn basic_region(keys: Vec<InitialKey>) -> StructuralRegion {
    region(REGION, ROOT, REPEAT, keys, span(8))
}

pub fn key(value: u64, source: SourceSpan) -> InitialKey {
    InitialKey::new(value, source)
}

pub fn program_with(
    format: ConstructionFormatVersion,
    namespace: SchemaNamespace,
    revision: SchemaRevision,
    nodes: Vec<TemplateNode>,
    regions: Vec<StructuralRegion>,
    source: SourceSpan,
) -> ConstructionProgram {
    ConstructionProgram::new(format, namespace, revision, nodes, regions, source)
}

pub fn basic_program() -> ConstructionProgram {
    program_with(
        SUPPORTED_CONSTRUCTION_FORMAT,
        SCHEMA_NAMESPACE,
        SCHEMA_REVISION,
        vec![
            root(vec![ChildSlot::region(REGION, span(7))]),
            repeat(vec![initial_property(
                PROPERTY,
                PropertyValue::ScalarI32(10),
                span(9),
            )]),
        ],
        vec![basic_region(vec![key(7, span(10))])],
        span(4),
    )
}

pub fn validate_fixture(
    manifest: SchemaManifest,
    program: ConstructionProgram,
) -> Result<ValidatedConstruction, IrValidationError> {
    let schema = validate_schema(manifest, TEST_LIMITS)?;
    validate_construction(&schema, program, TEST_LIMITS)
}

pub fn input_policy_property(source: SourceSpan) -> PropertySchema {
    property(
        PropertyId::new(1),
        ValueType::InputPolicy,
        PropertyValue::InputPolicy(InputPolicy::Accept),
        InvalidationSet::from_class(InvalidationClass::HitTest),
        source,
    )
}
