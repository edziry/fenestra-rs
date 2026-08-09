mod support;

use fenestra_ui_ir::prototype::{
    ChildSlot, ComponentTypeId, InvalidationClass, InvalidationSet, IrValidationErrorKind,
    PropertyId, SchemaNamespace, SourceId, SourceSpan, StructuralRegionId, TemplateNodeId,
    ValidationLimitKind, ValidationLimits, validate_construction, validate_schema,
};
use support::{
    COMPONENT, SCHEMA_REVISION, basic_manifest, basic_program, component, key, manifest_with, node,
    program_with, region_with_invalidation, scalar_property, span,
};

#[test]
fn every_limit_is_inclusive_at_the_exact_basic_fixture_size() {
    let limits = ValidationLimits::new(1, 1, 2, 1, 1, 1, 1, 2, 2);
    let schema = validate_schema(basic_manifest(), limits).expect("schema should fit limits");

    validate_construction(&schema, basic_program(), limits)
        .expect("construction should fit exact limits");
}

#[test]
fn count_limit_preflight_wins_over_an_invalid_crossing_record_span() {
    let invalid = SourceSpan::bytes(SourceId::new(0), 9, 4);
    let manifest = manifest_with(
        fenestra_ui_ir::prototype::SUPPORTED_SCHEMA_FORMAT,
        support::SCHEMA_NAMESPACE,
        SCHEMA_REVISION,
        vec![component(
            COMPONENT,
            vec![scalar_property(PropertyId::new(0), span(2))],
            invalid,
        )],
        span(0),
    );
    let limits = support::TEST_LIMITS.with_components(0);

    let error = validate_schema(manifest, limits).expect_err("component limit should fail");

    assert_eq!(
        error.kind(),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::Components)
    );
    assert_eq!(error.span(), invalid);
}

#[test]
fn nested_region_expansion_uses_checked_multiplicity() {
    let namespace = SchemaNamespace::new(7);
    let root = TemplateNodeId::new(0);
    let outer_body = TemplateNodeId::new(1);
    let inner_body = TemplateNodeId::new(2);
    let outer = StructuralRegionId::new(0);
    let inner = StructuralRegionId::new(1);
    let structural = InvalidationSet::from_class(InvalidationClass::Structure)
        .union(InvalidationSet::from_class(InvalidationClass::Layout));
    let manifest = manifest_with(
        fenestra_ui_ir::prototype::SUPPORTED_SCHEMA_FORMAT,
        namespace,
        SCHEMA_REVISION,
        vec![component(
            ComponentTypeId::new(0),
            vec![scalar_property(PropertyId::new(0), span(2))],
            span(1),
        )],
        span(0),
    );
    let program = program_with(
        support::SUPPORTED_CONSTRUCTION_FORMAT,
        namespace,
        SCHEMA_REVISION,
        vec![
            node(
                root,
                ComponentTypeId::new(0),
                Vec::new(),
                vec![ChildSlot::region(outer, span(11))],
                span(10),
            ),
            node(
                outer_body,
                ComponentTypeId::new(0),
                Vec::new(),
                vec![ChildSlot::region(inner, span(13))],
                span(12),
            ),
            node(
                inner_body,
                ComponentTypeId::new(0),
                Vec::new(),
                Vec::new(),
                span(14),
            ),
        ],
        vec![
            region_with_invalidation(
                outer,
                root,
                outer_body,
                vec![key(1, span(15)), key(2, span(16))],
                structural,
                span(17),
            ),
            region_with_invalidation(
                inner,
                outer_body,
                inner_body,
                vec![key(3, span(18)), key(4, span(19)), key(5, span(20))],
                structural,
                span(21),
            ),
        ],
        span(9),
    );
    let schema = validate_schema(manifest, support::TEST_LIMITS).expect("schema should validate");

    let error = validate_construction(
        &schema,
        program.clone(),
        support::TEST_LIMITS.with_initial_instances(8),
    )
    .expect_err("1 + 2 + 6 instances should exceed eight");
    assert_eq!(
        error.kind(),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::InitialInstances)
    );
    assert_eq!(error.span(), span(21));

    validate_construction(
        &schema,
        program,
        support::TEST_LIMITS.with_initial_instances(9),
    )
    .expect("nine expanded instances should fit an inclusive limit");
}

