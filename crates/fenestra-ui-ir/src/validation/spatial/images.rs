use std::collections::HashSet;

use crate::error::{IrValidationError, IrValidationErrorKind};
use crate::spatial::SpatialProgramV2;

use super::bindings;
use super::failure;

pub(super) fn validate_images(program: &SpatialProgramV2) -> Result<(), IrValidationError> {
    let mut symbols = HashSet::new();
    for image in program.images() {
        bindings::span(image.span())?;
        bindings::span(image.symbol().span())?;
        if !symbols.insert(*image.symbol().value()) {
            return Err(failure(
                IrValidationErrorKind::DuplicateSpatialImage,
                image.symbol().span(),
            ));
        }
        bindings::span(image.width().span())?;
        bindings::span(image.height().span())?;
        bindings::span(image.stride().span())?;
    }
    Ok(())
}
