use fenestra_ui_authoring::prototype::{
    AnchorKindV2, AuthoringDiagnosticKindV2, AuthoringLimitKindV2, DiagnosticLocationV2,
};
use fenestra_ui_ir::prototype::{
    IrValidationErrorKind, SpatialValidationLimitsV2, StyleValidationLimits, ValidationLimitKind,
    ValidationLimits, validate_construction, validate_schema, validate_spatial, validate_style,
};

use crate::support;

const MAX_FIXED: i64 = 140_737_488_289_792;

#[test]
fn all_thirteen_authoring_limits_map_to_the_exact_spatial_validation_slots() {
    let compiled = support::compile_fen(support::FIXTURE);
    let style = validated_style(&compiled);
    let cases = [
        (
            AuthoringLimitKindV2::SpatialNodes,
            ValidationLimitKind::SpatialNodes,
            7,
        ),
        (
            AuthoringLimitKindV2::Shapes,
            ValidationLimitKind::SpatialShapes,
            5,
        ),
        (
            AuthoringLimitKindV2::Brushes,
            ValidationLimitKind::SpatialBrushes,
            3,
        ),
        (
            AuthoringLimitKindV2::Clips,
            ValidationLimitKind::SpatialClips,
            3,
        ),
        (
            AuthoringLimitKindV2::PaintItems,
            ValidationLimitKind::SpatialPaintItems,
            4,
        ),
        (
            AuthoringLimitKindV2::HitItems,
            ValidationLimitKind::SpatialHitItems,
            4,
        ),
        (
            AuthoringLimitKindV2::SemanticItems,
            ValidationLimitKind::SpatialSemanticItems,
            3,
        ),
        (
            AuthoringLimitKindV2::Paths,
            ValidationLimitKind::SpatialPaths,
            1,
        ),
        (
            AuthoringLimitKindV2::PathVerbs,
            ValidationLimitKind::SpatialPathVerbs,
            5,
        ),
        (
            AuthoringLimitKindV2::PolygonPoints,
            ValidationLimitKind::SpatialPolygonPoints,
            3,
        ),
        (
            AuthoringLimitKindV2::GradientStops,
            ValidationLimitKind::SpatialGradientStops,
            3,
        ),
        (
            AuthoringLimitKindV2::Images,
            ValidationLimitKind::SpatialImages,
            1,
        ),
        (
            AuthoringLimitKindV2::ImageBytes,
            ValidationLimitKind::SpatialImageBytes,
            16,
        ),
    ];
    let exact_values = cases.map(|(_, _, observed)| observed);
    validate_spatial(
        &style,
        compiled.spatial().clone(),
        SpatialValidationLimitsV2::new(exact_values),
    )
    .expect("all exact spatial validator limits must pass");

    for (index, (authoring, validation, observed)) in cases.into_iter().enumerate() {
        let mut values = exact_values;
        values[index] = observed - 1;
        let error = validate_spatial(
            &style,
            compiled.spatial().clone(),
            SpatialValidationLimitsV2::new(values),
        )
        .expect_err("one-under spatial validator limit must fail");
        assert_eq!(
            error.kind(),
            IrValidationErrorKind::LimitExceeded(validation),
            "wrong IR limit slot for {authoring:?}"
        );

        let error = support::compile_fen_with(
            support::FIXTURE,
            support::limits_with(authoring, observed - 1),
        )
        .expect_err("one-under authored resource must fail before allocation");
        assert_eq!(
            error.kind(),
            AuthoringDiagnosticKindV2::LimitExceeded(authoring)
        );
    }
}

#[test]
fn private_construction_bounds_are_depth_four_and_eight_instances() {
    support::compile_fen_with(support::FIXTURE, support::limits())
        .expect("reference depth four and eight instances must pass");

    let without_old_owner =
        support::replace_once(support::FIXTURE, "    child template overlay;\n", "");
    let depth_five = support::replace_once(
        &without_old_owner,
        "  template free_child = 3: fixture {\n    set span_x = 12;\n    set span_y = 10;\n  }",
        "  template free_child = 3: fixture {\n    set span_x = 12;\n    set span_y = 10;\n    child template overlay;\n  }",
    );
    assert_ir_kind(
        &depth_five,
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::TemplateDepth),
    );

    let nine_instances =
        support::replace_once(support::FIXTURE, "keys [10, 20]", "keys [10, 20, 30]");
    assert_ir_kind(
        &nine_instances,
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::InitialInstances),
    );
}

