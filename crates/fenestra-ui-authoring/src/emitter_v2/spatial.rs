mod geometry;
mod items;
mod layout;
mod value;

use fenestra_ui_ir::prototype::{
    SpatialImageDeclarationV2, SpatialNodeDeclarationV2, SpatialProgramV2,
};
use proc_macro2::TokenStream;

use crate::emitter::builder::{array_into, ir_call, ir_path, u8_literal, u32_literal, u64_literal};

use self::geometry::{brushes, shapes};
use self::items::{clips, hit_items, paint_items, semantic_items};
use self::layout::{container, placement, viewport_container};
use self::value::{field_image, field_node, field_template, source_span};

pub(super) fn spatial(program: &SpatialProgramV2) -> TokenStream {
    ir_call(
        &["SpatialProgramV2", "new"],
        vec![
            ir_path(&["SUPPORTED_SPATIAL_FORMAT"]),
            ir_call(
                &["SchemaNamespace", "new"],
                vec![u64_literal(program.schema_namespace().get())],
                false,
            ),
            ir_call(
                &["SchemaRevision", "new"],
                vec![u32_literal(program.schema_revision().get())],
                false,
            ),
            viewport_container(program.viewport_container()),
            records(program.nodes().iter().map(node).collect()),
            records(program.images().iter().map(image).collect()),
            source_span(program.span()),
        ],
        true,
    )
}

fn node(node: &SpatialNodeDeclarationV2) -> TokenStream {
    ir_call(
        &["SpatialNodeDeclarationV2", "new"],
        vec![
            field_node(node.symbol()),
            field_template(node.template()),
            layout::node_parent(node.parent()),
            placement(node.placement()),
            container(node.container()),
            shapes(node.shapes()),
            brushes(node.brushes()),
            clips(node.clips()),
            paint_items(node.paint_items()),
            hit_items(node.hit_items()),
            semantic_items(node.semantic_items()),
            source_span(node.span()),
        ],
        true,
    )
}

fn image(image: &SpatialImageDeclarationV2) -> TokenStream {
    ir_call(
        &["SpatialImageDeclarationV2", "new"],
        vec![
            field_image(image.symbol()),
            value::field_u32(image.width()),
            value::field_u32(image.height()),
            value::field_u32(image.stride()),
            array_into(
                image.bytes().iter().copied().map(u8_literal).collect(),
                image.bytes().len() > 1,
            ),
            source_span(image.span()),
        ],
        true,
    )
}

pub(super) fn records(records: Vec<TokenStream>) -> TokenStream {
    let trailing = records.len() > 1;
    array_into(records, trailing)
}
