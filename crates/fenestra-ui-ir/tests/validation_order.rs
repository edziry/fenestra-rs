mod support;

use fenestra_ui_ir::prototype::{
    ChildSlot, IrValidationErrorKind, StructuralRegionId, TemplateNodeId, ValidationLimitKind,
    validate_construction, validate_schema,
};
use support::{
    COMPONENT, REGION, REPEAT, ROOT, SCHEMA_NAMESPACE, SCHEMA_REVISION, basic_manifest,
    basic_region, node, program_with, root, span,
};

#[test]
fn depth_reports_the_first_authored_sibling_edge() {
    let first = TemplateNodeId::new(1);
    let second = TemplateNodeId::new(2);
    let program = program_with(
        support::SUPPORTED_CONSTRUCTION_FORMAT,
        SCHEMA_NAMESPACE,
        SCHEMA_REVISION,
        vec![
            root(vec![
                ChildSlot::static_node(first, span(11)),
                ChildSlot::static_node(second, span(12)),
            ]),
            node(first, COMPONENT, Vec::new(), Vec::new(), span(13)),
            node(second, COMPONENT, Vec::new(), Vec::new(), span(14)),
        ],
        Vec::new(),
        span(4),
    );
    let schema =
        validate_schema(basic_manifest(), support::TEST_LIMITS).expect("schema should validate");

    let error = validate_construction(
        &schema,
        program,
        support::TEST_LIMITS.with_template_depth(1),
    )
    .expect_err("first child should exceed the depth limit");

    assert_eq!(
        error.kind(),
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::TemplateDepth)
    );
    assert_eq!(error.span(), span(11));
}

#[test]
fn static_owners_are_registered_before_repeat_body_owners() {
    let program = program_with(
        support::SUPPORTED_CONSTRUCTION_FORMAT,
        SCHEMA_NAMESPACE,
        SCHEMA_REVISION,
        vec![
            root(vec![
                ChildSlot::region(REGION, span(7)),
                ChildSlot::static_node(REPEAT, span(11)),
            ]),
            node(REPEAT, COMPONENT, Vec::new(), Vec::new(), span(6)),
        ],
        vec![basic_region(Vec::new())],
        span(4),
    );
    let schema =
        validate_schema(basic_manifest(), support::TEST_LIMITS).expect("schema should validate");

    let error = validate_construction(&schema, program, support::TEST_LIMITS)
        .expect_err("repeat body should have a second owner");

    assert_eq!(error.kind(), IrValidationErrorKind::DuplicateNodeOwner);
    assert_eq!(error.span(), span(8));
}

#[test]
fn root_limit_failures_use_the_program_header() {
    let schema =
        validate_schema(basic_manifest(), support::TEST_LIMITS).expect("schema should validate");
    for (limits, expected_kind) in [
        (
            support::TEST_LIMITS.with_template_depth(0),
            ValidationLimitKind::TemplateDepth,
        ),
        (
            support::TEST_LIMITS.with_initial_instances(0),
            ValidationLimitKind::InitialInstances,
        ),
    ] {
        let error = validate_construction(&schema, basic_root_only_program(), limits)
            .expect_err("root should exceed a zero limit");
        assert_eq!(
            error.kind(),
            IrValidationErrorKind::LimitExceeded(expected_kind)
        );
        assert_eq!(error.span(), span(4));
    }
}

#[test]
fn schema_revision_is_part_of_the_authored_identity() {
    let program = program_with(
        support::SUPPORTED_CONSTRUCTION_FORMAT,
        SCHEMA_NAMESPACE,
        fenestra_ui_ir::prototype::SchemaRevision::new(SCHEMA_REVISION.get() + 1),
        vec![root(Vec::new())],
        Vec::new(),
        span(4),
    );
    let schema =
        validate_schema(basic_manifest(), support::TEST_LIMITS).expect("schema should validate");

    let error = validate_construction(&schema, program, support::TEST_LIMITS)
        .expect_err("revision mismatch should fail");

    assert_eq!(error.kind(), IrValidationErrorKind::SchemaIdentityMismatch);
    assert_eq!(error.span(), span(4));
}

#[test]
fn region_ids_can_use_the_maximum_sparse_value() {
    let maximum_region = StructuralRegionId::new(u32::MAX);
    let program = program_with(
        support::SUPPORTED_CONSTRUCTION_FORMAT,
        SCHEMA_NAMESPACE,
        SCHEMA_REVISION,
        vec![
            root(vec![ChildSlot::region(maximum_region, span(7))]),
            node(REPEAT, COMPONENT, Vec::new(), Vec::new(), span(6)),
        ],
        vec![support::region(
            maximum_region,
            ROOT,
            REPEAT,
            Vec::new(),
            span(8),
        )],
        span(4),
    );
    let schema =
        validate_schema(basic_manifest(), support::TEST_LIMITS).expect("schema should validate");

    let validated = validate_construction(&schema, program, support::TEST_LIMITS)
        .expect("sparse region should validate");

    assert_eq!(
        validated
            .region(maximum_region)
            .expect("region should resolve")
            .id(),
        maximum_region
    );
}

fn basic_root_only_program() -> fenestra_ui_ir::prototype::ConstructionProgram {
    program_with(
        support::SUPPORTED_CONSTRUCTION_FORMAT,
        SCHEMA_NAMESPACE,
        SCHEMA_REVISION,
        vec![root(Vec::new())],
        Vec::new(),
        span(4),
    )
}
