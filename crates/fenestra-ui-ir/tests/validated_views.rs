mod support;

use fenestra_ui_ir::prototype::{
    ChildSlot, InputPolicy, InvalidationClass, InvalidationSet, PropertyId, PropertyValue,
    TemplateNodeId, ValueType,
};
use support::{
    COMPONENT, PROPERTY, REGION, REPEAT, ROOT, SCHEMA_NAMESPACE, SCHEMA_REVISION, basic_manifest,
    basic_program, component, initial_property, manifest_with, node, program_with, property, span,
    validate_fixture,
};

#[test]
fn validated_views_expose_runtime_construction_inputs() {
    let validated =
        validate_fixture(basic_manifest(), basic_program()).expect("fixture should validate");
    let root = validated.root_factory();
    let region = validated.region(REGION).expect("region should resolve");
    let repeat = region.repeat_body();

    assert_eq!(root.component().id(), COMPONENT);
    assert_eq!(
        validated
            .template(ROOT)
            .expect("root template should resolve")
            .id(),
        ROOT
    );
    assert!(validated.template(TemplateNodeId::new(u32::MAX)).is_none());
    assert_eq!(region.owner().id(), ROOT);
    assert_eq!(repeat.id(), REPEAT);
    assert_eq!(
        root.component()
            .properties()
            .map(|item| item.id())
            .collect::<Vec<_>>(),
        vec![PROPERTY]
    );
    assert_eq!(
        root.effective_value(PROPERTY),
        Some(&PropertyValue::ScalarI32(0))
    );
    assert_eq!(
        repeat.effective_value(PROPERTY),
        Some(&PropertyValue::ScalarI32(10))
    );

    let assignments = repeat.initial_properties().collect::<Vec<_>>();
    assert_eq!(assignments.len(), 1);
    assert_eq!(assignments[0].property().id(), PROPERTY);
    assert_eq!(assignments[0].value(), &PropertyValue::ScalarI32(10));
    assert!(
        root.component()
            .property(PROPERTY)
            .expect("property should resolve")
            .invalidation()
            .contains(InvalidationClass::Layout)
    );
    assert!(region.invalidation().contains(InvalidationClass::Structure));
}

#[test]
fn every_closed_value_variant_survives_validation() {
    let bool_id = PropertyId::new(1);
    let rgba_id = PropertyId::new(2);
    let input_id = PropertyId::new(3);
    let manifest = manifest_with(
        support::SUPPORTED_SCHEMA_FORMAT,
        SCHEMA_NAMESPACE,
        SCHEMA_REVISION,
        vec![component(
            COMPONENT,
            vec![
                support::scalar_property(PROPERTY, span(1)),
                property(
                    bool_id,
                    ValueType::Bool,
                    PropertyValue::Bool(false),
                    InvalidationSet::from_class(InvalidationClass::Semantics),
                    span(2),
                ),
                property(
                    rgba_id,
                    ValueType::Rgba8,
                    PropertyValue::Rgba8([0, 0, 0, 0]),
                    InvalidationSet::from_class(InvalidationClass::Paint),
                    span(3),
                ),
                property(
                    input_id,
                    ValueType::InputPolicy,
                    PropertyValue::InputPolicy(InputPolicy::Accept),
                    InvalidationSet::from_class(InvalidationClass::HitTest),
                    span(4),
                ),
            ],
            span(5),
        )],
        span(0),
    );
    let program = program_with(
        support::SUPPORTED_CONSTRUCTION_FORMAT,
        SCHEMA_NAMESPACE,
        SCHEMA_REVISION,
        vec![node(
            ROOT,
            COMPONENT,
            vec![
                initial_property(bool_id, PropertyValue::Bool(true), span(7)),
                initial_property(rgba_id, PropertyValue::Rgba8([1, 2, 3, 4]), span(8)),
                initial_property(
                    input_id,
                    PropertyValue::InputPolicy(InputPolicy::Ignore),
                    span(9),
                ),
            ],
            Vec::<ChildSlot>::new(),
            span(6),
        )],
        Vec::new(),
        span(10),
    );

    let validated = validate_fixture(manifest, program).expect("vocabulary should validate");
    let root = validated.root_factory();

    assert_eq!(
        root.effective_value(bool_id),
        Some(&PropertyValue::Bool(true))
    );
    assert_eq!(
        root.effective_value(rgba_id),
        Some(&PropertyValue::Rgba8([1, 2, 3, 4]))
    );
    assert_eq!(
        root.effective_value(input_id),
        Some(&PropertyValue::InputPolicy(InputPolicy::Ignore))
    );
}

#[test]
fn repeated_validation_creates_a_new_construction_domain() {
    let schema = fenestra_ui_ir::prototype::validate_schema(basic_manifest(), support::TEST_LIMITS)
        .expect("schema should validate");
    let first = fenestra_ui_ir::prototype::validate_construction(
        &schema,
        basic_program(),
        support::TEST_LIMITS,
    )
    .expect("first construction should validate");
    let second = fenestra_ui_ir::prototype::validate_construction(
        &schema,
        basic_program(),
        support::TEST_LIMITS,
    )
    .expect("second construction should validate");

    assert!(first.schema().shares_domain_with(second.schema()));
    assert!(!first.shares_domain_with(&second));
    assert!(first.shares_domain_with(&first.clone()));
}
