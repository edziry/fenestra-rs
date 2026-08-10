use fenestra_ui_ir::prototype::{
    ChildSlot, ComponentSchema, ComponentTypeId, ConstructionProgram, InitialKey, InitialProperty,
    InvalidationSet, PropertyId, PropertySchema, PropertyValue, SUPPORTED_CONSTRUCTION_FORMAT,
    SUPPORTED_SCHEMA_FORMAT, SUPPORTED_STYLE_FORMAT, SchemaManifest, SchemaNamespace,
    SchemaRevision, SourceId, SourceSpan, StructuralRegion, StructuralRegionId, StyleAssignment,
    StyleProgram, TemplateNode, TemplateNodeId, ValueType,
};

pub(crate) struct ResolvedDocumentV1 {
    pub(crate) format: u32,
    pub(crate) document_anchor: u32,
    pub(crate) schema: ResolvedSchemaV1,
    pub(crate) construction: ResolvedConstructionV1,
    pub(crate) style: ResolvedStyleV1,
}

pub(crate) struct ResolvedSchemaV1 {
    pub(crate) namespace: u64,
    pub(crate) revision: u32,
    pub(crate) components: Vec<ResolvedComponentV1>,
    pub(crate) anchor: u32,
}

pub(crate) struct ResolvedComponentV1 {
    pub(crate) name: Box<str>,
    pub(crate) id: u32,
    pub(crate) properties: Vec<ResolvedPropertyV1>,
    pub(crate) anchor: u32,
}

pub(crate) struct ResolvedPropertyV1 {
    pub(crate) name: Box<str>,
    pub(crate) id: u32,
    pub(crate) value_type: ValueType,
    pub(crate) default: PropertyValue,
    pub(crate) invalidation: InvalidationSet,
    pub(crate) anchor: u32,
}

pub(crate) struct ResolvedConstructionV1 {
    pub(crate) templates: Vec<ResolvedTemplateV1>,
    pub(crate) regions: Vec<ResolvedRegionV1>,
    pub(crate) anchor: u32,
}

pub(crate) struct ResolvedTemplateV1 {
    pub(crate) name: Box<str>,
    pub(crate) id: u32,
    pub(crate) component: u32,
    pub(crate) initial_properties: Vec<ResolvedInitialPropertyV1>,
    pub(crate) children: Vec<ResolvedChildV1>,
    pub(crate) anchor: u32,
}

pub(crate) struct ResolvedInitialPropertyV1 {
    pub(crate) property: u32,
    pub(crate) value: PropertyValue,
    pub(crate) anchor: u32,
}

pub(crate) enum ResolvedChildV1 {
    Static { template: u32, anchor: u32 },
    Region { region: u32, anchor: u32 },
}

pub(crate) struct ResolvedRegionV1 {
    pub(crate) name: Box<str>,
    pub(crate) id: u32,
    pub(crate) owner: u32,
    pub(crate) repeat_body: u32,
    pub(crate) initial_keys: Vec<ResolvedInitialKeyV1>,
    pub(crate) invalidation: InvalidationSet,
    pub(crate) anchor: u32,
}

pub(crate) struct ResolvedInitialKeyV1 {
    pub(crate) value: u64,
    pub(crate) anchor: u32,
}

pub(crate) struct ResolvedStyleV1 {
    pub(crate) assignments: Vec<ResolvedStyleAssignmentV1>,
    pub(crate) anchor: u32,
}

pub(crate) struct ResolvedStyleAssignmentV1 {
    pub(crate) target: u32,
    pub(crate) property: u32,
    pub(crate) value: PropertyValue,
    pub(crate) anchor: u32,
}

impl ResolvedDocumentV1 {
    pub(crate) fn raw_programs(&self) -> (SchemaManifest, ConstructionProgram, StyleProgram) {
        let namespace = SchemaNamespace::new(self.schema.namespace);
        let revision = SchemaRevision::new(self.schema.revision);
        (
            self.schema.raw(),
            self.construction.raw(namespace, revision),
            self.style.raw(namespace, revision),
        )
    }

    pub(crate) const fn authoring_format(&self) -> u32 {
        self.format
    }

    pub(crate) const fn document_anchor(&self) -> u32 {
        self.document_anchor
    }

    pub(crate) fn semantic_records(&self) -> usize {
        let properties = self
            .schema
            .components
            .iter()
            .map(|component| component.properties.len())
            .sum::<usize>();
        let templates = self
            .construction
            .templates
            .iter()
            .map(|template| 1 + template.initial_properties.len() + template.children.len())
            .sum::<usize>();
        let regions = self
            .construction
            .regions
            .iter()
            .map(|region| 1 + region.initial_keys.len())
            .sum::<usize>();
        4 + self.schema.components.len()
            + properties
            + templates
            + regions
            + self.style.assignments.len()
    }

