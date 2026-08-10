use fenestra_ui_ir::prototype::{
    InputPolicy, InvalidationClass, InvalidationSet, PropertyValue, ValueType,
};
use proc_macro2::TokenStream;

use super::builder::{
    array, bool_literal, i32_literal, ir_call, ir_path, method_call, u8_literal, u32_literal,
    u64_literal,
};

pub(super) fn value_type(value: ValueType) -> TokenStream {
    let variant = match value {
        ValueType::Bool => "Bool",
        ValueType::ScalarI32 => "ScalarI32",
        ValueType::Rgba8 => "Rgba8",
        ValueType::InputPolicy => "InputPolicy",
    };
    ir_path(&["ValueType", variant])
}

pub(super) fn property_value(value: &PropertyValue) -> TokenStream {
    match value {
        PropertyValue::Bool(value) => ir_call(
            &["PropertyValue", "Bool"],
            vec![bool_literal(*value)],
            false,
        ),
        PropertyValue::ScalarI32(value) => ir_call(
            &["PropertyValue", "ScalarI32"],
            vec![i32_literal(*value)],
            false,
        ),
        PropertyValue::Rgba8(channels) => ir_call(
            &["PropertyValue", "Rgba8"],
            vec![array(
                channels.iter().copied().map(u8_literal).collect(),
                false,
            )],
            false,
        ),
        PropertyValue::InputPolicy(policy) => ir_call(
            &["PropertyValue", "InputPolicy"],
            vec![input_policy(*policy)],
            true,
        ),
    }
}

pub(super) fn invalidation(set: InvalidationSet) -> TokenStream {
    let mut classes = set.iter();
    let Some(first) = classes.next() else {
        return ir_path(&["InvalidationSet", "NONE"]);
    };
    let mut expression = invalidation_class(first);
    for class in classes {
        expression = method_call(expression, "union", vec![invalidation_class(class)], true);
    }
    expression
}

pub(super) fn span(anchor: u32) -> TokenStream {
    ir_call(
        &["SourceSpan", "bytes"],
        vec![
            ir_call(&["SourceId", "new"], vec![u32_literal(0)], false),
            u32_literal(anchor),
            u32_literal(anchor.saturating_add(1)),
        ],
        true,
    )
}

fn input_policy(policy: InputPolicy) -> TokenStream {
    let variant = match policy {
        InputPolicy::Accept => "Accept",
        InputPolicy::Ignore => "Ignore",
    };
    ir_path(&["InputPolicy", variant])
}

fn invalidation_class(class: InvalidationClass) -> TokenStream {
    let variant = match class {
        InvalidationClass::Structure => "Structure",
        InvalidationClass::StyleMatch => "StyleMatch",
        InvalidationClass::Intrinsic => "Intrinsic",
        InvalidationClass::Layout => "Layout",
        InvalidationClass::Semantics => "Semantics",
        InvalidationClass::HitTest => "HitTest",
        InvalidationClass::Paint => "Paint",
        InvalidationClass::Composition => "Composition",
        InvalidationClass::Surface => "Surface",
    };
    ir_call(
        &["InvalidationSet", "from_class"],
        vec![ir_path(&["InvalidationClass", variant])],
        true,
    )
}

pub(super) fn schema_namespace(value: u64) -> TokenStream {
    ir_call(&["SchemaNamespace", "new"], vec![u64_literal(value)], false)
}

pub(super) fn schema_revision(value: u32) -> TokenStream {
    ir_call(&["SchemaRevision", "new"], vec![u32_literal(value)], false)
}
