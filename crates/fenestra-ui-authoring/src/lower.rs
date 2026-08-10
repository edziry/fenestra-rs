use std::collections::HashMap;

use fenestra_ui_ir::prototype::ValueType;

use crate::compiled::CompiledAuthoringV1;
use crate::diagnostic::{AuthoringDiagnosticKindV1, AuthoringDiagnosticV1};
use crate::limits::AuthoringLimitsV1;
use crate::parsed::ParsedDocumentV1;
use crate::resolved::{
    ResolvedComponentV1, ResolvedDocumentV1, ResolvedPropertyV1, ResolvedSchemaV1, logical_span,
};
use crate::source::DiagnosticLocationV1;

mod resolve;
mod validate;

use resolve::{resolve_construction, resolve_style};
use validate::{source_map, validate_programs};

pub(super) struct ComponentInfo {
    pub(super) id: u32,
    pub(super) properties: HashMap<Box<str>, PropertyInfo>,
}

#[derive(Clone, Copy)]
pub(super) struct PropertyInfo {
    pub(super) id: u32,
    pub(super) value_type: ValueType,
}

pub(super) struct TemplateInfo {
    pub(super) id: u32,
    pub(super) component: Box<str>,
}

pub(crate) fn lower_document_v1(
    parsed: ParsedDocumentV1,
    limits: AuthoringLimitsV1,
) -> Result<CompiledAuthoringV1, AuthoringDiagnosticV1> {
    let components = component_table(&parsed)?;
    let templates = template_table(&parsed)?;
    let regions = region_table(&parsed)?;
    let resolved = resolve_document(&parsed, &components, &templates, &regions)?;
    let (schema, construction, style) = resolved.raw_programs();
    validate_programs(&parsed, &schema, &construction, &style, limits)?;
    let source_map = source_map(&parsed);
    let catalog = vec![b'@'; parsed.anchors.len()];
    Ok(CompiledAuthoringV1::new(
        schema,
        construction,
        style,
        catalog,
        source_map,
        resolved,
    ))
}

fn component_table(
    parsed: &ParsedDocumentV1,
) -> Result<HashMap<Box<str>, ComponentInfo>, AuthoringDiagnosticV1> {
    let mut components = HashMap::with_capacity(parsed.schema.components.len());
    for component in &parsed.schema.components {
        if components.contains_key(component.name.as_ref()) {
            return Err(failure(
                parsed,
                component.anchor,
                AuthoringDiagnosticKindV1::DuplicateComponentName,
            ));
        }
        let mut properties = HashMap::with_capacity(component.properties.len());
        for property in &component.properties {
            if properties.contains_key(property.name.as_ref()) {
                return Err(failure(
                    parsed,
                    property.anchor,
                    AuthoringDiagnosticKindV1::DuplicatePropertyName,
                ));
            }
            if property.default.value_type() != property.value_type {
                return Err(failure(
                    parsed,
                    property.anchor,
                    AuthoringDiagnosticKindV1::ValueTypeMismatch,
                ));
            }
            properties.insert(
                property.name.clone(),
                PropertyInfo {
                    id: property.id,
                    value_type: property.value_type,
                },
            );
        }
        components.insert(
            component.name.clone(),
            ComponentInfo {
                id: component.id,
                properties,
            },
        );
    }
    Ok(components)
}

fn template_table(
    parsed: &ParsedDocumentV1,
) -> Result<HashMap<Box<str>, TemplateInfo>, AuthoringDiagnosticV1> {
    let mut templates = HashMap::with_capacity(parsed.construction.templates.len());
    for template in &parsed.construction.templates {
        if templates.contains_key(template.name.as_ref()) {
            return Err(failure(
                parsed,
                template.anchor,
                AuthoringDiagnosticKindV1::DuplicateTemplateName,
            ));
        }
        templates.insert(
            template.name.clone(),
            TemplateInfo {
                id: template.id,
                component: template.component.clone(),
            },
        );
    }
    Ok(templates)
}

fn region_table(
    parsed: &ParsedDocumentV1,
) -> Result<HashMap<Box<str>, u32>, AuthoringDiagnosticV1> {
    let mut regions = HashMap::with_capacity(parsed.construction.regions.len());
    for region in &parsed.construction.regions {
        if regions.insert(region.name.clone(), region.id).is_some() {
            return Err(failure(
                parsed,
                region.anchor,
                AuthoringDiagnosticKindV1::DuplicateRegionName,
            ));
        }
    }
    Ok(regions)
}

fn resolve_document(
    parsed: &ParsedDocumentV1,
    components: &HashMap<Box<str>, ComponentInfo>,
    templates: &HashMap<Box<str>, TemplateInfo>,
    regions: &HashMap<Box<str>, u32>,
) -> Result<ResolvedDocumentV1, AuthoringDiagnosticV1> {
    let schema = ResolvedSchemaV1 {
        namespace: parsed.schema.namespace,
        revision: parsed.schema.revision,
        components: parsed
            .schema
            .components
            .iter()
            .map(|component| ResolvedComponentV1 {
                name: component.name.clone(),
                id: component.id,
                properties: component
                    .properties
                    .iter()
                    .map(|property| ResolvedPropertyV1 {
                        name: property.name.clone(),
                        id: property.id,
                        value_type: property.value_type,
                        default: property.default.clone(),
                        invalidation: property.invalidation,
                        anchor: property.anchor,
                    })
                    .collect(),
                anchor: component.anchor,
            })
            .collect(),
        anchor: parsed.schema.anchor,
    };
    let construction = resolve_construction(parsed, components, templates, regions)?;
    let style = resolve_style(parsed, components, templates)?;
    Ok(ResolvedDocumentV1 {
        format: parsed.format,
        document_anchor: parsed.document_anchor,
        schema,
        construction,
        style,
    })
}

pub(super) fn failure(
    parsed: &ParsedDocumentV1,
    ordinal: u32,
    kind: AuthoringDiagnosticKindV1,
) -> AuthoringDiagnosticV1 {
    let anchor = &parsed.anchors[ordinal as usize];
    AuthoringDiagnosticV1::new(
        parsed.frontend,
        kind,
        DiagnosticLocationV1::Anchored {
            logical: logical_span(ordinal),
            anchor_kind: anchor.kind,
            physical: anchor.physical,
        },
    )
}
