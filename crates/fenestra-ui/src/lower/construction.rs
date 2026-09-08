use fenestra_ui_ir::prototype::{
    ChildSlot, ComponentSchema, ComponentTypeId, ConstructionProgram, InitialProperty,
    InvalidationClass, InvalidationSet, PropertySchema, PropertyValue,
    SUPPORTED_CONSTRUCTION_FORMAT, SUPPORTED_SCHEMA_FORMAT, SUPPORTED_STYLE_FORMAT, SchemaManifest,
    StyleProgram, StyleValidationLimits, TemplateNode, TemplateNodeId, ValidatedStyleProgram,
    ValidationLimits, validate_construction, validate_schema, validate_style,
};

use super::{
    BACKGROUND, FlatView, INPUT, NAMESPACE, REVISION, SPAN, VIEW_REVISION, invalid_program,
};
use crate::{Error, Size, Style};

const COMPONENT: ComponentTypeId = ComponentTypeId::new(0);

pub(super) fn build(
    flat: &FlatView<'_>,
    sizes: &[Size],
    property_slots: usize,
) -> Result<ValidatedStyleProgram, Error> {
    let n = flat.nodes.len();
    let limits = ValidationLimits::new(1, 7, n, 0, n - 1, property_slots, 0, flat.depth, n);
    let properties = Style::new()
        .values(Size::new(64, 64))
        .into_iter()
        .chain([(VIEW_REVISION, PropertyValue::ScalarI32(0))])
        .map(|(property, default)| {
            let invalidation = if property == BACKGROUND || property == VIEW_REVISION {
                InvalidationSet::from_class(InvalidationClass::Paint)
            } else if property == INPUT {
                InvalidationSet::from_class(InvalidationClass::HitTest)
            } else {
                InvalidationSet::from_class(InvalidationClass::Layout)
                    .union(InvalidationSet::from_class(InvalidationClass::Paint))
                    .union(InvalidationSet::from_class(InvalidationClass::HitTest))
            };
            PropertySchema::new(property, default.value_type(), default, invalidation, SPAN)
        })
        .collect();
    let manifest = SchemaManifest::new(
        SUPPORTED_SCHEMA_FORMAT,
        NAMESPACE,
        REVISION,
        vec![ComponentSchema::new(COMPONENT, properties, SPAN)],
        SPAN,
    );
    let schema = validate_schema(manifest, limits).map_err(invalid_program)?;
    let nodes = flat
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let properties = node
                .element
                .style
                .values(sizes[index])
                .into_iter()
                .chain([(VIEW_REVISION, PropertyValue::ScalarI32(0))])
                .map(|(property, value)| InitialProperty::new(property, value, SPAN))
                .collect();
            let children = node
                .children
                .iter()
                .map(|&id| ChildSlot::static_node(TemplateNodeId::new(id as u32), SPAN))
                .collect();
            TemplateNode::new(
                TemplateNodeId::new(index as u32),
                COMPONENT,
                properties,
                children,
                SPAN,
            )
        })
        .collect();
    let construction = ConstructionProgram::new(
        SUPPORTED_CONSTRUCTION_FORMAT,
        NAMESPACE,
        REVISION,
        nodes,
        Vec::new(),
        SPAN,
    );
    let construction =
        validate_construction(&schema, construction, limits).map_err(invalid_program)?;
    let style = StyleProgram::new(
        SUPPORTED_STYLE_FORMAT,
        NAMESPACE,
        REVISION,
        Vec::new(),
        SPAN,
    );
    validate_style(&construction, style, StyleValidationLimits::new(0)).map_err(invalid_program)
}
