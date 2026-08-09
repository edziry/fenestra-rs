mod support;

use fenestra_ui_ir::prototype::{
    ChildFactory, ChildSlot, ComponentTypeId, InitialProperty, InvalidationClass, InvalidationSet,
    PropertyId, PropertyValue, SchemaNamespace, SourceId, SourceSpan, StructuralRegionId,
    TemplateNodeId, validate_construction, validate_schema,
};
use support::{
    COMPONENT, PROPERTY, REGION, ROOT, SCHEMA_NAMESPACE, SCHEMA_REVISION, basic_manifest,
    basic_program, component, key, manifest_with, node, program_with, property, region, span,
    validate_fixture,
};

#[test]
fn validates_an_immutable_ordered_construction_fixture() {
    let second_component = ComponentTypeId::new(1);
    let static_child = TemplateNodeId::new(1);
    let first_repeat = TemplateNodeId::new(2);
    let second_repeat = TemplateNodeId::new(3);
    let second_region = StructuralRegionId::new(1);
    let manifest = manifest_with(
        fenestra_ui_ir::prototype::SUPPORTED_SCHEMA_FORMAT,
        SCHEMA_NAMESPACE,
        SCHEMA_REVISION,
        vec![
            component(
                COMPONENT,
                vec![support::scalar_property(PROPERTY, span(2))],
                span(1),
            ),
            component(
                second_component,
                vec![property(
                    PROPERTY,
                    fenestra_ui_ir::prototype::ValueType::Rgba8,
                    PropertyValue::Rgba8([10, 20, 30, 255]),
                    InvalidationSet::from_class(InvalidationClass::Paint),
                    span(3),
                )],
                span(4),
            ),
        ],
        span(0),
    );
    let slots = vec![
        ChildSlot::static_node(static_child, span(11)),
        ChildSlot::region(REGION, span(12)),
        ChildSlot::region(second_region, span(13)),
    ];
    let program = program_with(
        support::SUPPORTED_CONSTRUCTION_FORMAT,
        SCHEMA_NAMESPACE,
        SCHEMA_REVISION,
        vec![
            node(ROOT, COMPONENT, Vec::new(), slots, span(10)),
            node(
                static_child,
                second_component,
                vec![InitialProperty::new(
                    PROPERTY,
                    PropertyValue::Rgba8([1, 2, 3, 255]),
                    span(14),
                )],
                Vec::new(),
                span(15),
            ),
            node(
                first_repeat,
                second_component,
                Vec::new(),
                Vec::new(),
                span(16),
            ),
            node(
                second_repeat,
                second_component,
                Vec::new(),
                Vec::new(),
                span(17),
            ),
        ],
        vec![
            region(
                REGION,
                ROOT,
                first_repeat,
                vec![key(5, span(18)), key(8, span(19))],
                span(20),
            ),
            region(second_region, ROOT, second_repeat, Vec::new(), span(21)),
        ],
        span(9),
    );

    let validated = validate_fixture(manifest, program).expect("fixture should validate");

    assert_eq!(validated.root_factory().id(), ROOT);
    let children = validated.root_factory().children().collect::<Vec<_>>();
    assert_eq!(children.len(), 3);
    assert!(matches!(
        children[0],
        ChildFactory::Static { template, .. } if template.id() == static_child
    ));
    assert!(matches!(
        children[1],
        ChildFactory::Region { region, .. } if region.id() == REGION
    ));
    assert!(matches!(
        children[2],
        ChildFactory::Region { region, .. } if region.id() == second_region
    ));
    assert_eq!(
        validated
            .region(REGION)
            .expect("first region should resolve")
            .initial_keys()
            .map(|key| key.value())
            .collect::<Vec<_>>(),
        vec![5, 8]
    );
    assert_eq!(
        validated
            .region(second_region)
            .expect("second region should resolve")
            .initial_keys()
            .count(),
        0
    );
    assert_eq!(
        validated
            .schema()
            .component(second_component)
            .expect("component should resolve")
            .property(PROPERTY)
            .expect("property should resolve")
            .id(),
        PROPERTY
    );
}

#[test]
fn invalidation_union_and_iteration_are_deterministic() {
    let all = [
        InvalidationClass::Structure,
        InvalidationClass::StyleMatch,
        InvalidationClass::Intrinsic,
        InvalidationClass::Layout,
        InvalidationClass::Semantics,
        InvalidationClass::HitTest,
        InvalidationClass::Paint,
        InvalidationClass::Composition,
        InvalidationClass::Surface,
    ];
    let set = all.iter().rev().fold(InvalidationSet::NONE, |set, class| {
        set.union(InvalidationSet::from_class(*class))
    });

    assert_eq!(set.iter().collect::<Vec<_>>(), all);
    assert!(all.into_iter().all(|class| set.contains(class)));
    assert!(!set.is_empty());
}

#[test]
fn local_ids_are_scoped_by_their_owning_records() {
    let first = validate_fixture(basic_manifest(), basic_program()).expect("first fixture");
    let second = validate_fixture(basic_manifest(), basic_program()).expect("second fixture");

    assert_eq!(
        first
            .schema()
            .component(COMPONENT)
            .expect("component should resolve")
            .property(PROPERTY)
            .expect("property should resolve")
            .id(),
        PropertyId::new(0)
    );
    assert_eq!(first.root_factory().id(), TemplateNodeId::new(0));
    assert_eq!(
        second
            .region(StructuralRegionId::new(0))
            .expect("region should resolve")
            .id(),
        StructuralRegionId::new(0)
    );
    assert_eq!(SourceSpan::synthetic(), SourceSpan::Synthetic);
    assert_eq!(SourceId::new(0).get(), 0);
    assert!(!first.schema().shares_domain_with(second.schema()));
    assert!(first.shares_domain_with(&first.clone()));
}

#[test]
fn construction_retains_the_exact_validated_schema_domain() {
    let schema =
        validate_schema(basic_manifest(), support::TEST_LIMITS).expect("schema should validate");
    let construction = validate_construction(&schema, basic_program(), support::TEST_LIMITS)
        .expect("construction should validate");
    let other = validate_schema(basic_manifest(), support::TEST_LIMITS)
        .expect("second schema should validate");

    assert!(construction.schema().shares_domain_with(&schema));
    assert!(schema.shares_domain_with(&schema.clone()));
    assert!(!construction.schema().shares_domain_with(&other));
}

#[test]
fn sparse_maximum_ids_do_not_define_storage_size() {
    let maximum = u32::MAX;
    let component_id = ComponentTypeId::new(maximum);
    let property_id = PropertyId::new(maximum);
    let root_id = TemplateNodeId::new(maximum);
    let source = SourceSpan::bytes(SourceId::new(maximum), maximum, maximum);
    let manifest = manifest_with(
        fenestra_ui_ir::prototype::SUPPORTED_SCHEMA_FORMAT,
        SchemaNamespace::new(u64::MAX),
        support::SCHEMA_REVISION,
        vec![component(
            component_id,
            vec![support::scalar_property(property_id, source)],
            source,
        )],
        source,
    );
    let program = program_with(
        support::SUPPORTED_CONSTRUCTION_FORMAT,
        SchemaNamespace::new(u64::MAX),
        support::SCHEMA_REVISION,
        vec![node(root_id, component_id, Vec::new(), Vec::new(), source)],
        Vec::new(),
        source,
    );

    let validated = validate_fixture(manifest, program).expect("sparse fixture should validate");

    assert_eq!(validated.root_factory().id(), root_id);
}
