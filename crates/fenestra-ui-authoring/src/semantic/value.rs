use fenestra_ui_ir::prototype::{
    InputPolicy, InvalidationClass, InvalidationSet, PropertyValue, ValueType,
};

pub(super) fn value_type_name(value_type: ValueType) -> &'static str {
    match value_type {
        ValueType::Bool => "bool",
        ValueType::ScalarI32 => "scalar-i32",
        ValueType::Rgba8 => "rgba8",
        ValueType::InputPolicy => "input-policy",
    }
}

pub(super) fn value_name(value: &PropertyValue) -> String {
    match value {
        PropertyValue::Bool(value) => format!("bool:{value}"),
        PropertyValue::ScalarI32(value) => format!("scalar-i32:{value}"),
        PropertyValue::Rgba8([red, green, blue, alpha]) => {
            format!("rgba8:{red},{green},{blue},{alpha}")
        }
        PropertyValue::InputPolicy(InputPolicy::Accept) => "input-policy:accept".to_owned(),
        PropertyValue::InputPolicy(InputPolicy::Ignore) => "input-policy:ignore".to_owned(),
    }
}

pub(super) fn invalidation_name(invalidation: InvalidationSet) -> String {
    let mut output = String::new();
    for class in invalidation.iter() {
        if !output.is_empty() {
            output.push(',');
        }
        output.push_str(invalidation_class_name(class));
    }
    output
}

fn invalidation_class_name(class: InvalidationClass) -> &'static str {
    match class {
        InvalidationClass::Structure => "structure",
        InvalidationClass::StyleMatch => "style-match",
        InvalidationClass::Intrinsic => "intrinsic",
        InvalidationClass::Layout => "layout",
        InvalidationClass::Semantics => "semantics",
        InvalidationClass::HitTest => "hit-test",
        InvalidationClass::Paint => "paint",
        InvalidationClass::Composition => "composition",
        InvalidationClass::Surface => "surface",
    }
}
