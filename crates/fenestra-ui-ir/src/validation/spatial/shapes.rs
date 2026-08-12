use std::collections::HashSet;

use crate::error::{IrValidationError, IrValidationErrorKind};
use crate::spatial::{SpatialPathVerbRecipeV2, SpatialProgramV2, SpatialShapeGeometryV2};
use crate::validated::ValidatedStyleProgram;

use super::bindings;
use super::context::SpatialContext;
use super::failure;

pub(super) fn validate_shapes(
    style: &ValidatedStyleProgram,
    program: &SpatialProgramV2,
    _context: &SpatialContext,
) -> Result<(), IrValidationError> {
    for node in program.nodes() {
        let target = *node.template().value();
        let mut symbols = HashSet::new();
        for shape in node.shapes() {
            bindings::span(shape.span())?;
            bindings::span(shape.symbol().span())?;
            if !symbols.insert(*shape.symbol().value()) {
                return Err(failure(
                    IrValidationErrorKind::DuplicateSpatialShape,
                    shape.symbol().span(),
                ));
            }
            match shape.geometry() {
                SpatialShapeGeometryV2::Rect {
                    origin,
                    width,
                    height,
                } => {
                    bindings::point(style, target, *origin)?;
                    bindings::fixed(style, target, *width)?;
                    bindings::fixed(style, target, *height)?;
                }
                SpatialShapeGeometryV2::Circle { center, radius } => {
                    bindings::point(style, target, *center)?;
                    bindings::fixed(style, target, *radius)?;
                }
                SpatialShapeGeometryV2::Polygon { points } => {
                    for point in points {
                        bindings::span(point.span())?;
                        bindings::point(style, target, point.point())?;
                    }
                }
                SpatialShapeGeometryV2::Path { verbs } => {
                    for verb in verbs {
                        bindings::span(verb.span())?;
                        match verb {
                            SpatialPathVerbRecipeV2::MoveTo { to, .. }
                            | SpatialPathVerbRecipeV2::LineTo { to, .. } => {
                                bindings::point(style, target, *to)?;
                            }
                            SpatialPathVerbRecipeV2::QuadraticTo { control, to, .. } => {
                                bindings::point(style, target, *control)?;
                                bindings::point(style, target, *to)?;
                            }
                            SpatialPathVerbRecipeV2::CubicTo {
                                control1,
                                control2,
                                to,
                                ..
                            } => {
                                bindings::point(style, target, *control1)?;
                                bindings::point(style, target, *control2)?;
                                bindings::point(style, target, *to)?;
                            }
                            SpatialPathVerbRecipeV2::Close { .. } => {}
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
