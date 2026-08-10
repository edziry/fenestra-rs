use std::collections::HashMap;

use fenestra_ui_ir::prototype::{PropertyValue, ValueType};

use crate::diagnostic::{AuthoringDiagnosticKindV1, AuthoringDiagnosticV1};
use crate::parsed::{
    ParsedChildV1, ParsedDocumentV1, ParsedInitialPropertyV1, ParsedTemplateV1, SpannedV1,
};
use crate::resolved::{
    ResolvedChildV1, ResolvedConstructionV1, ResolvedInitialKeyV1, ResolvedInitialPropertyV1,
    ResolvedRegionV1, ResolvedStyleAssignmentV1, ResolvedStyleV1, ResolvedTemplateV1,
};

use super::{ComponentInfo, TemplateInfo, failure, failure_at_origin};

pub(super) fn resolve_construction(
    parsed: &ParsedDocumentV1,
    components: &HashMap<Box<str>, ComponentInfo>,
    templates: &HashMap<Box<str>, TemplateInfo>,
    regions: &HashMap<Box<str>, u32>,
) -> Result<ResolvedConstructionV1, AuthoringDiagnosticV1> {
    let resolved_templates = parsed
        .construction
        .templates
        .iter()
        .map(|template| resolve_template(parsed, template, components, templates, regions))
        .collect::<Result<Vec<_>, _>>()?;
    let mut resolved_regions = Vec::with_capacity(parsed.construction.regions.len());
    for region in &parsed.construction.regions {
        let owner = spanned_template_id(parsed, templates, &region.owner, region.anchor)?;
        let repeat_body =
            spanned_template_id(parsed, templates, &region.repeat_body, region.anchor)?;
        resolved_regions.push(ResolvedRegionV1 {
            name: region.name.clone(),
            id: region.id,
            owner,
            repeat_body,
            initial_keys: region
                .initial_keys
                .iter()
                .map(|key| ResolvedInitialKeyV1 {
                    value: key.value,
                    anchor: key.anchor,
                })
                .collect(),
            invalidation: region.invalidation,
            anchor: region.anchor,
        });
    }
    Ok(ResolvedConstructionV1 {
        templates: resolved_templates,
        regions: resolved_regions,
        anchor: parsed.construction.anchor,
    })
}

fn resolve_template(
    parsed: &ParsedDocumentV1,
    template: &ParsedTemplateV1,
    components: &HashMap<Box<str>, ComponentInfo>,
    templates: &HashMap<Box<str>, TemplateInfo>,
    regions: &HashMap<Box<str>, u32>,
) -> Result<ResolvedTemplateV1, AuthoringDiagnosticV1> {
    let component = components
        .get(template.component.value.as_ref())
        .ok_or_else(|| {
            failure_at_origin(
                parsed,
                template.anchor,
                AuthoringDiagnosticKindV1::UnknownComponentName,
                template.component.physical,
            )
        })?;
    let initial_properties = template
        .initial_properties
        .iter()
        .map(|property| resolve_initial(parsed, component, property))
        .collect::<Result<Vec<_>, _>>()?;
    let children = template
        .children
        .iter()
        .map(|child| match child {
            ParsedChildV1::Static { template, anchor } => Ok(ResolvedChildV1::Static {
                template: template_id(parsed, templates, template, *anchor)?,
                anchor: *anchor,
            }),
            ParsedChildV1::Region { region, anchor } => {
                let id = regions.get(region.as_ref()).copied().ok_or_else(|| {
                    failure(
                        parsed,
                        *anchor,
                        AuthoringDiagnosticKindV1::UnknownRegionName,
                    )
                })?;
                Ok(ResolvedChildV1::Region {
                    region: id,
                    anchor: *anchor,
                })
            }
        })
        .collect::<Result<Vec<_>, AuthoringDiagnosticV1>>()?;
    Ok(ResolvedTemplateV1 {
        name: template.name.clone(),
        id: template.id,
        component: component.id,
        initial_properties,
        children,
        anchor: template.anchor,
    })
}

fn resolve_initial(
    parsed: &ParsedDocumentV1,
    component: &ComponentInfo,
    initial: &ParsedInitialPropertyV1,
) -> Result<ResolvedInitialPropertyV1, AuthoringDiagnosticV1> {
    let property = component
        .properties
        .get(initial.property.as_ref())
        .ok_or_else(|| {
            failure(
                parsed,
                initial.anchor,
                AuthoringDiagnosticKindV1::UnknownPropertyName,
            )
        })?;
    require_type(parsed, initial.anchor, &initial.value, property.value_type)?;
    Ok(ResolvedInitialPropertyV1 {
        property: property.id,
        value: initial.value.value.clone(),
        anchor: initial.anchor,
    })
}

pub(super) fn resolve_style(
    parsed: &ParsedDocumentV1,
    components: &HashMap<Box<str>, ComponentInfo>,
    templates: &HashMap<Box<str>, TemplateInfo>,
) -> Result<ResolvedStyleV1, AuthoringDiagnosticV1> {
    let mut assignments = Vec::with_capacity(parsed.style.assignments.len());
    for assignment in &parsed.style.assignments {
        let target = templates
            .get(assignment.target.value.as_ref())
            .ok_or_else(|| {
                failure_at_origin(
                    parsed,
                    assignment.anchor,
                    AuthoringDiagnosticKindV1::UnknownTemplateName,
                    assignment.target.physical,
                )
            })?;
        let component = components.get(target.component.as_ref()).ok_or_else(|| {
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
        require_type(
            parsed,
            assignment.anchor,
            &assignment.value,
            property.value_type,
        )?;
        assignments.push(ResolvedStyleAssignmentV1 {
            target: target.id,
            property: property.id,
            value: assignment.value.value.clone(),
            anchor: assignment.anchor,
        });
    }
    Ok(ResolvedStyleV1 {
        assignments,
        anchor: parsed.style.anchor,
    })
}

fn template_id(
    parsed: &ParsedDocumentV1,
    templates: &HashMap<Box<str>, TemplateInfo>,
    name: &str,
    anchor: u32,
) -> Result<u32, AuthoringDiagnosticV1> {
    templates.get(name).map(|entry| entry.id).ok_or_else(|| {
        failure(
            parsed,
            anchor,
            AuthoringDiagnosticKindV1::UnknownTemplateName,
        )
    })
}

fn spanned_template_id(
    parsed: &ParsedDocumentV1,
    templates: &HashMap<Box<str>, TemplateInfo>,
    name: &SpannedV1<Box<str>>,
    anchor: u32,
) -> Result<u32, AuthoringDiagnosticV1> {
    templates
        .get(name.value.as_ref())
        .map(|entry| entry.id)
        .ok_or_else(|| {
            failure_at_origin(
                parsed,
                anchor,
                AuthoringDiagnosticKindV1::UnknownTemplateName,
                name.physical,
            )
        })
}

fn require_type(
    parsed: &ParsedDocumentV1,
    anchor: u32,
    value: &SpannedV1<PropertyValue>,
    expected: ValueType,
) -> Result<(), AuthoringDiagnosticV1> {
    if value.value.value_type() == expected {
        Ok(())
    } else {
        Err(failure_at_origin(
            parsed,
            anchor,
            AuthoringDiagnosticKindV1::ValueTypeMismatch,
            value.physical,
        ))
    }
}
