use fenestra_ui_ir::prototype::{InputPolicy, InvalidationClass, InvalidationSet, PropertyValue};
use fenestra_ui_runtime::prototype::{
    HeadlessRect, HeadlessSemanticAction, HeadlessSemanticRole, HeadlessSurface,
};

pub(super) fn property_value(value: &PropertyValue) -> String {
    match value {
        PropertyValue::Bool(value) => format!("bool:{value}"),
        PropertyValue::ScalarI32(value) => format!("scalar-i32:{value}"),
        PropertyValue::Rgba8(value) => color(*value),
        PropertyValue::InputPolicy(value) => format!("input-policy:{}", input(*value)),
    }
}

pub(super) fn invalidation(value: InvalidationSet) -> String {
    let mut output = String::new();
    for class in value.iter() {
        if !output.is_empty() {
            output.push(',');
        }
        output.push_str(invalidation_class(class));
    }
    output
}

pub(super) fn color(value: [u8; 4]) -> String {
    format!("rgba8:{},{},{},{}", value[0], value[1], value[2], value[3])
}

pub(super) fn input(value: InputPolicy) -> &'static str {
    match value {
        InputPolicy::Accept => "accept",
        InputPolicy::Ignore => "ignore",
    }
}

pub(super) fn rect(value: HeadlessRect) -> String {
    format!(
        "{},{},{},{}",
        value.x(),
        value.y(),
        value.width(),
        value.height()
    )
}

pub(super) fn surface(value: HeadlessSurface) -> String {
    format!("width={}|height={}", value.width(), value.height())
}

pub(super) fn semantic_role(value: HeadlessSemanticRole) -> &'static str {
    match value {
        HeadlessSemanticRole::Control => "control",
    }
}

pub(super) fn semantic_action(value: HeadlessSemanticAction) -> &'static str {
    match value {
        HeadlessSemanticAction::Activate => "activate",
    }
}

fn invalidation_class(value: InvalidationClass) -> &'static str {
    match value {
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
