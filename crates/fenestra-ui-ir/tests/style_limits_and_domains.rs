#[path = "support/style.rs"]
mod style_support;
mod support;

use fenestra_ui_ir::prototype::{
    ChildSlot, ComponentSchema, ComponentTypeId, ConstructionProgram, InvalidationClass,
    InvalidationSet, IrValidationErrorKind, PropertyId, PropertySchema, PropertyValue,
    SUPPORTED_CONSTRUCTION_FORMAT, SUPPORTED_SCHEMA_FORMAT, SUPPORTED_STYLE_FORMAT, SchemaManifest,
    SchemaNamespace, SchemaRevision, SourceId, SourceSpan, StyleAssignment, StyleProgram,
    TemplateNode, TemplateNodeId, ValidationLimitKind, ValidationLimits, ValueType,
    validate_construction, validate_schema, validate_style,
};

use style_support::{construction, program, scalar_assignment, style_limits, validate_program};
use support::{REPEAT, ROOT, span};

#[test]
fn assignment_limit_is_inclusive_and_precedes_the_crossing_span() {
    let construction = construction();
    let exact = program(vec![
        scalar_assignment(ROOT, 20, span(40)),
        scalar_assignment(REPEAT, 30, span(41)),
    ]);
    validate_program(&construction, exact, 2).expect("exact assignment limit should validate");

    let invalid = SourceSpan::bytes(SourceId::new(0), 9, 4);
    let crossing = program(vec![
        scalar_assignment(ROOT, 20, span(42)),
        scalar_assignment(REPEAT, 30, invalid),
    ]);
    let error = validate_program(&construction, crossing, 1)
        .expect_err("count preflight should reject the crossing assignment");
    assert_eq!(
        error.kind(),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::StyleAssignments)
    );
    assert_eq!(error.span(), invalid);

    validate_program(&construction, program(Vec::new()), 0)
        .expect("zero should accept an empty style program");
}

#[test]
fn style_domains_share_only_through_clones_and_retain_the_exact_construction() {
    let construction = construction();
    let first = validate_program(&construction, style_support::basic_style_program(), 1)
        .expect("first style should validate");
    let clone = first.clone();
    let repeated = validate_program(&construction, style_support::basic_style_program(), 1)
        .expect("repeated style should validate");
    let other_construction = style_support::construction();
    let other = validate_program(&other_construction, style_support::basic_style_program(), 1)
        .expect("style against other construction should validate");

    assert!(first.shares_domain_with(&clone));
    assert!(!first.shares_domain_with(&repeated));
    assert!(first.construction().shares_domain_with(&construction));
    assert!(!first.construction().shares_domain_with(&other_construction));
    assert!(other.construction().shares_domain_with(&other_construction));

    let rendered = format!("{first:?}");
    for forbidden in ["ScalarI32", "20", "validation_domain", "index"] {
        assert!(
            !rendered.contains(forbidden),
            "leaked {forbidden}: {rendered}"
        );
    }
}

#[test]
fn sparse_symbols_and_component_local_property_ids_resolve_without_collision() {
    let maximum = u32::MAX;
    let namespace = SchemaNamespace::new(u64::MAX);
    let revision = SchemaRevision::new(maximum);
    let first_component = ComponentTypeId::new(0);
    let second_component = ComponentTypeId::new(maximum);
    let property = PropertyId::new(maximum);
    let root = TemplateNodeId::new(0);
    let child = TemplateNodeId::new(maximum);
    let source = SourceSpan::bytes(SourceId::new(maximum), maximum, maximum);
    let layout = InvalidationSet::from_class(InvalidationClass::Layout);
    let paint = InvalidationSet::from_class(InvalidationClass::Paint);
    let manifest = SchemaManifest::new(
        SUPPORTED_SCHEMA_FORMAT,
        namespace,
        revision,
        vec![
            ComponentSchema::new(
                first_component,
                vec![PropertySchema::new(
                    property,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(1),
                    layout,
                    source,
                )],
                source,
            ),
            ComponentSchema::new(
                second_component,
                vec![PropertySchema::new(
                    property,
                    ValueType::Rgba8,
                    PropertyValue::Rgba8([0, 0, 0, 0]),
                    paint,
                    source,
                )],
                source,
            ),
        ],
        source,
    );
    let construction_program = ConstructionProgram::new(
        SUPPORTED_CONSTRUCTION_FORMAT,
        namespace,
        revision,
        vec![
            TemplateNode::new(
                root,
                first_component,
                Vec::new(),
                vec![ChildSlot::static_node(child, source)],
                source,
            ),
            TemplateNode::new(child, second_component, Vec::new(), Vec::new(), source),
        ],
        Vec::new(),
        source,
    );
    let limits = ValidationLimits::new(2, 2, 2, 0, 1, 0, 0, 2, 2);
    let schema = validate_schema(manifest, limits).expect("sparse schema should validate");
    let construction = validate_construction(&schema, construction_program, limits)
        .expect("sparse construction should validate");
    let style_program = StyleProgram::new(
        SUPPORTED_STYLE_FORMAT,
        namespace,
        revision,
        vec![
            StyleAssignment::new(root, property, PropertyValue::ScalarI32(7), source),
            StyleAssignment::new(child, property, PropertyValue::Rgba8([1, 2, 3, 4]), source),
        ],
        source,
    );
    let style = validate_style(&construction, style_program, style_limits(2))
        .expect("sparse style should validate");

    let assignments = style.assignments().collect::<Vec<_>>();
    assert_eq!(assignments[0].target().component().id(), first_component);
    assert_eq!(assignments[1].target().component().id(), second_component);
    assert_eq!(assignments[0].property().id(), property);
    assert_eq!(assignments[1].property().id(), property);
}
