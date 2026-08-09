use std::collections::HashMap;

use crate::error::{IrValidationError, IrValidationErrorKind, ValidationLimitKind};
use crate::ids::{PropertyId, SUPPORTED_STYLE_FORMAT, TemplateNodeId};
use crate::limits::StyleValidationLimits;
use crate::style::StyleProgram;
use crate::validated::{ValidatedConstruction, ValidatedStyleProgram};

use super::{failure, limit_failure};

/// Validates and links an exact-target style program to one construction domain.
pub fn validate_style(
    construction: &ValidatedConstruction,
    program: StyleProgram,
    limits: StyleValidationLimits,
) -> Result<ValidatedStyleProgram, IrValidationError> {
    if !program.span.is_valid() {
        return Err(failure(
            IrValidationErrorKind::InvalidSourceSpan,
            program.span,
        ));
    }
    if program.format != SUPPORTED_STYLE_FORMAT {
        return Err(failure(
            IrValidationErrorKind::UnsupportedStyleFormat,
            program.span,
        ));
    }

    let manifest = &construction.schema().data.manifest;
    if program.schema_namespace != manifest.namespace
        || program.schema_revision != manifest.revision
    {
        return Err(failure(
            IrValidationErrorKind::SchemaIdentityMismatch,
            program.span,
        ));
    }

    if program.assignments.len() > limits.assignments() {
        return Err(limit_failure(
            ValidationLimitKind::StyleAssignments,
            program.assignments[limits.assignments()].span,
        ));
    }

    let mut indexes = HashMap::with_capacity(program.assignments.len());
    for (index, assignment) in program.assignments.iter().enumerate() {
        if !assignment.span.is_valid() {
            return Err(failure(
                IrValidationErrorKind::InvalidSourceSpan,
                assignment.span,
            ));
        }

        let target = construction
            .template(assignment.target)
            .ok_or_else(|| failure(IrValidationErrorKind::MissingStyleTarget, assignment.span))?;
        let key: (TemplateNodeId, PropertyId) = (assignment.target, assignment.property);
        if indexes.contains_key(&key) {
            return Err(failure(
                IrValidationErrorKind::DuplicateStyleAssignment,
                assignment.span,
            ));
        }
        let property = target
            .component()
            .property(assignment.property)
            .ok_or_else(|| failure(IrValidationErrorKind::UnknownStyleProperty, assignment.span))?;
        if property.value_type() != assignment.value.value_type() {
            return Err(failure(
                IrValidationErrorKind::StylePropertyTypeMismatch,
                assignment.span,
            ));
        }
        indexes.insert(key, index);
    }

    Ok(ValidatedStyleProgram::new(
        construction.clone(),
        program,
        indexes,
    ))
}
