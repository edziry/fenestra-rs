use crate::error::{IrValidationError, IrValidationErrorKind};
use crate::source::SourceSpan;
use crate::spatial::{SpatialBindingV2, SpatialFieldV2, SpatialPointRecipeV2};
use crate::validated::ValidatedStyleProgram;
use crate::value::ValueType;

use super::failure;

const MIN_FIXED: i64 = -140_737_488_289_792;
const MAX_FIXED: i64 = 140_737_488_289_792;

pub(super) fn span(source: SourceSpan) -> Result<(), IrValidationError> {
    if source.is_valid() {
        Ok(())
    } else {
        Err(failure(IrValidationErrorKind::InvalidSourceSpan, source))
    }
}

fn property(
    style: &ValidatedStyleProgram,
    target: crate::ids::TemplateNodeId,
    property: crate::ids::PropertyId,
    expected: ValueType,
    source: SourceSpan,
) -> Result<(), IrValidationError> {
    let template = style
        .construction()
        .template(target)
        .expect("phase three resolves spatial templates");
    let Some(property) = template.component().property(property) else {
        return Err(failure(
            IrValidationErrorKind::UnknownSpatialProperty,
            source,
        ));
    };
    if property.value_type() != expected {
        return Err(failure(
            IrValidationErrorKind::SpatialPropertyTypeMismatch,
            source,
        ));
    }
    Ok(())
}

pub(super) fn integer(
    style: &ValidatedStyleProgram,
    target: crate::ids::TemplateNodeId,
    field: SpatialFieldV2<SpatialBindingV2<i32>>,
) -> Result<(), IrValidationError> {
    span(field.span())?;
    if let SpatialBindingV2::Property(id) = field.value() {
        property(style, target, *id, ValueType::ScalarI32, field.span())?;
    }
    Ok(())
}

pub(super) fn fixed(
    style: &ValidatedStyleProgram,
    target: crate::ids::TemplateNodeId,
    field: SpatialFieldV2<SpatialBindingV2<i64>>,
) -> Result<(), IrValidationError> {
    span(field.span())?;
    match field.value() {
        SpatialBindingV2::Literal(value) if !(MIN_FIXED..=MAX_FIXED).contains(value) => {
            Err(failure(
                IrValidationErrorKind::SpatialFixed16OutOfRange,
                field.span(),
            ))
        }
        SpatialBindingV2::Property(id) => {
            property(style, target, *id, ValueType::ScalarI32, field.span())
        }
        SpatialBindingV2::Literal(_) => Ok(()),
    }
}

pub(super) fn color(
    style: &ValidatedStyleProgram,
    target: crate::ids::TemplateNodeId,
    field: SpatialFieldV2<SpatialBindingV2<[u8; 4]>>,
) -> Result<(), IrValidationError> {
    span(field.span())?;
    if let SpatialBindingV2::Property(id) = field.value() {
        property(style, target, *id, ValueType::Rgba8, field.span())?;
    }
    Ok(())
}

pub(super) fn input(
    style: &ValidatedStyleProgram,
    target: crate::ids::TemplateNodeId,
    field: SpatialFieldV2<SpatialBindingV2<crate::value::InputPolicy>>,
) -> Result<(), IrValidationError> {
    span(field.span())?;
    if let SpatialBindingV2::Property(id) = field.value() {
        property(style, target, *id, ValueType::InputPolicy, field.span())?;
    }
    Ok(())
}

pub(super) fn point(
    style: &ValidatedStyleProgram,
    target: crate::ids::TemplateNodeId,
    point: SpatialPointRecipeV2,
) -> Result<(), IrValidationError> {
    fixed(style, target, point.x())?;
    fixed(style, target, point.y())
}
