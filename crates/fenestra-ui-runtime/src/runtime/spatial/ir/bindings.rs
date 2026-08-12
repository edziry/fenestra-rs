use fenestra_ui_ir::prototype::{
    InputPolicy, PropertyValue, SpatialBindingV2, SpatialFieldV2, SpatialPointRecipeV2,
};
use fenestra_ui_spatial::prototype::{
    SpatialInputPolicyV2, SpatialPointV2, SpatialRgba8V2, SpatialScalarV2,
};

use super::super::error::{RuntimeSpatialIrErrorKindV2, RuntimeSpatialIrErrorV2};
use super::super::view::RuntimeSpatialBuildViewV2;
use crate::logical_tree::NodeId;

pub(super) fn i32_value(
    field: SpatialFieldV2<SpatialBindingV2<i32>>,
    owner: NodeId,
    view: RuntimeSpatialBuildViewV2<'_>,
) -> Result<i32, RuntimeSpatialIrErrorV2> {
    match *field.value() {
        SpatialBindingV2::Literal(value) => Ok(value),
        SpatialBindingV2::Property(property) => match view.property(owner, property) {
            Some(PropertyValue::ScalarI32(value)) => Ok(*value),
            _ => Err(invariant(field.span())),
        },
    }
}

pub(super) fn scalar(
    field: SpatialFieldV2<SpatialBindingV2<i64>>,
    owner: NodeId,
    view: RuntimeSpatialBuildViewV2<'_>,
) -> Result<SpatialScalarV2, RuntimeSpatialIrErrorV2> {
    let raw = match *field.value() {
        SpatialBindingV2::Literal(value) => value,
        SpatialBindingV2::Property(property) => match view.property(owner, property) {
            Some(PropertyValue::ScalarI32(value)) => i64::from(*value) * SpatialScalarV2::SCALE,
            _ => return Err(invariant(field.span())),
        },
    };
    Ok(SpatialScalarV2::new(raw))
}

pub(super) fn point(
    recipe: SpatialPointRecipeV2,
    owner: NodeId,
    view: RuntimeSpatialBuildViewV2<'_>,
) -> Result<SpatialPointV2, RuntimeSpatialIrErrorV2> {
    Ok(SpatialPointV2::new(
        scalar(recipe.x(), owner, view)?,
        scalar(recipe.y(), owner, view)?,
    ))
}

pub(super) fn color(
    field: SpatialFieldV2<SpatialBindingV2<[u8; 4]>>,
    owner: NodeId,
    view: RuntimeSpatialBuildViewV2<'_>,
) -> Result<SpatialRgba8V2, RuntimeSpatialIrErrorV2> {
    let [r, g, b, a] = match *field.value() {
        SpatialBindingV2::Literal(value) => value,
        SpatialBindingV2::Property(property) => match view.property(owner, property) {
            Some(PropertyValue::Rgba8(value)) => *value,
            _ => return Err(invariant(field.span())),
        },
    };
    Ok(SpatialRgba8V2::new(r, g, b, a))
}

pub(super) fn input_policy(
    field: SpatialFieldV2<SpatialBindingV2<InputPolicy>>,
    owner: NodeId,
    view: RuntimeSpatialBuildViewV2<'_>,
) -> Result<SpatialInputPolicyV2, RuntimeSpatialIrErrorV2> {
    let value = match *field.value() {
        SpatialBindingV2::Literal(value) => value,
        SpatialBindingV2::Property(property) => match view.property(owner, property) {
            Some(PropertyValue::InputPolicy(value)) => *value,
            _ => return Err(invariant(field.span())),
        },
    };
    Ok(match value {
        InputPolicy::Accept => SpatialInputPolicyV2::Accept,
        InputPolicy::Ignore => SpatialInputPolicyV2::Ignore,
    })
}

fn invariant(span: fenestra_ui_ir::prototype::SourceSpan) -> RuntimeSpatialIrErrorV2 {
    RuntimeSpatialIrErrorV2::new(RuntimeSpatialIrErrorKindV2::InvariantViolation, span)
}
