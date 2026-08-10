use crate::diagnostic::{AuthoringDiagnosticKindV1, AuthoringDiagnosticV1};
use crate::parsed::ParsedDocumentV1;
use crate::resolved::{ResolvedStyleAssignmentV1, ResolvedStyleV1};

use super::super::{failure, failure_at_origin};
use super::{NameIndexesV1, literal};

pub(super) fn resolve_style(
    parsed: &ParsedDocumentV1,
    indexes: &NameIndexesV1,
) -> Result<ResolvedStyleV1, AuthoringDiagnosticV1> {
    let mut assignments = Vec::with_capacity(parsed.style.assignments.len());
    for assignment in &parsed.style.assignments {
        let target = indexes
            .templates
            .get(assignment.target.value.as_ref())
            .ok_or_else(|| {
                failure_at_origin(
                    parsed,
                    assignment.anchor,
                    AuthoringDiagnosticKindV1::UnknownTemplateName,
                    assignment.target.physical,
                )
            })?;
        let component = indexes
            .components
            .get(target.component.as_ref())
            .ok_or_else(|| {
                failure_at_origin(
                    parsed,
                    assignment.anchor,
                    AuthoringDiagnosticKindV1::UnknownComponentName,
                    assignment.target.physical,
                )
            })?;
        let property = component
            .properties
            .get(assignment.property.as_ref())
            .ok_or_else(|| {
                failure(
                    parsed,
                    assignment.anchor,
                    AuthoringDiagnosticKindV1::UnknownPropertyName,
                )
            })?;
        let value = literal(parsed, assignment.anchor, &assignment.value)?;
        if value.value_type() != property.value_type {
            return Err(failure_at_origin(
                parsed,
                assignment.anchor,
                AuthoringDiagnosticKindV1::ValueTypeMismatch,
                assignment.value.physical,
            ));
        }
        assignments.push(ResolvedStyleAssignmentV1 {
            target: target.id,
            property: property.id,
            value: value.clone(),
            anchor: assignment.anchor,
        });
    }
    Ok(ResolvedStyleV1 {
        assignments,
        anchor: parsed.style.anchor,
    })
}
