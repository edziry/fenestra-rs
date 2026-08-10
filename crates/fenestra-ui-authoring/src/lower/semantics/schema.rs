use std::collections::HashSet;

use crate::diagnostic::{AuthoringDiagnosticKindV1, AuthoringDiagnosticV1};
use crate::parsed::ParsedDocumentV1;
use crate::resolved::{ResolvedComponentV1, ResolvedPropertyV1, ResolvedSchemaV1};

use super::super::{failure, failure_at_origin};
use super::literal;

pub(super) fn resolve_schema(
    parsed: &ParsedDocumentV1,
) -> Result<ResolvedSchemaV1, AuthoringDiagnosticV1> {
    let namespace = *literal(parsed, parsed.schema.anchor, &parsed.schema.namespace)?;
    let revision = *literal(parsed, parsed.schema.anchor, &parsed.schema.revision)?;
    let mut components = Vec::with_capacity(parsed.schema.components.len());
    let mut seen_components = HashSet::new();

    for component in &parsed.schema.components {
        if !seen_components.insert(component.name.as_ref()) {
            return Err(failure(
                parsed,
                component.anchor,
                AuthoringDiagnosticKindV1::DuplicateComponentName,
            ));
        }
        let id = *literal(parsed, component.anchor, &component.id)?;
        let mut properties = Vec::with_capacity(component.properties.len());
        let mut seen_properties = HashSet::new();
        for property in &component.properties {
            if !seen_properties.insert(property.name.as_ref()) {
                return Err(failure(
                    parsed,
                    property.anchor,
                    AuthoringDiagnosticKindV1::DuplicatePropertyName,
                ));
            }
            let property_id = *literal(parsed, property.anchor, &property.id)?;
            let default = literal(parsed, property.anchor, &property.default)?;
            if default.value_type() != property.value_type {
                return Err(failure_at_origin(
                    parsed,
                    property.anchor,
                    AuthoringDiagnosticKindV1::ValueTypeMismatch,
                    property.default.physical,
                ));
            }
            properties.push(ResolvedPropertyV1 {
                name: property.name.clone(),
                id: property_id,
                value_type: property.value_type,
                default: default.clone(),
                invalidation: property.invalidation,
                anchor: property.anchor,
            });
        }
        components.push(ResolvedComponentV1 {
            name: component.name.clone(),
            id,
            properties,
            anchor: component.anchor,
        });
    }

    Ok(ResolvedSchemaV1 {
        namespace,
        revision,
        components,
        anchor: parsed.schema.anchor,
    })
}
