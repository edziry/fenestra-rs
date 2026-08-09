#[path = "support/style.rs"]
mod style_support;
mod support;

use fenestra_ui_ir::prototype::{
    IrValidationErrorKind, PropertyId, PropertyValue, SUPPORTED_STYLE_FORMAT, SchemaNamespace,
    SchemaRevision, SourceId, SourceSpan, StyleFormatVersion, StyleValidationLimits,
    TemplateNodeId, ValidationLimitKind, validate_style,
};

use style_support::{
    assignment, construction, program_with, scalar_assignment, style_limits, validate_program,
};
use support::construction_faults::CONSTRUCTION_FAULTS;
use support::malformed::malformed_fixture;
use support::{PROPERTY, ROOT, SCHEMA_NAMESPACE, SCHEMA_REVISION, span};

#[derive(Clone, Copy, Debug)]
enum StyleFault {
    UnsupportedFormat,
    NamespaceMismatch,
    RevisionMismatch,
    InvalidProgramSpan,
    InvalidAssignmentSpan,
    MissingTarget,
    UnknownProperty,
    PropertyTypeMismatch,
    DuplicateAssignment,
    AssignmentLimit,
}

const STYLE_FAULTS: [StyleFault; 10] = [
    StyleFault::UnsupportedFormat,
    StyleFault::NamespaceMismatch,
    StyleFault::RevisionMismatch,
    StyleFault::InvalidProgramSpan,
    StyleFault::InvalidAssignmentSpan,
    StyleFault::MissingTarget,
    StyleFault::UnknownProperty,
    StyleFault::PropertyTypeMismatch,
    StyleFault::DuplicateAssignment,
    StyleFault::AssignmentLimit,
];

fn malformed_style(
    fault: StyleFault,
) -> (
    fenestra_ui_ir::prototype::StyleProgram,
    StyleValidationLimits,
    IrValidationErrorKind,
    SourceSpan,
) {
    let invalid = SourceSpan::bytes(SourceId::new(0), 50, 40);
    let mut format = SUPPORTED_STYLE_FORMAT;
    let mut namespace = SCHEMA_NAMESPACE;
    let mut revision = SCHEMA_REVISION;
    let mut assignments = vec![scalar_assignment(ROOT, 20, span(21))];
    let mut program_span = span(20);
    let mut limits = style_limits(64);

    let (kind, expected_span) = match fault {
        StyleFault::UnsupportedFormat => {
            format = StyleFormatVersion::new(SUPPORTED_STYLE_FORMAT.get() + 1);
            (IrValidationErrorKind::UnsupportedStyleFormat, program_span)
        }
        StyleFault::NamespaceMismatch => {
            namespace = SchemaNamespace::new(SCHEMA_NAMESPACE.get() + 1);
            (IrValidationErrorKind::SchemaIdentityMismatch, program_span)
        }
        StyleFault::RevisionMismatch => {
            revision = SchemaRevision::new(SCHEMA_REVISION.get() + 1);
            (IrValidationErrorKind::SchemaIdentityMismatch, program_span)
        }
        StyleFault::InvalidProgramSpan => {
            program_span = invalid;
            (IrValidationErrorKind::InvalidSourceSpan, invalid)
        }
        StyleFault::InvalidAssignmentSpan => {
            assignments = vec![scalar_assignment(ROOT, 20, invalid)];
            (IrValidationErrorKind::InvalidSourceSpan, invalid)
        }
        StyleFault::MissingTarget => {
            assignments = vec![scalar_assignment(TemplateNodeId::new(99), 20, span(22))];
            (IrValidationErrorKind::MissingStyleTarget, span(22))
        }
        StyleFault::UnknownProperty => {
            assignments = vec![assignment(
                ROOT,
                PropertyId::new(99),
                PropertyValue::ScalarI32(20),
                span(23),
            )];
            (IrValidationErrorKind::UnknownStyleProperty, span(23))
        }
        StyleFault::PropertyTypeMismatch => {
            assignments = vec![assignment(
                ROOT,
                PROPERTY,
                PropertyValue::Bool(true),
                span(24),
            )];
            (IrValidationErrorKind::StylePropertyTypeMismatch, span(24))
        }
        StyleFault::DuplicateAssignment => {
            assignments = vec![
                scalar_assignment(ROOT, 20, span(25)),
                scalar_assignment(ROOT, 30, span(26)),
            ];
            (IrValidationErrorKind::DuplicateStyleAssignment, span(26))
        }
        StyleFault::AssignmentLimit => {
            limits = style_limits(0);
            (
                IrValidationErrorKind::LimitExceeded(ValidationLimitKind::StyleAssignments),
                span(21),
            )
        }
    };

    (
        program_with(format, namespace, revision, assignments, program_span),
        limits,
        kind,
        expected_span,
    )
}

