use fenestra_ui_ir::prototype::{
    ChildSlot, ComponentSchema, ComponentTypeId, ConstructionProgram, InitialKey, InitialProperty,
    InputPolicy, InvalidationClass, InvalidationSet, PropertyId, PropertySchema, PropertyValue,
    SUPPORTED_CONSTRUCTION_FORMAT, SUPPORTED_SCHEMA_FORMAT, SUPPORTED_STYLE_FORMAT, SchemaManifest,
    SchemaNamespace, SchemaRevision, SourceSpan, StructuralRegion, StructuralRegionId,
    StyleAssignment, StyleProgram, StyleValidationLimits, TemplateNode, TemplateNodeId,
    ValidatedConstruction, ValidatedStyleProgram, ValidationLimits, ValueType,
    validate_construction, validate_schema, validate_style,
};
use fenestra_ui_runtime::prototype::RuntimeCapacity;

pub const COMPONENT: ComponentTypeId = ComponentTypeId::new(0);

pub const WIDTH: PropertyId = PropertyId::new(0);
pub const HEIGHT: PropertyId = PropertyId::new(1);
pub const COLOR: PropertyId = PropertyId::new(2);
pub const VISIBLE: PropertyId = PropertyId::new(3);
pub const INPUT: PropertyId = PropertyId::new(4);

pub const ROOT: TemplateNodeId = TemplateNodeId::new(0);
pub const CONTAINER: TemplateNodeId = TemplateNodeId::new(1);
pub const CONTROL: TemplateNodeId = TemplateNodeId::new(2);
pub const ITEM: TemplateNodeId = TemplateNodeId::new(3);
pub const ITEMS: StructuralRegionId = StructuralRegionId::new(0);

pub const FIRST_KEY: u64 = 10;
pub const SECOND_KEY: u64 = 20;
pub const INSERTED_KEY: u64 = 30;
pub const SEMANTIC_LABEL: u32 = 1;

pub const SCHEMA_WIDTH: i32 = 40;
pub const ROOT_WIDTH: i32 = 100;
pub const SURFACE_WIDTH: i32 = 120;
pub const SURFACE_HEIGHT: i32 = 90;
pub const CONTROL_STYLE_COLOR: [u8; 4] = [10, 20, 30, 255];
pub const ITEM_STYLE_COLOR: [u8; 4] = [80, 90, 100, 255];
pub const DIRECT_COLOR: [u8; 4] = [20, 30, 40, 255];

const NAMESPACE: SchemaNamespace = SchemaNamespace::new(8001);
const REVISION: SchemaRevision = SchemaRevision::new(1);
const IR_LIMITS: ValidationLimits = ValidationLimits::new(1, 5, 4, 1, 3, 12, 2, 3, 5);

pub fn runtime_capacity() -> RuntimeCapacity {
    RuntimeCapacity::new(8, 8, 8, 2, 40, 3)
}

pub fn construction() -> ValidatedConstruction {
    let schema = validate_schema(schema(), IR_LIMITS).expect("headless schema should validate");
    validate_construction(&schema, program(), IR_LIMITS)
        .expect("headless construction should validate")
}

pub fn empty_style() -> ValidatedStyleProgram {
    validate_style_program(Vec::new(), 0)
}

pub fn exact_style() -> ValidatedStyleProgram {
    exact_style_with(Vec::new())
}

pub fn exact_style_with(
    additional: Vec<(TemplateNodeId, PropertyId, PropertyValue)>,
) -> ValidatedStyleProgram {
    let synthetic = SourceSpan::synthetic();
    let mut assignments = vec![
        StyleAssignment::new(
            CONTROL,
            COLOR,
            PropertyValue::Rgba8(CONTROL_STYLE_COLOR),
            synthetic,
        ),
        StyleAssignment::new(
            ITEM,
            COLOR,
            PropertyValue::Rgba8(ITEM_STYLE_COLOR),
            synthetic,
        ),
    ];
    assignments.extend(additional.into_iter().map(|(template, property, value)| {
        StyleAssignment::new(template, property, value, synthetic)
    }));
    let assignment_limit = assignments.len();
    validate_style_program(assignments, assignment_limit)
}