    pub(crate) fn authored_name_bytes(&self) -> usize {
        let schema_names = self
            .schema
            .components
            .iter()
            .map(|component| {
                component.name.len()
                    + component
                        .properties
                        .iter()
                        .map(|property| property.name.len())
                        .sum::<usize>()
            })
            .sum::<usize>();
        let construction_names = self
            .construction
            .templates
            .iter()
            .map(|template| template.name.len())
            .sum::<usize>()
            + self
                .construction
                .regions
                .iter()
                .map(|region| region.name.len())
                .sum::<usize>();
        schema_names + construction_names
    }
}

impl ResolvedSchemaV1 {
    fn raw(&self) -> SchemaManifest {
        SchemaManifest::new(
            SUPPORTED_SCHEMA_FORMAT,
            SchemaNamespace::new(self.namespace),
            SchemaRevision::new(self.revision),
            self.components
                .iter()
                .map(ResolvedComponentV1::raw)
                .collect(),
            logical_span(self.anchor),
        )
    }
}

impl ResolvedComponentV1 {
    fn raw(&self) -> ComponentSchema {
        ComponentSchema::new(
            ComponentTypeId::new(self.id),
            self.properties
                .iter()
                .map(ResolvedPropertyV1::raw)
                .collect(),
            logical_span(self.anchor),
        )
    }
}

impl ResolvedPropertyV1 {
    fn raw(&self) -> PropertySchema {
        PropertySchema::new(
            PropertyId::new(self.id),
            self.value_type,
            self.default.clone(),
            self.invalidation,
            logical_span(self.anchor),
        )
    }
}

impl ResolvedConstructionV1 {
    fn raw(&self, namespace: SchemaNamespace, revision: SchemaRevision) -> ConstructionProgram {
        ConstructionProgram::new(
            SUPPORTED_CONSTRUCTION_FORMAT,
            namespace,
            revision,
            self.templates.iter().map(ResolvedTemplateV1::raw).collect(),
            self.regions.iter().map(ResolvedRegionV1::raw).collect(),
            logical_span(self.anchor),
        )
    }
}

impl ResolvedTemplateV1 {
    fn raw(&self) -> TemplateNode {
        TemplateNode::new(
            TemplateNodeId::new(self.id),
            ComponentTypeId::new(self.component),
            self.initial_properties
                .iter()
                .map(ResolvedInitialPropertyV1::raw)
                .collect(),
            self.children.iter().map(ResolvedChildV1::raw).collect(),
            logical_span(self.anchor),
        )
    }
}

impl ResolvedInitialPropertyV1 {
    fn raw(&self) -> InitialProperty {
        InitialProperty::new(
            PropertyId::new(self.property),
            self.value.clone(),
            logical_span(self.anchor),
        )
    }
}

impl ResolvedChildV1 {
    fn raw(&self) -> ChildSlot {
        match *self {
            Self::Static { template, anchor } => {
                ChildSlot::static_node(TemplateNodeId::new(template), logical_span(anchor))
            }
            Self::Region { region, anchor } => {
                ChildSlot::region(StructuralRegionId::new(region), logical_span(anchor))
            }
        }
    }
}

impl ResolvedRegionV1 {
    fn raw(&self) -> StructuralRegion {
        StructuralRegion::new(
            StructuralRegionId::new(self.id),
            TemplateNodeId::new(self.owner),
            TemplateNodeId::new(self.repeat_body),
            self.initial_keys
                .iter()
                .map(ResolvedInitialKeyV1::raw)
                .collect(),
            self.invalidation,
            logical_span(self.anchor),
        )
    }
}

impl ResolvedInitialKeyV1 {
    fn raw(&self) -> InitialKey {
        InitialKey::new(self.value, logical_span(self.anchor))
    }
}

impl ResolvedStyleV1 {
    fn raw(&self, namespace: SchemaNamespace, revision: SchemaRevision) -> StyleProgram {
        StyleProgram::new(
            SUPPORTED_STYLE_FORMAT,
            namespace,
            revision,
            self.assignments
                .iter()
                .map(ResolvedStyleAssignmentV1::raw)
                .collect(),
            logical_span(self.anchor),
        )
    }
}

impl ResolvedStyleAssignmentV1 {
    fn raw(&self) -> StyleAssignment {
        StyleAssignment::new(
            TemplateNodeId::new(self.target),
            PropertyId::new(self.property),
            self.value.clone(),
            logical_span(self.anchor),
        )
    }
}

pub(crate) const fn logical_span(ordinal: u32) -> SourceSpan {
    SourceSpan::bytes(SourceId::new(0), ordinal, ordinal + 1)
}
