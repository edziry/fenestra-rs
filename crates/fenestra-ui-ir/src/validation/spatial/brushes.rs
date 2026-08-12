use std::collections::HashSet;

use crate::error::{IrValidationError, IrValidationErrorKind};
use crate::spatial::{SpatialBrushContentV2, SpatialProgramV2};
use crate::validated::ValidatedStyleProgram;

use super::bindings;
use super::context::SpatialContext;
use super::failure;

pub(super) fn validate_brushes(
    style: &ValidatedStyleProgram,
    program: &SpatialProgramV2,
    _context: &SpatialContext,
) -> Result<(), IrValidationError> {
    for node in program.nodes() {
        let target = *node.template().value();
        let mut symbols = HashSet::new();
        for brush in node.brushes() {
            bindings::span(brush.span())?;
            bindings::span(brush.symbol().span())?;
            if !symbols.insert(*brush.symbol().value()) {
                return Err(failure(
                    IrValidationErrorKind::DuplicateSpatialBrush,
                    brush.symbol().span(),
                ));
            }
            match brush.content() {
                SpatialBrushContentV2::Solid { color } => {
                    bindings::color(style, target, *color)?;
                }
                SpatialBrushContentV2::LinearGradient { start, end, stops } => {
                    bindings::point(style, target, *start)?;
                    bindings::point(style, target, *end)?;
                    for stop in stops {
                        bindings::span(stop.span())?;
                        bindings::span(stop.offset().span())?;
                        bindings::color(style, target, stop.color())?;
                    }
                }
            }
        }
    }
    Ok(())
}
