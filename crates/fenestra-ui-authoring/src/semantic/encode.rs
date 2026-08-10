use crate::resolved::{
    ResolvedChildV1, ResolvedDocumentV1, ResolvedInitialPropertyV1, ResolvedPropertyV1,
    ResolvedRegionV1, ResolvedStyleAssignmentV1, ResolvedTemplateV1,
};

use super::value::{invalidation_name, value_name, value_type_name};
use super::{
    SemanticArtifactErrorKindV1, SemanticArtifactErrorV1, SemanticArtifactLimitKindV1,
    SemanticArtifactLimitsV1,
};

pub(super) fn encode_resolved_v1(
    resolved: &ResolvedDocumentV1,
    limits: SemanticArtifactLimitsV1,
) -> Result<Box<str>, SemanticArtifactErrorV1> {
    let count = record_count(resolved).ok_or_else(invalid_document)?;
    if count > limits.limit(SemanticArtifactLimitKindV1::Records) {
        return Err(limit_exceeded(SemanticArtifactLimitKindV1::Records));
    }

    let mut records = Vec::with_capacity(count);
    collect_document(resolved, &mut records)?;
    if records.len() != count {
        return Err(invalid_document());
    }
    records.sort_by_key(|record| record.anchor);
    for (ordinal, record) in records.iter().enumerate() {
        let ordinal = u32::try_from(ordinal).map_err(|_| invalid_document())?;
        if record.anchor != ordinal {
            return Err(invalid_document());
        }
    }

    let mut writer = ArtifactWriterV1::new(limits);
    writer.push_line(&format!("fenestra-authoring-semantics|1|records={count}"))?;
    for record in records {
        writer.push_line(&record.line)?;
    }
    Ok(writer.finish().into_boxed_str())
}

fn collect_document(
    document: &ResolvedDocumentV1,
    records: &mut Vec<RecordV1>,
) -> Result<(), SemanticArtifactErrorV1> {
    records.push(RecordV1::new(
        document.document_anchor,
        "document",
        format!("format={}", document.format),
    )?);
    records.push(RecordV1::new(
        document.schema.anchor,
        "schema",
        format!(
            "namespace={}|revision={}",
            document.schema.namespace, document.schema.revision
        ),
    )?);

    for (component_order, component) in document.schema.components.iter().enumerate() {
        validate_name(&component.name)?;
        records.push(RecordV1::new(
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

    records.push(RecordV1::new(
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

    records.push(RecordV1::new(
        document.style.anchor,
        "style",
        schema_identity(document),
    )?);
    for (order, assignment) in document.style.assignments.iter().enumerate() {
        collect_style_assignment(order, assignment, records)?;
    }
    Ok(())
}

fn collect_property(
    component: u32,
    order: usize,
    property: &ResolvedPropertyV1,
    records: &mut Vec<RecordV1>,
) -> Result<(), SemanticArtifactErrorV1> {
    validate_name(&property.name)?;
    records.push(RecordV1::new(
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
    records: &mut Vec<RecordV1>,
) -> Result<(), SemanticArtifactErrorV1> {
    validate_name(&template.name)?;
    records.push(RecordV1::new(
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
        let (kind, reference, reference_value) = match *child {
            ResolvedChildV1::Static { template, .. } => ("static-child", "target", template),
            ResolvedChildV1::Region { region, .. } => ("region-child", "region", region),
        };
        records.push(RecordV1::new(
            child_anchor(child),
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
    records: &mut Vec<RecordV1>,
) -> Result<(), SemanticArtifactErrorV1> {
    records.push(RecordV1::new(
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
    records: &mut Vec<RecordV1>,
) -> Result<(), SemanticArtifactErrorV1> {
    validate_name(&region.name)?;
    records.push(RecordV1::new(
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
        records.push(RecordV1::new(
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
    records: &mut Vec<RecordV1>,
) -> Result<(), SemanticArtifactErrorV1> {
    records.push(RecordV1::new(
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

fn child_anchor(child: &ResolvedChildV1) -> u32 {
    match *child {
        ResolvedChildV1::Static { anchor, .. } | ResolvedChildV1::Region { anchor, .. } => anchor,
    }
}

fn validate_name(name: &str) -> Result<(), SemanticArtifactErrorV1> {
    let mut bytes = name.bytes();
    let valid_start = bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_');
    if !valid_start
        || !bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        || name.contains('|')
    {
        return Err(invalid_document());
    }
    Ok(())
}

fn record_count(document: &ResolvedDocumentV1) -> Option<usize> {
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

struct RecordV1 {
    anchor: u32,
    line: String,
}

impl RecordV1 {
    fn new(anchor: u32, kind: &str, fields: String) -> Result<Self, SemanticArtifactErrorV1> {
        let end = anchor.checked_add(1).ok_or_else(invalid_document)?;
        Ok(Self {
            anchor,
            line: format!("record|{anchor}|{kind}|span={anchor}:{end}|{fields}"),
        })
    }
}

struct ArtifactWriterV1 {
    output: String,
    limits: SemanticArtifactLimitsV1,
}

impl ArtifactWriterV1 {
    const fn new(limits: SemanticArtifactLimitsV1) -> Self {
        Self {
            output: String::new(),
            limits,
        }
    }

    fn push_line(&mut self, line: &str) -> Result<(), SemanticArtifactErrorV1> {
        if line.len() > self.limits.limit(SemanticArtifactLimitKindV1::LineBytes) {
            return Err(limit_exceeded(SemanticArtifactLimitKindV1::LineBytes));
        }
        let bytes = self
            .output
            .len()
            .checked_add(line.len())
            .and_then(|value| value.checked_add(1))
            .ok_or_else(invalid_document)?;
        if bytes
            > self
                .limits
                .limit(SemanticArtifactLimitKindV1::ArtifactBytes)
        {
            return Err(limit_exceeded(SemanticArtifactLimitKindV1::ArtifactBytes));
        }
        if !line.bytes().all(|byte| (0x20..=0x7e).contains(&byte)) {
            return Err(invalid_document());
        }
        self.output.push_str(line);
        self.output.push('\n');
        Ok(())
    }

    fn finish(self) -> String {
        self.output
    }
}

fn limit_exceeded(limit: SemanticArtifactLimitKindV1) -> SemanticArtifactErrorV1 {
    SemanticArtifactErrorV1::new(SemanticArtifactErrorKindV1::LimitExceeded(limit))
}

fn invalid_document() -> SemanticArtifactErrorV1 {
    SemanticArtifactErrorV1::new(SemanticArtifactErrorKindV1::InvalidCompiledDocument)
}
