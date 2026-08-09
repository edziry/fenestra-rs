use fenestra_ui_ir::prototype::{
    ChildSlot, ComponentSchema, ComponentTypeId, ConstructionProgram, InitialKey,
    InvalidationClass, InvalidationSet, SUPPORTED_CONSTRUCTION_FORMAT, SUPPORTED_SCHEMA_FORMAT,
    SchemaManifest, SchemaNamespace, SchemaRevision, SourceSpan, StructuralRegion,
    StructuralRegionId, TemplateNode, TemplateNodeId, ValidatedConstruction, ValidationLimits,
    validate_construction, validate_schema,
};

pub const OUTER_REGION: StructuralRegionId = StructuralRegionId::new(0);

pub fn hidden_overflow_construction() -> ValidatedConstruction {
    let span = SourceSpan::synthetic();
    let namespace = SchemaNamespace::new(73);
    let revision = SchemaRevision::new(1);
    let component = ComponentTypeId::new(0);
    let binary_depth = usize::BITS as usize;
    let manifest = SchemaManifest::new(
        SUPPORTED_SCHEMA_FORMAT,
        namespace,
        revision,
        vec![ComponentSchema::new(component, Vec::new(), span)],
        span,
    );

    let mut templates = Vec::with_capacity(binary_depth + 2);
    let mut regions = Vec::with_capacity(binary_depth + 1);
    for index in 0..=binary_depth {
        let region = StructuralRegionId::new(index as u32);
        templates.push(TemplateNode::new(
            TemplateNodeId::new(index as u32),
            component,
            Vec::new(),
            vec![ChildSlot::region(region, span)],
            span,
        ));
        let keys = if index == 0 {
            Vec::new()
        } else {
            vec![InitialKey::new(0, span), InitialKey::new(1, span)]
        };
        regions.push(StructuralRegion::new(
            region,
            TemplateNodeId::new(index as u32),
            TemplateNodeId::new((index + 1) as u32),
            keys,
            InvalidationSet::from_class(InvalidationClass::Structure),
            span,
        ));
    }
    templates.push(TemplateNode::new(
        TemplateNodeId::new((binary_depth + 1) as u32),
        component,
        Vec::new(),
        Vec::new(),
        span,
    ));

    let program = ConstructionProgram::new(
        SUPPORTED_CONSTRUCTION_FORMAT,
        namespace,
        revision,
        templates,
        regions,
        span,
    );
    let limits = ValidationLimits::new(
        1,
        0,
        binary_depth + 2,
        binary_depth + 1,
        binary_depth + 1,
        0,
        binary_depth * 2,
        binary_depth + 2,
        1,
    );
    let schema =
        validate_schema(manifest, limits).expect("overflow fixture schema should validate");
    validate_construction(&schema, program, limits)
        .expect("uninstantiated overflow fixture should validate")
}
