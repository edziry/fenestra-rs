use fenestra_ui_ir::prototype::{
    ChildSlot, ComponentSchema, ConstructionProgram, InputPolicy, InvalidationClass,
    InvalidationSet, PropertySchema, PropertyValue, SUPPORTED_CONSTRUCTION_FORMAT,
    SUPPORTED_SCHEMA_FORMAT, SUPPORTED_STYLE_FORMAT, SchemaManifest, SchemaNamespace,
    SchemaRevision, SourceSpan, StyleProgram, StyleValidationLimits, TemplateNode, TemplateNodeId,
    ValidationLimits, ValueType, validate_construction, validate_schema, validate_style,
};
use fenestra_ui_runtime::prototype::{
    HeadlessProjectionCapacity, HeadlessProjectionSpec, HeadlessSurface, RuntimeCapacity, UiRuntime,
};

use super::headless::{COLOR, COMPONENT, HEIGHT, INPUT, ROOT, VISIBLE, WIDTH};

const NAMESPACE: SchemaNamespace = SchemaNamespace::new(8011);
const REVISION: SchemaRevision = SchemaRevision::new(1);

pub fn initialize(parents: &[Option<usize>]) -> UiRuntime {
    let node_count = parents.len();
    assert_ne!(node_count, 0, "large layout fixture requires a root");
    assert_eq!(parents[0], None, "large layout fixture root has no parent");
    let limits = ValidationLimits::new(
        1,
        5,
        node_count,
        0,
        node_count - 1,
        0,
        0,
        node_count,
        node_count,
    );
    let schema = validate_schema(schema(), limits).expect("large layout schema should validate");
    let construction = validate_construction(&schema, program(parents), limits)
        .expect("large layout construction should validate");
    let style = validate_style(
        &construction,
        StyleProgram::new(
            SUPPORTED_STYLE_FORMAT,
            NAMESPACE,
            REVISION,
            Vec::new(),
            SourceSpan::synthetic(),
        ),
        StyleValidationLimits::new(0),
    )
    .expect("large empty style should validate");
    let spec = HeadlessProjectionSpec::new(
        WIDTH,
        HEIGHT,
        COLOR,
        VISIBLE,
        INPUT,
        ROOT,
        1,
        HeadlessProjectionCapacity::new(node_count, node_count, 0, 0, 0),
    );
    UiRuntime::new_headless(
        style,
        spec,
        HeadlessSurface::new(0, 0),
        RuntimeCapacity::new(1, node_count, node_count, 0, node_count * 5, 2),
    )
    .expect("large reference runtime should initialize")
}

pub fn node_ceiling_parents() -> Vec<Option<usize>> {
    let mut parents = vec![None];
    parents.extend((1..=4).map(|_| Some(0)));
    parents.extend((5..33).map(|index| Some(1 + (index - 5) / 7)));
    parents
}

pub fn depth_ceiling_parents() -> Vec<Option<usize>> {
    let mut parents = vec![None];
    parents.extend((1..9).map(|index| Some(index - 1)));
    parents
}

pub fn children_ceiling_parents() -> Vec<Option<usize>> {
    let mut parents = vec![None];
    parents.extend((1..18).map(|_| Some(0)));
    parents
}

fn schema() -> SchemaManifest {
    let span = SourceSpan::synthetic();
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
                    PropertyValue::ScalarI32(100),
                    InvalidationClass::Layout,
                ),
                property(
                    HEIGHT,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(1),
                    InvalidationClass::Layout,
                ),
                property(
                    COLOR,
                    ValueType::Rgba8,
                    PropertyValue::Rgba8([1, 1, 1, 255]),
                    InvalidationClass::Paint,
                ),
                property(
                    VISIBLE,
                    ValueType::Bool,
                    PropertyValue::Bool(true),
                    InvalidationClass::Paint,
                ),
                property(
                    INPUT,
                    ValueType::InputPolicy,
                    PropertyValue::InputPolicy(InputPolicy::Ignore),
                    InvalidationClass::HitTest,
                ),
            ],
            span,
        )],
        span,
    )
}

fn property(
    id: fenestra_ui_ir::prototype::PropertyId,
    value_type: ValueType,
    default: PropertyValue,
    invalidation: InvalidationClass,
) -> PropertySchema {
    PropertySchema::new(
        id,
        value_type,
        default,
        InvalidationSet::from_class(invalidation),
        SourceSpan::synthetic(),
    )
}

fn program(parents: &[Option<usize>]) -> ConstructionProgram {
    let span = SourceSpan::synthetic();
    let mut children = vec![Vec::new(); parents.len()];
    for (index, parent) in parents.iter().copied().enumerate().skip(1) {
        let parent = parent.expect("every later large fixture node has a parent");
        assert!(
            parent < index,
            "large fixture parent must precede its child"
        );
        children[parent].push(ChildSlot::static_node(
            TemplateNodeId::new(index as u32),
            span,
        ));
    }
    ConstructionProgram::new(
        SUPPORTED_CONSTRUCTION_FORMAT,
        NAMESPACE,
        REVISION,
        children
            .into_iter()
            .enumerate()
            .map(|(index, children)| {
                TemplateNode::new(
                    TemplateNodeId::new(index as u32),
                    COMPONENT,
                    Vec::new(),
                    children,
                    span,
                )
            })
            .collect(),
        Vec::new(),
        span,
    )
}