#[test]
fn multiplicity_overflow_reports_the_first_authored_sibling_region() {
    let prefix_regions = usize::BITS as usize - 1;
    let first_overflow = StructuralRegionId::new(prefix_regions as u32);
    let second_overflow = StructuralRegionId::new(prefix_regions as u32 + 1);
    let first_target = TemplateNodeId::new(prefix_regions as u32 + 1);
    let second_target = TemplateNodeId::new(prefix_regions as u32 + 2);
    let structural = InvalidationSet::from_class(InvalidationClass::Structure);
    let mut nodes = Vec::new();
    let mut regions = Vec::new();

    for index in 0..=prefix_regions {
        let owner = TemplateNodeId::new(index as u32);
        let children = if index < prefix_regions {
            vec![ChildSlot::region(
                StructuralRegionId::new(index as u32),
                span(100 + index as u32),
            )]
        } else {
            vec![
                ChildSlot::region(first_overflow, span(300)),
                ChildSlot::region(second_overflow, span(301)),
            ]
        };
        nodes.push(node(
            owner,
            COMPONENT,
            Vec::new(),
            children,
            span(400 + index as u32),
        ));

        if index < prefix_regions {
            regions.push(region_with_invalidation(
                StructuralRegionId::new(index as u32),
                owner,
                TemplateNodeId::new(index as u32 + 1),
                vec![
                    key(0, span(500 + index as u32 * 2)),
                    key(1, span(501 + index as u32 * 2)),
                ],
                structural,
                span(200 + index as u32),
            ));
        }
    }
    nodes.push(node(
        first_target,
        COMPONENT,
        Vec::new(),
        Vec::new(),
        span(700),
    ));
    nodes.push(node(
        second_target,
        COMPONENT,
        Vec::new(),
        Vec::new(),
        span(701),
    ));
    regions.push(region_with_invalidation(
        first_overflow,
        TemplateNodeId::new(prefix_regions as u32),
        first_target,
        vec![key(0, span(702)), key(1, span(703))],
        structural,
        span(302),
    ));
    regions.push(region_with_invalidation(
        second_overflow,
        TemplateNodeId::new(prefix_regions as u32),
        second_target,
        vec![key(0, span(704)), key(1, span(705))],
        structural,
        span(303),
    ));

    let program = program_with(
        support::SUPPORTED_CONSTRUCTION_FORMAT,
        support::SCHEMA_NAMESPACE,
        SCHEMA_REVISION,
        nodes,
        regions,
        span(99),
    );
    let count = prefix_regions + 3;
    let region_count = prefix_regions + 2;
    let limits = ValidationLimits::new(
        1,
        1,
        count,
        region_count,
        region_count,
        0,
        region_count * 2,
        prefix_regions + 2,
        usize::MAX,
    );
    let schema = validate_schema(basic_manifest(), limits).expect("schema should validate");

    let error = validate_construction(&schema, program, limits)
        .expect_err("the first final multiplication should overflow");

    assert_eq!(
        error.kind(),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::InitialInstances)
    );
    assert_eq!(error.span(), span(302));
}

#[test]
fn deep_static_ownership_is_validated_iteratively() {
    let depth = 2_048_usize;
    let mut nodes = Vec::with_capacity(depth);
    for index in 0..depth {
        let children = if index + 1 < depth {
            vec![ChildSlot::static_node(
                TemplateNodeId::new(index as u32 + 1),
                span(1_000 + index as u32),
            )]
        } else {
            Vec::new()
        };
        nodes.push(node(
            TemplateNodeId::new(index as u32),
            COMPONENT,
            Vec::new(),
            children,
            span(4_000 + index as u32),
        ));
    }
    let program = program_with(
        support::SUPPORTED_CONSTRUCTION_FORMAT,
        support::SCHEMA_NAMESPACE,
        SCHEMA_REVISION,
        nodes,
        Vec::new(),
        span(900),
    );
    let limits = ValidationLimits::new(1, 1, depth, 0, depth - 1, 0, 0, depth, depth);
    let schema = validate_schema(basic_manifest(), limits).expect("schema should validate");

    let validated = validate_construction(&schema, program, limits)
        .expect("deep static chain should validate without recursion");

    assert_eq!(validated.root_factory().id(), TemplateNodeId::new(0));
}
