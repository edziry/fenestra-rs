use std::fmt;

use fenestra_ui_ir::prototype::{
    InputPolicy, PropertyId, SpatialBindingV2, SpatialBrushSymbolV2, SpatialClipSymbolV2,
    SpatialImageSymbolV2, SpatialNodeSymbolV2, SpatialShapeSymbolV2, TemplateNodeId,
};

pub(super) trait FieldValue {
    fn field_value(&self) -> String;
}

macro_rules! symbol_value {
    ($type:ty, $tag:literal) => {
        impl FieldValue for $type {
            fn field_value(&self) -> String {
                format!(concat!($tag, ":{}"), self.get())
            }
        }
    };
}

symbol_value!(SpatialNodeSymbolV2, "node");
symbol_value!(SpatialImageSymbolV2, "image");
symbol_value!(SpatialShapeSymbolV2, "shape");
symbol_value!(SpatialBrushSymbolV2, "brush");
symbol_value!(SpatialClipSymbolV2, "clip");
symbol_value!(TemplateNodeId, "template");

macro_rules! integer_value {
    ($type:ty, $tag:literal) => {
        impl FieldValue for $type {
            fn field_value(&self) -> String {
                format!(concat!($tag, ":{}"), self)
            }
        }
    };
}

integer_value!(u8, "u8");
integer_value!(u16, "u16");
integer_value!(u32, "u32");
integer_value!(i32, "i32");

impl FieldValue for SpatialBindingV2<i32> {
    fn field_value(&self) -> String {
        binding_value(*self, "i32-literal")
    }
}

impl FieldValue for SpatialBindingV2<i64> {
    fn field_value(&self) -> String {
        binding_value(*self, "fixed16-literal")
    }
}

impl FieldValue for SpatialBindingV2<[u8; 4]> {
    fn field_value(&self) -> String {
        match *self {
            SpatialBindingV2::Literal([red, green, blue, alpha]) => {
                format!("rgba8-literal:{red},{green},{blue},{alpha}")
            }
            SpatialBindingV2::Property(property) => property_value("rgba8", property),
        }
    }
}

impl FieldValue for SpatialBindingV2<InputPolicy> {
    fn field_value(&self) -> String {
        match *self {
            SpatialBindingV2::Literal(InputPolicy::Accept) => {
                "input-policy-literal:accept".to_owned()
            }
            SpatialBindingV2::Literal(InputPolicy::Ignore) => {
                "input-policy-literal:ignore".to_owned()
            }
            SpatialBindingV2::Property(property) => property_value("input-policy", property),
        }
    }
}

fn binding_value<T: fmt::Display + Copy>(binding: SpatialBindingV2<T>, literal: &str) -> String {
    match binding {
        SpatialBindingV2::Literal(value) => format!("{literal}:{value}"),
        SpatialBindingV2::Property(property) => {
            let property_tag = literal.strip_suffix("-literal").unwrap_or(literal);
            property_value(property_tag, property)
        }
    }
}

fn property_value(tag: &str, property: PropertyId) -> String {
    format!("{tag}-property:{}", property.get())
}
