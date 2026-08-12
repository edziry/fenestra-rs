mod construction;
mod schema;
mod style;

use std::collections::HashMap;

use fenestra_ui_ir::prototype::ValueType;

use crate::diagnostic::{AuthoringDiagnosticKindV1, AuthoringDiagnosticV1};
use crate::parsed::{ParsedDocumentV1, ParsedLiteralV1};
use crate::resolved::ResolvedDocumentV1;

use self::construction::resolve_construction;
use self::schema::resolve_schema;
use self::style::resolve_style;
use super::failure_at_origin;

pub(super) struct ComponentBindingV1 {
    pub(super) id: u32,
    pub(super) properties: HashMap<Box<str>, PropertyBindingV1>,
}

pub(super) struct PropertyBindingV1 {
    pub(super) id: u32,
    pub(super) value_type: ValueType,
}

pub(super) struct TemplateBindingV1 {
    pub(super) id: u32,
    pub(super) component: Box<str>,
}

pub(super) struct NameIndexesV1 {
    pub(super) components: HashMap<Box<str>, ComponentBindingV1>,
    pub(super) templates: HashMap<Box<str>, TemplateBindingV1>,
    pub(super) regions: HashMap<Box<str>, u32>,
}

impl NameIndexesV1 {
    fn new(parsed: &ParsedDocumentV1) -> Self {
        let mut components = HashMap::new();
        for component in &parsed.schema.components {
            components.entry(component.name.clone()).or_insert_with(|| {
                let properties = component
                    .properties
                    .iter()
                    .map(|property| {
                        (
                            property.name.clone(),
                            PropertyBindingV1 {
                                id: provisional(&property.id),
                                value_type: property.value_type,
                            },
                        )
                    })
                    .collect();
                ComponentBindingV1 {
                    id: provisional(&component.id),
                    properties,
                }
            });
        }

        let mut templates = HashMap::new();
        for template in &parsed.construction.templates {
            templates
                .entry(template.name.clone())
                .or_insert_with(|| TemplateBindingV1 {
                    id: provisional(&template.id),
                    component: template.component.value.clone(),
                });
        }

        let mut regions = HashMap::new();
        for region in &parsed.construction.regions {
            regions
                .entry(region.name.clone())
                .or_insert_with(|| provisional(&region.id));
        }

        Self {
            components,
            templates,
            regions,
        }
    }
}

pub(crate) fn resolve_semantics(
    parsed: &ParsedDocumentV1,
) -> Result<ResolvedDocumentV1, AuthoringDiagnosticV1> {
    let indexes = NameIndexesV1::new(parsed);
    let schema = resolve_schema(parsed)?;
    let construction = resolve_construction(parsed, &indexes)?;
    let style = resolve_style(parsed, &indexes)?;
    Ok(ResolvedDocumentV1 {
        format: parsed.format,
        document_anchor: parsed.document_anchor,
        schema,
        construction,
        style,
    })
}

pub(super) fn literal<'a, T>(
    parsed: &ParsedDocumentV1,
    anchor: u32,
    literal: &'a ParsedLiteralV1<T>,
) -> Result<&'a T, AuthoringDiagnosticV1> {
    literal.value.as_ref().map_err(|physical| {
        failure_at_origin(
            parsed,
            anchor,
            AuthoringDiagnosticKindV1::InvalidLiteral,
            *physical,
        )
    })
}

fn provisional<T>(literal: &ParsedLiteralV1<T>) -> T
where
    T: Copy + Default,
{
    literal.value.unwrap_or_else(|_| T::default())
}
