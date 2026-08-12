use crate::error::IrValidationError;
use crate::spatial::{
    SpatialAnchorTargetRecipeV2, SpatialDimensionRecipeV2, SpatialPlacementRecipeV2,
    SpatialProgramV2, SpatialTransformRecipeV2,
};
use crate::validated::ValidatedStyleProgram;

use super::bindings;
use super::context::SpatialContext;

pub(super) fn validate_layout(
    style: &ValidatedStyleProgram,
    program: &SpatialProgramV2,
    _context: &SpatialContext,
) -> Result<(), IrValidationError> {
    let viewport = program.viewport_container();
    bindings::span(viewport.span())?;
    for field in [
        viewport.left(),
        viewport.right(),
        viewport.top(),
        viewport.bottom(),
        viewport.gap(),
    ] {
        bindings::span(field.span())?;
    }
    for node in program.nodes() {
        let target = *node.template().value();
        match node.placement() {
            SpatialPlacementRecipeV2::Layout(layout) => {
                dimension(style, target, layout.width())?;
                dimension(style, target, layout.height())?;
                transform(style, target, layout.transform())?;
            }
            SpatialPlacementRecipeV2::Free(free) => {
                bindings::integer(style, target, free.width())?;
                bindings::integer(style, target, free.height())?;
                if let SpatialAnchorTargetRecipeV2::Node(field) = free.target() {
                    bindings::span(field.span())?;
                }
                bindings::point(style, target, free.offset())?;
                transform(style, target, free.transform())?;
            }
        }
        let container = node.container();
        let padding = container.padding();
        bindings::integer(style, target, padding.left())?;
        bindings::integer(style, target, padding.right())?;
        bindings::integer(style, target, padding.top())?;
        bindings::integer(style, target, padding.bottom())?;
        bindings::integer(style, target, container.gap())?;
    }
    Ok(())
}

fn dimension(
    style: &ValidatedStyleProgram,
    target: crate::ids::TemplateNodeId,
    recipe: SpatialDimensionRecipeV2,
) -> Result<(), IrValidationError> {
    bindings::integer(style, target, recipe.minimum())?;
    bindings::integer(style, target, recipe.preferred())?;
    bindings::integer(style, target, recipe.maximum())
}

fn transform(
    style: &ValidatedStyleProgram,
    target: crate::ids::TemplateNodeId,
    recipe: SpatialTransformRecipeV2,
) -> Result<(), IrValidationError> {
    bindings::fixed(style, target, recipe.a())?;
    bindings::fixed(style, target, recipe.b())?;
    bindings::fixed(style, target, recipe.c())?;
    bindings::fixed(style, target, recipe.d())?;
    bindings::fixed(style, target, recipe.tx())?;
    bindings::fixed(style, target, recipe.ty())?;
    bindings::point(style, target, recipe.origin())
}
