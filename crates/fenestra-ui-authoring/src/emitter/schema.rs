use proc_macro2::TokenStream;

use crate::resolved::{ResolvedComponentV1, ResolvedPropertyV1, ResolvedSchemaV1};

use super::builder::{array_into, ir_call, ir_path, u32_literal};
use super::value::{
    invalidation, property_value, schema_namespace, schema_revision, span, value_type,
};

pub(super) fn schema(schema: &ResolvedSchemaV1) -> TokenStream {
    let components = schema.components.iter().map(component).collect::<Vec<_>>();
    ir_call(
        &["SchemaManifest", "new"],
        vec![
            ir_path(&["SUPPORTED_SCHEMA_FORMAT"]),
            schema_namespace(schema.namespace),
            schema_revision(schema.revision),
            record_array(components),
            span(schema.anchor),
        ],
        true,
    )
}

fn component(component: &ResolvedComponentV1) -> TokenStream {
    let properties = component
        .properties
        .iter()
        .map(property)
        .collect::<Vec<_>>();
    ir_call(
        &["ComponentSchema", "new"],
        vec![
            ir_call(
                &["ComponentTypeId", "new"],
                vec![u32_literal(component.id)],
                false,
            ),
            record_array(properties),
            span(component.anchor),
        ],
        true,
    )
}

fn property(property: &ResolvedPropertyV1) -> TokenStream {
    ir_call(
        &["PropertySchema", "new"],
        vec![
            ir_call(
                &["PropertyId", "new"],
                vec![u32_literal(property.id)],
                false,
            ),
            value_type(property.value_type),
            property_value(&property.default),
            invalidation(property.invalidation),
            span(property.anchor),
        ],
        true,
    )
}

fn record_array(records: Vec<TokenStream>) -> TokenStream {
    let trailing = records.len() > 1;
    array_into(records, trailing)
}
