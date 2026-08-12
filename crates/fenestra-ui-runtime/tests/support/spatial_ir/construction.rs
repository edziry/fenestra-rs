use fenestra_ui_ir::prototype::{
    ChildSlot, ComponentSchema, ComponentTypeId, ConstructionProgram, InitialKey, InitialProperty,
    InvalidationClass, InvalidationSet, PropertySchema, PropertyValue,
    SUPPORTED_CONSTRUCTION_FORMAT, SUPPORTED_SCHEMA_FORMAT, SUPPORTED_STYLE_FORMAT, SchemaManifest,
    SchemaNamespace, SchemaRevision, StructuralRegion, StyleAssignment, StyleProgram,
    StyleValidationLimits, TemplateNode, ValidatedStyleProgram, ValidationLimits, ValueType,
    validate_construction, validate_schema, validate_style,
};

use super::{
    COLOR, EMPTY, EMPTY_REGION, FIRST_KEY, INNER, INNER_KEY, INNER_REGION, LEAF, NODE_ANCHOR,
    OUTER, OUTER_REGION, POLICY, ROOT, SECOND_KEY, VIEW_ANCHOR, WIDTH, span,
};

const COMPONENT: ComponentTypeId = ComponentTypeId::new(0);
const NAMESPACE: SchemaNamespace = SchemaNamespace::new(9001);
const REVISION: SchemaRevision = SchemaRevision::new(2);

pub(super) fn style(
    width: PropertyValue,
    color: PropertyValue,
    policy: PropertyValue,
) -> ValidatedStyleProgram {
    let schema = validate_schema(
        schema(),
        ValidationLimits::new(8, 16, 16, 16, 32, 8, 16, 16, 64),
    )
    .expect("runtime spatial IR schema should validate");
    let construction = validate_construction(
        &schema,
        construction(),
        ValidationLimits::new(8, 16, 16, 16, 32, 8, 16, 16, 64),
    )
    .expect("runtime spatial IR construction should validate");
    validate_style(
        &construction,
        StyleProgram::new(
            SUPPORTED_STYLE_FORMAT,
            NAMESPACE,
            REVISION,
            vec![
                StyleAssignment::new(OUTER, WIDTH, width, span(60)),
                StyleAssignment::new(OUTER, COLOR, color, span(61)),
                StyleAssignment::new(OUTER, POLICY, policy, span(62)),
            ],
            span(63),
        ),
        StyleValidationLimits::new(3),
    )
    .expect("runtime spatial IR style should validate")
}

fn schema() -> SchemaManifest {
    SchemaManifest::new(
        SUPPORTED_SCHEMA_FORMAT,
        NAMESPACE,
        REVISION,
        vec![ComponentSchema::new(
            COMPONENT,
            vec![
                PropertySchema::new(
                    WIDTH,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(8),
                    invalidation(InvalidationClass::Layout),
                    span(2),
                ),
                PropertySchema::new(
                    COLOR,
                    ValueType::Rgba8,
                    PropertyValue::Rgba8([5, 10, 15, 255]),
                    invalidation(InvalidationClass::Paint),
                    span(3),
                ),
                PropertySchema::new(
                    POLICY,
                    ValueType::InputPolicy,
                    PropertyValue::InputPolicy(fenestra_ui_ir::prototype::InputPolicy::Ignore),
                    invalidation(InvalidationClass::HitTest),
                    span(4),
                ),
            ],
            span(1),
        )],
        span(0),
    )
}

fn construction() -> ConstructionProgram {
    ConstructionProgram::new(
        SUPPORTED_CONSTRUCTION_FORMAT,
        NAMESPACE,
        REVISION,
        vec![
            template(
                ROOT,
                vec![
                    initial(WIDTH, PropertyValue::ScalarI32(40), 11),
                    initial(COLOR, PropertyValue::Rgba8([1, 1, 1, 255]), 12),
                ],
                vec![
                    ChildSlot::region(OUTER_REGION, span(13)),
                    ChildSlot::region(EMPTY_REGION, span(14)),
                ],
                10,
            ),
            template(
                OUTER,
                vec![
                    initial(WIDTH, PropertyValue::ScalarI32(10), 21),
                    initial(COLOR, PropertyValue::Rgba8([2, 3, 4, 255]), 22),
                    initial(
                        POLICY,
                        PropertyValue::InputPolicy(fenestra_ui_ir::prototype::InputPolicy::Ignore),
                        23,
                    ),
                ],
                vec![ChildSlot::region(INNER_REGION, span(24))],
                20,
            ),
            template(
                INNER,
                vec![
                    initial(WIDTH, PropertyValue::ScalarI32(4), 31),
                    initial(COLOR, PropertyValue::Rgba8([7, 8, 9, 255]), 32),
                ],
                vec![ChildSlot::static_node(LEAF, span(33))],
                30,
            ),
            template(EMPTY, Vec::new(), Vec::new(), 40),
            template(
                LEAF,
                Vec::new(),
                vec![
                    ChildSlot::static_node(NODE_ANCHOR, span(42)),
                    ChildSlot::static_node(VIEW_ANCHOR, span(43)),
                ],
                41,
            ),
            template(NODE_ANCHOR, Vec::new(), Vec::new(), 44),
            template(VIEW_ANCHOR, Vec::new(), Vec::new(), 45),
        ],
        vec![
            StructuralRegion::new(
                OUTER_REGION,
                ROOT,
                OUTER,
                vec![
                    InitialKey::new(FIRST_KEY, span(51)),
                    InitialKey::new(SECOND_KEY, span(52)),
                ],
                invalidation(InvalidationClass::Structure),
                span(50),
            ),
            StructuralRegion::new(
                INNER_REGION,
                OUTER,
                INNER,
                vec![InitialKey::new(INNER_KEY, span(54))],
                invalidation(InvalidationClass::Structure),
                span(53),
            ),
            StructuralRegion::new(
                EMPTY_REGION,
                ROOT,
                EMPTY,
                Vec::new(),
                invalidation(InvalidationClass::Structure),
                span(55),
            ),
        ],
        span(9),
    )
}

fn template(
    id: fenestra_ui_ir::prototype::TemplateNodeId,
    initial_properties: Vec<InitialProperty>,
    children: Vec<ChildSlot>,
    source: u32,
) -> TemplateNode {
    TemplateNode::new(id, COMPONENT, initial_properties, children, span(source))
}

fn initial(
    property: fenestra_ui_ir::prototype::PropertyId,
    value: PropertyValue,
    source: u32,
) -> InitialProperty {
    InitialProperty::new(property, value, span(source))
}

fn invalidation(class: InvalidationClass) -> InvalidationSet {
    InvalidationSet::from_class(class)
}