#[test]
fn fixed_range_ir_failures_bridge_the_exact_field_span_and_phase_order() {
    let invalid = (MAX_FIXED + 1).to_string();
    let first = support::replace_once(
        support::FIXTURE,
        "      width property span_x;",
        &format!("      width fixed({invalid});"),
    );
    let two = support::replace_once(
        &first,
        "      radius fixed(131072);",
        &format!("      radius fixed({});", MAX_FIXED + 2),
    );
    assert_spatial_field_ir_error(
        &two,
        IrValidationErrorKind::SpatialFixed16OutOfRange,
        &invalid,
        &[&(MAX_FIXED + 2).to_string()],
    );
}

#[test]
fn the_full_negative_i64_minimum_reaches_fixed_domain_validation() {
    let literal = "-9223372036854775808";
    let source = support::replace_once(
        support::FIXTURE,
        "      radius fixed(131072);",
        &format!("      radius fixed({literal});"),
    );
    assert_spatial_field_ir_error(
        &source,
        IrValidationErrorKind::SpatialFixed16OutOfRange,
        literal,
        &[],
    );
}

fn validated_style(
    compiled: &fenestra_ui_authoring::prototype::CompiledAuthoringV2,
) -> fenestra_ui_ir::prototype::ValidatedStyleProgram {
    let validation = ValidationLimits::new(1, 8, 7, 1, 6, 19, 2, 4, 8);
    let schema = validate_schema(compiled.schema().clone(), validation)
        .expect("reference schema should validate");
    let construction = validate_construction(&schema, compiled.construction().clone(), validation)
        .expect("reference construction should validate");
    validate_style(
        &construction,
        compiled.style().clone(),
        StyleValidationLimits::new(3),
    )
    .expect("reference style should validate")
}

fn assert_ir_kind(source: &str, expected: IrValidationErrorKind) {
    let error = support::compile_fen_with(source, support::limits())
        .expect_err("the mutated reference must fail validation");
    assert_eq!(
        error.kind(),
        AuthoringDiagnosticKindV2::IrValidation(expected)
    );
}

fn assert_spatial_field_ir_error(
    source: &str,
    expected: IrValidationErrorKind,
    literal: &str,
    later_invalid_literals: &[&str],
) {
    let error = support::compile_fen_with(source, support::limits())
        .expect_err("the invalid fixed literal must fail raw IR validation");
    assert_eq!(
        error.kind(),
        AuthoringDiagnosticKindV2::IrValidation(expected)
    );
    let DiagnosticLocationV2::Anchored {
        logical,
        anchor_kind,
        physical,
    } = error.location()
    else {
        panic!("IR validation must map back through a logical anchor");
    };
    assert_eq!(*anchor_kind, AnchorKindV2::SpatialField);
    assert_eq!(physical.source_id(), Some(support::SOURCE_ID));
    let start = source.find(literal).expect("literal must occur") as u32;
    let end = start + u32::try_from(literal.len()).expect("fixture literal length");
    assert_eq!(physical.fen_byte_range(), Some((start, end)));

    let mut compiled_source = support::replace_once(source, literal, "0");
    for later in later_invalid_literals {
        compiled_source = support::replace_once(&compiled_source, later, "0");
    }
    let compiled = support::compile_fen(&compiled_source);
    let mapped = compiled
        .source_map()
        .entries()
        .iter()
        .find(|entry| {
            entry.anchor_kind() == AnchorKindV2::SpatialField
                && entry.physical_origin().fen_byte_range() == Some((start, start + 1))
        })
        .expect("replacement field must remain mapped");
    assert_eq!(mapped.logical_span(), *logical);
}
