use crate::error::{IrValidationError, IrValidationErrorKind};
use crate::spatial::{
    SpatialCoverageRecipeV2, SpatialNodeDeclarationV2, SpatialPaintRecipeV2, SpatialProgramV2,
};
use crate::validated::ValidatedStyleProgram;

use super::bindings;
use super::clips::validate_address;
use super::context::SpatialContext;
use super::failure;

pub(super) fn validate_items(
    style: &ValidatedStyleProgram,
    program: &SpatialProgramV2,
    context: &SpatialContext,
) -> Result<(), IrValidationError> {
    for (index, node) in program.nodes().iter().enumerate() {
        for paint in node.paint_items() {
            validate_paint(style, program, context, node, index, paint)?;
        }
    }
    for (index, node) in program.nodes().iter().enumerate() {
        for hit in node.hit_items() {
            bindings::span(hit.span())?;
            coverage(style, context, node, index, hit.coverage())?;
            if let Some(clip) = hit.clip() {
                validate_address(program, context, node, index, clip, None)?;
            }
            bindings::input(style, *node.template().value(), hit.input_policy())?;
        }
    }
    for (index, node) in program.nodes().iter().enumerate() {
        for semantic in node.semantic_items() {
            bindings::span(semantic.span())?;
            bindings::span(semantic.shape().span())?;
            if !context.shapes[index].contains(semantic.shape().value()) {
                return Err(failure(
                    IrValidationErrorKind::MissingSpatialShape,
                    semantic.shape().span(),
                ));
            }
            if let Some(clip) = semantic.clip() {
                validate_address(program, context, node, index, clip, None)?;
            }
        }
    }
    Ok(())
}

fn validate_paint(
    style: &ValidatedStyleProgram,
    program: &SpatialProgramV2,
    context: &SpatialContext,
    node: &SpatialNodeDeclarationV2,
    index: usize,
    paint: &SpatialPaintRecipeV2,
) -> Result<(), IrValidationError> {
    bindings::span(paint.span())?;
    match paint {
        SpatialPaintRecipeV2::CoveragePaint {
            coverage: recipe,
            brush,
            opacity,
            clip,
            ..
        } => {
            coverage(style, context, node, index, *recipe)?;
            bindings::span(brush.span())?;
            if !context.brushes[index].contains(brush.value()) {
                return Err(failure(
                    IrValidationErrorKind::MissingSpatialBrush,
                    brush.span(),
                ));
            }
            bindings::span(opacity.span())?;
            if let Some(clip) = clip {
                validate_address(program, context, node, index, *clip, None)?;
            }
        }
        SpatialPaintRecipeV2::ImagePaint {
            image,
            source_x,
            source_y,
            source_width,
            source_height,
            destination_origin,
            destination_width,
            destination_height,
            opacity,
            clip,
            ..
        } => {
            bindings::span(image.span())?;
            if !context.images.contains(image.value()) {
                return Err(failure(
                    IrValidationErrorKind::MissingSpatialImage,
                    image.span(),
                ));
            }
            for field in [source_x, source_y, source_width, source_height] {
                bindings::span(field.span())?;
            }
            let target = *node.template().value();
            bindings::point(style, target, *destination_origin)?;
            bindings::fixed(style, target, *destination_width)?;
            bindings::fixed(style, target, *destination_height)?;
            bindings::span(opacity.span())?;
            if let Some(clip) = clip {
                validate_address(program, context, node, index, *clip, None)?;
            }
        }
    }
    Ok(())
}

fn coverage(
    style: &ValidatedStyleProgram,
    context: &SpatialContext,
    node: &SpatialNodeDeclarationV2,
    index: usize,
    coverage: SpatialCoverageRecipeV2,
) -> Result<(), IrValidationError> {
    let (shape, width) = match coverage {
        SpatialCoverageRecipeV2::Fill { shape, .. } => (shape, None),
        SpatialCoverageRecipeV2::RoundStroke { shape, width } => (shape, Some(width)),
    };
    bindings::span(shape.span())?;
    if !context.shapes[index].contains(shape.value()) {
        return Err(failure(
            IrValidationErrorKind::MissingSpatialShape,
            shape.span(),
        ));
    }
    if let Some(width) = width {
        bindings::fixed(style, *node.template().value(), width)?;
    }
    Ok(())
}
