use fenestra_ui_ir::prototype::{
    InputPolicy, PropertyId, SourceSpan, SpatialBindingV2, SpatialBrushSymbolV2,
    SpatialClipAddressV2, SpatialClipSymbolV2, SpatialFieldV2, SpatialImageSymbolV2,
    SpatialNodeSymbolV2, SpatialShapeSymbolV2, TemplateNodeId,
};
use proc_macro2::{Ident, Span, TokenStream, TokenTree};

use crate::emitter::builder::{
    array, call, i32_literal, i64_literal, ir_call, ir_path, u8_literal, u16_literal, u32_literal,
};

pub(super) fn source_span(value: SourceSpan) -> TokenStream {
    match value {
        SourceSpan::Synthetic => ir_call(&["SourceSpan", "synthetic"], Vec::new(), false),
        SourceSpan::Bytes { source, start, end } => ir_call(
            &["SourceSpan", "bytes"],
            vec![
                ir_call(&["SourceId", "new"], vec![u32_literal(source.get())], false),
                u32_literal(start),
                u32_literal(end),
            ],
            true,
        ),
    }
}

pub(super) fn field_i32(value: SpatialFieldV2<i32>) -> TokenStream {
    field(value, |value| i32_literal(*value))
}

pub(super) fn field_u8(value: SpatialFieldV2<u8>) -> TokenStream {
    field(value, |value| u8_literal(*value))
}

pub(super) fn field_u16(value: SpatialFieldV2<u16>) -> TokenStream {
    field(value, |value| u16_literal(*value))
}

pub(super) fn field_u32(value: SpatialFieldV2<u32>) -> TokenStream {
    field(value, |value| u32_literal(*value))
}

pub(super) fn field_i32_binding(value: SpatialFieldV2<SpatialBindingV2<i32>>) -> TokenStream {
    field(value, |value| binding(value, |value| i32_literal(*value)))
}

pub(super) fn field_i64_binding(value: SpatialFieldV2<SpatialBindingV2<i64>>) -> TokenStream {
    field(value, |value| binding(value, |value| i64_literal(*value)))
}

pub(super) fn field_color_binding(value: SpatialFieldV2<SpatialBindingV2<[u8; 4]>>) -> TokenStream {
    field(value, |value| {
        binding(value, |channels| {
            array(channels.iter().copied().map(u8_literal).collect(), false)
        })
    })
}

pub(super) fn field_input_binding(
    value: SpatialFieldV2<SpatialBindingV2<InputPolicy>>,
) -> TokenStream {
    field(value, |value| binding(value, |value| input_policy(*value)))
}

macro_rules! symbol_field {
    ($name:ident, $symbol:ty, $path:literal) => {
        pub(super) fn $name(value: SpatialFieldV2<$symbol>) -> TokenStream {
            field(value, |value| {
                ir_call(&[$path, "new"], vec![u32_literal(value.get())], false)
            })
        }
    };
}

symbol_field!(field_node, SpatialNodeSymbolV2, "SpatialNodeSymbolV2");
symbol_field!(field_shape, SpatialShapeSymbolV2, "SpatialShapeSymbolV2");
symbol_field!(field_brush, SpatialBrushSymbolV2, "SpatialBrushSymbolV2");
symbol_field!(field_clip, SpatialClipSymbolV2, "SpatialClipSymbolV2");
symbol_field!(field_image, SpatialImageSymbolV2, "SpatialImageSymbolV2");
symbol_field!(field_template, TemplateNodeId, "TemplateNodeId");

pub(super) fn clip_address(value: SpatialClipAddressV2) -> TokenStream {
    ir_call(
        &["SpatialClipAddressV2", "new"],
        vec![field_node(value.owner()), field_clip(value.clip())],
        true,
    )
}

pub(super) fn optional_clip(value: Option<SpatialClipAddressV2>) -> TokenStream {
    option(value.map(clip_address))
}

pub(super) fn option(value: Option<TokenStream>) -> TokenStream {
    match value {
        Some(value) => call(identifier("Some"), vec![value], false),
        None => identifier("None"),
    }
}

fn field<T>(value: SpatialFieldV2<T>, emit: impl FnOnce(&T) -> TokenStream) -> TokenStream {
    ir_call(
        &["SpatialFieldV2", "new"],
        vec![emit(value.value()), source_span(value.span())],
        true,
    )
}

fn binding<T>(value: &SpatialBindingV2<T>, emit: impl FnOnce(&T) -> TokenStream) -> TokenStream {
    match value {
        SpatialBindingV2::Literal(value) => {
            ir_call(&["SpatialBindingV2", "Literal"], vec![emit(value)], false)
        }
        SpatialBindingV2::Property(property) => ir_call(
            &["SpatialBindingV2", "Property"],
            vec![property_id(*property)],
            false,
        ),
    }
}

fn property_id(value: PropertyId) -> TokenStream {
    ir_call(
        &["PropertyId", "new"],
        vec![u32_literal(value.get())],
        false,
    )
}

fn input_policy(value: InputPolicy) -> TokenStream {
    let variant = match value {
        InputPolicy::Accept => "Accept",
        InputPolicy::Ignore => "Ignore",
    };
    ir_path(&["InputPolicy", variant])
}

fn identifier(value: &str) -> TokenStream {
    TokenStream::from(TokenTree::Ident(Ident::new(value, Span::call_site())))
}
