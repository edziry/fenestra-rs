use crate::resolved::{
    ResolvedChildV1, ResolvedDocumentV1, ResolvedInitialPropertyV1, ResolvedPropertyV1,
    ResolvedRegionV1, ResolvedStyleAssignmentV1, ResolvedTemplateV1,
};

use super::record::{InvalidRecord, Record, validate_name};
use super::value::{invalidation_name, value_name, value_type_name};

pub(super) fn collect_logical_records_v1(
    document: &ResolvedDocumentV1,
    records: &mut Vec<Record>,
) -> Result<(), InvalidRecord> {
    records.push(Record::new(
        document.document_anchor,
        "document",
        format!("format={}", document.format),
    )?);
    records.push(Record::new(
        document.schema.anchor,
        "schema",
        format!(
            "namespace={}|revision={}",
            document.schema.namespace, document.schema.revision
        ),
    )?);

    for (component_order, component) in document.schema.components.iter().enumerate() {
        validate_name(&component.name)?;
        records.push(Record::new(
            component.anchor,
            "component",
            format!(
                "order={component_order}|name={}|id={}",
                component.name, component.id
            ),
        )?);
        for (property_order, property) in component.properties.iter().enumerate() {
            collect_property(component.id, property_order, property, records)?;
        }
    }

    records.push(Record::new(
        document.construction.anchor,
        "construction",
        schema_identity(document),
    )?);
    for (template_order, template) in document.construction.templates.iter().enumerate() {
        collect_template(template_order, template, records)?;
    }
    for (region_order, region) in document.construction.regions.iter().enumerate() {
        collect_region(region_order, region, records)?;
    }

    records.push(Record::new(
        document.style.anchor,
        "style",
        schema_identity(document),
    )?);
    for (order, assignment) in document.style.assignments.iter().enumerate() {
        collect_style_assignment(order, assignment, records)?;
    }
    Ok(())
}

pub(super) fn logical_record_count_v1(document: &ResolvedDocumentV1) -> Option<usize> {
    let mut count = 4usize;
    for component in &document.schema.components {
        count = count
            .checked_add(1)?
            .checked_add(component.properties.len())?;
    }
    for template in &document.construction.templates {
        count = count
            .checked_add(1)?
            .checked_add(template.initial_properties.len())?
            .checked_add(template.children.len())?;
    }
    for region in &document.construction.regions {
        count = count
            .checked_add(1)?
            .checked_add(region.initial_keys.len())?;
    }
    count.checked_add(document.style.assignments.len())
}

fn collect_property(
    component: u32,
    order: usize,
    property: &ResolvedPropertyV1,
    records: &mut Vec<Record>,
) -> Result<(), InvalidRecord> {
    validate_name(&property.name)?;
    records.push(Record::new(
        property.anchor,
        "property",
        format!(
            "component={component}|order={order}|name={}|id={}|type={}|default={}|invalidates={}",
            property.name,
            property.id,
            value_type_name(property.value_type),
            value_name(&property.default),
            invalidation_name(property.invalidation),
        ),
    )?);
    Ok(())
}

fn collect_template(
    order: usize,
    template: &ResolvedTemplateV1,
    records: &mut Vec<Record>,
) -> Result<(), InvalidRecord> {
    validate_name(&template.name)?;
    records.push(Record::new(
        template.anchor,
        "template",
        format!(
            "order={order}|name={}|id={}|component={}",
            template.name, template.id, template.component
        ),
    )?);
    for (initial_order, initial) in template.initial_properties.iter().enumerate() {
        collect_initial_property(template.id, initial_order, initial, records)?;
    }
    for (child_order, child) in template.children.iter().enumerate() {
        let (kind, anchor, reference, reference_value) = match *child {
            ResolvedChildV1::Static { template, anchor } => {
                ("static-child", anchor, "target", template)
            }
            ResolvedChildV1::Region { region, anchor } => {
                ("region-child", anchor, "region", region)
            }
        };
        records.push(Record::new(
            anchor,
            kind,
            format!(
                "template={}|order={child_order}|{reference}={reference_value}",
                template.id,
            ),
        )?);
    }
    Ok(())
}

fn collect_initial_property(
    template: u32,
    order: usize,
    initial: &ResolvedInitialPropertyV1,
    records: &mut Vec<Record>,
) -> Result<(), InvalidRecord> {
    records.push(Record::new(
        initial.anchor,
        "initial-property",
        format!(
            "template={template}|order={order}|property={}|value={}",
            initial.property,
            value_name(&initial.value)
        ),
    )?);
    Ok(())
}

fn collect_region(
    order: usize,
    region: &ResolvedRegionV1,
    records: &mut Vec<Record>,
) -> Result<(), InvalidRecord> {
    validate_name(&region.name)?;
    records.push(Record::new(
        region.anchor,
        "region",
        format!(
            "order={order}|name={}|id={}|owner={}|repeat={}|invalidates={}",
            region.name,
            region.id,
            region.owner,
            region.repeat_body,
            invalidation_name(region.invalidation),
        ),
    )?);
    for (key_order, key) in region.initial_keys.iter().enumerate() {
        records.push(Record::new(
            key.anchor,
            "initial-key",
            format!("region={}|order={key_order}|key={}", region.id, key.value),
        )?);
    }
    Ok(())
}

fn collect_style_assignment(
    order: usize,
    assignment: &ResolvedStyleAssignmentV1,
    records: &mut Vec<Record>,
) -> Result<(), InvalidRecord> {
    records.push(Record::new(
        assignment.anchor,
        "style-assignment",
        format!(
            "order={order}|target={}|property={}|value={}",
            assignment.target,
            assignment.property,
            value_name(&assignment.value),
        ),
    )?);
    Ok(())
}

fn schema_identity(document: &ResolvedDocumentV1) -> String {
    format!(
        "namespace={}|revision={}",
        document.schema.namespace, document.schema.revision
    )
}