#[test]
fn malformed_style_corpus_covers_typed_failures_and_spans() {
    let construction = construction();
    for fault in STYLE_FAULTS {
        let (program, limits, expected_kind, expected_span) = malformed_style(fault);
        let error = validate_style(&construction, program, limits)
            .expect_err("malformed style should fail validation");
        assert_eq!(error.kind(), expected_kind, "unexpected kind for {fault:?}");
        assert_eq!(error.span(), expected_span, "unexpected span for {fault:?}");
    }
}

#[test]
fn all_error_kinds_remain_exhaustive_across_validator_corpora() {
    let mut expected = CONSTRUCTION_FAULTS
        .map(|fault| malformed_fixture(fault).3)
        .to_vec();
    expected.extend([
        IrValidationErrorKind::UnsupportedStyleFormat,
        IrValidationErrorKind::MissingStyleTarget,
        IrValidationErrorKind::UnknownStyleProperty,
        IrValidationErrorKind::StylePropertyTypeMismatch,
        IrValidationErrorKind::DuplicateStyleAssignment,
        IrValidationErrorKind::LimitExceeded(ValidationLimitKind::StyleAssignments),
    ]);

    assert_eq!(IrValidationErrorKind::ALL.as_slice(), expected.as_slice());
}

#[test]
fn style_validation_uses_documented_diagnostic_priority() {
    let construction = construction();
    let invalid = SourceSpan::bytes(SourceId::new(0), 9, 4);
    let header_error = program_with(
        StyleFormatVersion::new(SUPPORTED_STYLE_FORMAT.get() + 1),
        SchemaNamespace::new(SCHEMA_NAMESPACE.get() + 1),
        SCHEMA_REVISION,
        Vec::new(),
        invalid,
    );
    let error = validate_program(&construction, header_error, 0)
        .expect_err("invalid header span should win");
    assert_eq!(error.kind(), IrValidationErrorKind::InvalidSourceSpan);

    let format_error = program_with(
        StyleFormatVersion::new(SUPPORTED_STYLE_FORMAT.get() + 1),
        SchemaNamespace::new(SCHEMA_NAMESPACE.get() + 1),
        SCHEMA_REVISION,
        vec![scalar_assignment(ROOT, 20, span(29))],
        span(28),
    );
    let error = validate_program(&construction, format_error, 0)
        .expect_err("format should win over schema identity and limits");
    assert_eq!(error.kind(), IrValidationErrorKind::UnsupportedStyleFormat);

    let schema_error = program_with(
        SUPPORTED_STYLE_FORMAT,
        SchemaNamespace::new(SCHEMA_NAMESPACE.get() + 1),
        SCHEMA_REVISION,
        vec![scalar_assignment(ROOT, 20, span(29))],
        span(28),
    );
    let error = validate_program(&construction, schema_error, 0)
        .expect_err("schema identity should win over limits");
    assert_eq!(error.kind(), IrValidationErrorKind::SchemaIdentityMismatch);

    let assignments = vec![
        scalar_assignment(TemplateNodeId::new(99), 20, span(30)),
        assignment(
            ROOT,
            PropertyId::new(99),
            PropertyValue::Bool(true),
            span(31),
        ),
    ];
    let error = validate_program(&construction, style_support::program(assignments), 2)
        .expect_err("first authored assignment should win");
    assert_eq!(error.kind(), IrValidationErrorKind::MissingStyleTarget);
    assert_eq!(error.span(), span(30));

    let duplicate_with_wrong_type = style_support::program(vec![
        scalar_assignment(ROOT, 20, span(32)),
        assignment(ROOT, PROPERTY, PropertyValue::Bool(true), span(33)),
    ]);
    let error = validate_program(&construction, duplicate_with_wrong_type, 2)
        .expect_err("duplicate identity should win over value type");
    assert_eq!(
        error.kind(),
        IrValidationErrorKind::DuplicateStyleAssignment
    );
    assert_eq!(error.span(), span(33));
}

#[test]
fn style_error_formatting_does_not_expose_values_or_domains() {
    let construction = construction();
    let (program, limits, _, _) = malformed_style(StyleFault::PropertyTypeMismatch);
    let error = validate_style(&construction, program, limits).expect_err("style should fail");
    let rendered = format!("{error:?} {error}");

    assert!(rendered.contains("style-property-type-mismatch"));
    for forbidden in ["Bool", "true", "validation_domain", "dense", "index"] {
        assert!(
            !rendered.contains(forbidden),
            "leaked {forbidden}: {rendered}"
        );
    }
}