fn validate_style_program(
    assignments: Vec<StyleAssignment>,
    assignment_limit: usize,
) -> ValidatedStyleProgram {
    let construction = construction();
    let program = StyleProgram::new(
        SUPPORTED_STYLE_FORMAT,
        NAMESPACE,
        REVISION,
        assignments,
        SourceSpan::synthetic(),
    );
    validate_style(
        &construction,
        program,
        StyleValidationLimits::new(assignment_limit),
    )
    .expect("headless style should validate")
}

fn schema() -> SchemaManifest {
    let synthetic = SourceSpan::synthetic();
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
                    PropertyValue::ScalarI32(SCHEMA_WIDTH),
                    dimension_invalidation(),
                    synthetic,
                ),
                PropertySchema::new(
                    HEIGHT,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(10),
                    dimension_invalidation(),
                    synthetic,
                ),
                PropertySchema::new(
                    COLOR,
                    ValueType::Rgba8,
                    PropertyValue::Rgba8([32, 32, 32, 255]),
                    paint(),
                    synthetic,
                ),
                PropertySchema::new(
                    VISIBLE,
                    ValueType::Bool,
                    PropertyValue::Bool(true),
                    visibility_invalidation(),
                    synthetic,
                ),
                PropertySchema::new(
                    INPUT,
                    ValueType::InputPolicy,
                    PropertyValue::InputPolicy(InputPolicy::Ignore),
                    hit_test(),
                    synthetic,
                ),
            ],
            synthetic,
        )],
        synthetic,
    )
}

fn program() -> ConstructionProgram {
    let synthetic = SourceSpan::synthetic();
    ConstructionProgram::new(
        SUPPORTED_CONSTRUCTION_FORMAT,
        NAMESPACE,
        REVISION,
        vec![
            TemplateNode::new(
                ROOT,
                COMPONENT,
                vec![
                    initial(WIDTH, PropertyValue::ScalarI32(ROOT_WIDTH)),
                    initial(HEIGHT, PropertyValue::ScalarI32(80)),
                    initial(COLOR, PropertyValue::Rgba8([1, 1, 1, 255])),
                ],
                vec![ChildSlot::static_node(CONTAINER, synthetic)],
                synthetic,
            ),
            TemplateNode::new(
                CONTAINER,
                COMPONENT,
                vec![
                    initial(WIDTH, PropertyValue::ScalarI32(80)),
                    initial(HEIGHT, PropertyValue::ScalarI32(50)),
                    initial(COLOR, PropertyValue::Rgba8([2, 2, 2, 255])),
                ],
                vec![
                    ChildSlot::static_node(CONTROL, synthetic),
                    ChildSlot::region(ITEMS, synthetic),
                ],
                synthetic,
            ),
            TemplateNode::new(
                CONTROL,
                COMPONENT,
                vec![
                    initial(WIDTH, PropertyValue::ScalarI32(30)),
                    initial(COLOR, PropertyValue::Rgba8([3, 3, 3, 255])),
                    initial(INPUT, PropertyValue::InputPolicy(InputPolicy::Accept)),
                ],
                Vec::new(),
                synthetic,
            ),
            TemplateNode::new(
                ITEM,
                COMPONENT,
                vec![
                    initial(HEIGHT, PropertyValue::ScalarI32(12)),
                    initial(COLOR, PropertyValue::Rgba8([4, 4, 4, 255])),
                    initial(INPUT, PropertyValue::InputPolicy(InputPolicy::Accept)),
                ],
                Vec::new(),
                synthetic,
            ),
        ],
        vec![StructuralRegion::new(
            ITEMS,
            CONTAINER,
            ITEM,
            vec![
                InitialKey::new(FIRST_KEY, synthetic),
                InitialKey::new(SECOND_KEY, synthetic),
            ],
            region_invalidation(),
            synthetic,
        )],
        synthetic,
    )
}

fn initial(property: PropertyId, value: PropertyValue) -> InitialProperty {
    InitialProperty::new(property, value, SourceSpan::synthetic())
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

fn paint() -> InvalidationSet {
    InvalidationSet::from_class(InvalidationClass::Paint)
}

fn hit_test() -> InvalidationSet {
    InvalidationSet::from_class(InvalidationClass::HitTest)
}

fn invalidation(classes: &[InvalidationClass]) -> InvalidationSet {
    classes.iter().fold(InvalidationSet::NONE, |set, class| {
        set.union(InvalidationSet::from_class(*class))
    })
}
