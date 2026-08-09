#![allow(dead_code)]

use fenestra_ui_ir::prototype::{
    IrValidationError, PropertyId, PropertyValue, SUPPORTED_STYLE_FORMAT, SchemaNamespace,
    SchemaRevision, SourceSpan, StyleAssignment, StyleFormatVersion, StyleProgram,
    StyleValidationLimits, TemplateNodeId, ValidatedConstruction, ValidatedStyleProgram,
    validate_style,
};

use crate::support::{
    PROPERTY, ROOT, SCHEMA_NAMESPACE, SCHEMA_REVISION, basic_manifest, basic_program, span,
    validate_fixture,
};

pub const STYLE_REPLACEMENT: i32 = 20;

pub fn style_limits(assignments: usize) -> StyleValidationLimits {
    StyleValidationLimits::new(assignments)
}

pub fn construction() -> ValidatedConstruction {
    validate_fixture(basic_manifest(), basic_program()).expect("construction should validate")
}

pub fn assignment(
    target: TemplateNodeId,
    property: PropertyId,
    value: PropertyValue,
    source: SourceSpan,
) -> StyleAssignment {
    StyleAssignment::new(target, property, value, source)
}

pub fn scalar_assignment(
    target: TemplateNodeId,
    value: i32,
    source: SourceSpan,
) -> StyleAssignment {
    assignment(target, PROPERTY, PropertyValue::ScalarI32(value), source)
}

pub fn program_with(
    format: StyleFormatVersion,
    namespace: SchemaNamespace,
    revision: SchemaRevision,
    assignments: Vec<StyleAssignment>,
    source: SourceSpan,
) -> StyleProgram {
    StyleProgram::new(format, namespace, revision, assignments, source)
}

pub fn program(assignments: Vec<StyleAssignment>) -> StyleProgram {
    program_with(
        SUPPORTED_STYLE_FORMAT,
        SCHEMA_NAMESPACE,
        SCHEMA_REVISION,
        assignments,
        span(20),
    )
}

pub fn basic_style_program() -> StyleProgram {
    program(vec![scalar_assignment(ROOT, STYLE_REPLACEMENT, span(21))])
}

pub fn validate_program(
    construction: &ValidatedConstruction,
    program: StyleProgram,
    assignment_limit: usize,
) -> Result<ValidatedStyleProgram, IrValidationError> {
    validate_style(construction, program, style_limits(assignment_limit))
}
