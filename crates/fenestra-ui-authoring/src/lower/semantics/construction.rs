use std::collections::HashSet;

use crate::diagnostic::{AuthoringDiagnosticKindV1, AuthoringDiagnosticV1};
use crate::parsed::{ParsedChildV1, ParsedDocumentV1, ParsedTemplateItemV1, SpannedV1};
use crate::resolved::{
    ResolvedChildV1, ResolvedConstructionV1, ResolvedInitialKeyV1, ResolvedInitialPropertyV1,
    ResolvedRegionV1, ResolvedTemplateV1,
};

use super::super::{failure, failure_at_origin};
use super::{ComponentBindingV1, NameIndexesV1, literal};

pub(super) fn resolve_construction(
    parsed: &ParsedDocumentV1,
    indexes: &NameIndexesV1,
) -> Result<ResolvedConstructionV1, AuthoringDiagnosticV1> {
    let mut templates = Vec::with_capacity(parsed.construction.templates.len());
    let mut seen_templates = HashSet::new();
    for template in &parsed.construction.templates {
        if !seen_templates.insert(template.name.as_ref()) {
            return Err(failure(
                parsed,
                template.anchor,
                AuthoringDiagnosticKindV1::DuplicateTemplateName,
            ));
        }
        let component = indexes
            .components
            .get(template.component.value.as_ref())
            .ok_or_else(|| {
                failure_at_origin(
                    parsed,
                    template.anchor,
                    AuthoringDiagnosticKindV1::UnknownComponentName,
                    template.component.physical,
                )
            })?;
        let id = *literal(parsed, template.anchor, &template.id)?;
        templates.push(resolve_template(parsed, indexes, component, template, id)?);
    }

    let mut regions = Vec::with_capacity(parsed.construction.regions.len());
    let mut seen_regions = HashSet::new();
    for region in &parsed.construction.regions {
        if !seen_regions.insert(region.name.as_ref()) {
            return Err(failure(
                parsed,
                region.anchor,
                AuthoringDiagnosticKindV1::DuplicateRegionName,
            ));
        }
        let owner = template_binding(parsed, indexes, region.anchor, &region.owner)?.id;
        let repeat_body = template_binding(parsed, indexes, region.anchor, &region.repeat_body)?.id;
        let id = *literal(parsed, region.anchor, &region.id)?;
        let mut initial_keys = Vec::with_capacity(region.initial_keys.len());
        for key in &region.initial_keys {
            initial_keys.push(ResolvedInitialKeyV1 {
                value: *literal(parsed, key.anchor, &key.value)?,
                anchor: key.anchor,
            });
        }
        regions.push(ResolvedRegionV1 {
            name: region.name.clone(),
            id,
            owner,
            repeat_body,
            initial_keys,
            invalidation: region.invalidation,
            anchor: region.anchor,
        });
    }

    Ok(ResolvedConstructionV1 {
        templates,
        regions,
        anchor: parsed.construction.anchor,
    })
}

fn resolve_template(
    parsed: &ParsedDocumentV1,
    indexes: &NameIndexesV1,
    component: &ComponentBindingV1,
    template: &crate::parsed::ParsedTemplateV1,
    id: u32,
) -> Result<ResolvedTemplateV1, AuthoringDiagnosticV1> {
    let mut initial_properties = Vec::new();
    let mut children = Vec::new();
    for item in &template.items {
        match item {
            ParsedTemplateItemV1::Initial(initial) => {
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
                let value = literal(parsed, initial.anchor, &initial.value)?;
                if value.value_type() != property.value_type {
                    return Err(failure_at_origin(
                        parsed,
                        initial.anchor,
                        AuthoringDiagnosticKindV1::ValueTypeMismatch,
                        initial.value.physical,
                    ));
                }
                initial_properties.push(ResolvedInitialPropertyV1 {
                    property: property.id,
                    value: value.clone(),
                    anchor: initial.anchor,
                });
            }
            ParsedTemplateItemV1::Child(child) => {
                children.push(resolve_child(parsed, indexes, child)?);
            }
        }
    }
    Ok(ResolvedTemplateV1 {
        name: template.name.clone(),
        id,
        component: component.id,
        initial_properties,
        children,
        anchor: template.anchor,
    })
}

fn resolve_child(
    parsed: &ParsedDocumentV1,
    indexes: &NameIndexesV1,
    child: &ParsedChildV1,
) -> Result<ResolvedChildV1, AuthoringDiagnosticV1> {
    match child {
        ParsedChildV1::Static { template, anchor } => {
            let binding = indexes.templates.get(template.as_ref()).ok_or_else(|| {
                failure(
                    parsed,
                    *anchor,
                    AuthoringDiagnosticKindV1::UnknownTemplateName,
                )
            })?;
            Ok(ResolvedChildV1::Static {
                template: binding.id,
                anchor: *anchor,
            })
        }
        ParsedChildV1::Region { region, anchor } => {
            let id = indexes
                .regions
                .get(region.as_ref())
                .copied()
                .ok_or_else(|| {
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
    }
}

fn template_binding<'a>(
    parsed: &ParsedDocumentV1,
    indexes: &'a NameIndexesV1,
    anchor: u32,
    name: &SpannedV1<Box<str>>,
) -> Result<&'a super::TemplateBindingV1, AuthoringDiagnosticV1> {
    indexes.templates.get(name.value.as_ref()).ok_or_else(|| {
        failure_at_origin(
            parsed,
            anchor,
            AuthoringDiagnosticKindV1::UnknownTemplateName,
            name.physical,
        )
    })
}
