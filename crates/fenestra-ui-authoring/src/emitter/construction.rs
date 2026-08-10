use proc_macro2::TokenStream;

use crate::resolved::{
    ResolvedChildV1, ResolvedConstructionV1, ResolvedInitialKeyV1, ResolvedInitialPropertyV1,
    ResolvedRegionV1, ResolvedTemplateV1,
};

use super::builder::{array_into, ir_call, ir_path, u32_literal, u64_literal};
use super::value::{invalidation, property_value, schema_namespace, schema_revision, span};

pub(super) fn construction(
    construction: &ResolvedConstructionV1,
    namespace: u64,
    revision: u32,
) -> TokenStream {
    let templates = construction
        .templates
        .iter()
        .map(template)
        .collect::<Vec<_>>();
    let regions = construction.regions.iter().map(region).collect::<Vec<_>>();
    ir_call(
        &["ConstructionProgram", "new"],
        vec![
            ir_path(&["SUPPORTED_CONSTRUCTION_FORMAT"]),
            schema_namespace(namespace),
            schema_revision(revision),
            record_array(templates),
            record_array(regions),
            span(construction.anchor),
        ],
        true,
    )
}

fn template(template: &ResolvedTemplateV1) -> TokenStream {
    let properties = template
        .initial_properties
        .iter()
        .map(initial_property)
        .collect::<Vec<_>>();
    let children = template.children.iter().map(child).collect::<Vec<_>>();
    ir_call(
        &["TemplateNode", "new"],
        vec![
            template_id(template.id),
            component_id(template.component),
            record_array(properties),
            record_array(children),
            span(template.anchor),
        ],
        true,
    )
}

fn initial_property(property: &ResolvedInitialPropertyV1) -> TokenStream {
    ir_call(
        &["InitialProperty", "new"],
        vec![
            property_id(property.property),
            property_value(&property.value),
            span(property.anchor),
        ],
        true,
    )
}

fn child(child: &ResolvedChildV1) -> TokenStream {
    match child {
        ResolvedChildV1::Static { template, anchor } => ir_call(
            &["ChildSlot", "static_node"],
            vec![template_id(*template), span(*anchor)],
            true,
        ),
        ResolvedChildV1::Region { region, anchor } => ir_call(
            &["ChildSlot", "region"],
            vec![region_id(*region), span(*anchor)],
            true,
        ),
    }
}

fn region(region: &ResolvedRegionV1) -> TokenStream {
    let keys = region
        .initial_keys
        .iter()
        .map(initial_key)
        .collect::<Vec<_>>();
    ir_call(
        &["StructuralRegion", "new"],
        vec![
            region_id(region.id),
            template_id(region.owner),
            template_id(region.repeat_body),
            record_array(keys),
            invalidation(region.invalidation),
            span(region.anchor),
        ],
        true,
    )
}

fn initial_key(key: &ResolvedInitialKeyV1) -> TokenStream {
    ir_call(
        &["InitialKey", "new"],
        vec![u64_literal(key.value), span(key.anchor)],
        true,
    )
}

fn component_id(value: u32) -> TokenStream {
    ir_call(&["ComponentTypeId", "new"], vec![u32_literal(value)], false)
}

fn property_id(value: u32) -> TokenStream {
    ir_call(&["PropertyId", "new"], vec![u32_literal(value)], false)
}

fn template_id(value: u32) -> TokenStream {
    ir_call(&["TemplateNodeId", "new"], vec![u32_literal(value)], false)
}

fn region_id(value: u32) -> TokenStream {
    ir_call(
        &["StructuralRegionId", "new"],
        vec![u32_literal(value)],
        false,
    )
}

fn record_array(records: Vec<TokenStream>) -> TokenStream {
    let trailing = records.len() > 1;
    array_into(records, trailing)
}
