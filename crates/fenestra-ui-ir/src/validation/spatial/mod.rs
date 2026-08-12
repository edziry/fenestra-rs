mod anchors;
mod bindings;
mod brushes;
mod clips;
mod context;
mod counts;
mod images;
mod items;
mod layout;
mod nodes;
mod shapes;

use crate::error::{IrValidationError, IrValidationErrorKind};
use crate::ids::SUPPORTED_SPATIAL_FORMAT;
use crate::limits::SpatialValidationLimitsV2;
use crate::spatial::SpatialProgramV2;
use crate::validated::{ValidatedSpatialProgramV2, ValidatedStyleProgram};

use super::failure;

/// Validates and links one symbolic spatial program to one exact style domain.
#[must_use = "spatial IR validation errors must be handled"]
pub fn validate_spatial(
    style: &ValidatedStyleProgram,
    program: SpatialProgramV2,
    limits: SpatialValidationLimitsV2,
) -> Result<ValidatedSpatialProgramV2, IrValidationError> {
    if !program.span().is_valid() {
        return Err(failure(
            IrValidationErrorKind::InvalidSourceSpan,
            program.span(),
        ));
    }
    if program.format() != SUPPORTED_SPATIAL_FORMAT {
        return Err(failure(
            IrValidationErrorKind::UnsupportedSpatialFormat,
            program.span(),
        ));
    }
    let manifest = &style.construction().schema().data.manifest;
    if program.schema_namespace() != manifest.namespace
        || program.schema_revision() != manifest.revision
    {
        return Err(failure(
            IrValidationErrorKind::SchemaIdentityMismatch,
            program.span(),
        ));
    }
    counts::preflight_spatial_counts(&program, limits)?;
    let context = context::build_context(style, &program);
    nodes::validate_nodes(style, &program, &context)?;
    layout::validate_layout(style, &program, &context)?;
    shapes::validate_shapes(style, &program, &context)?;
    brushes::validate_brushes(style, &program, &context)?;
    images::validate_images(&program)?;
    clips::validate_clips(&program, &context)?;
    items::validate_items(style, &program, &context)?;
    anchors::validate_anchors(&program, &context)?;
    Ok(ValidatedSpatialProgramV2::new(
        style.clone(),
        program,
        context.node_indexes,
        context.template_indexes,
        context.signatures,
    ))
}
