use std::collections::HashSet;

use crate::error::{IrValidationError, IrValidationErrorKind};
use crate::spatial::{SpatialClipAddressV2, SpatialNodeDeclarationV2, SpatialProgramV2};

use super::bindings;
use super::context::{SpatialContext, is_ancestor};
use super::failure;

pub(super) fn validate_clips(
    program: &SpatialProgramV2,
    context: &SpatialContext,
) -> Result<(), IrValidationError> {
    for (node_index, node) in program.nodes().iter().enumerate() {
        let mut symbols = HashSet::new();
        for (clip_index, clip) in node.clips().iter().enumerate() {
            bindings::span(clip.span())?;
            bindings::span(clip.symbol().span())?;
            if !symbols.insert(*clip.symbol().value()) {
                return Err(failure(
                    IrValidationErrorKind::DuplicateSpatialClip,
                    clip.symbol().span(),
                ));
            }
            if let Some(parent) = clip.parent() {
                validate_address(program, context, node, node_index, parent, Some(clip_index))?;
            }
            bindings::span(clip.shape().span())?;
            if !context.shapes[node_index].contains(clip.shape().value()) {
                return Err(failure(
                    IrValidationErrorKind::MissingSpatialShape,
                    clip.shape().span(),
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn validate_address(
    program: &SpatialProgramV2,
    context: &SpatialContext,
    node: &SpatialNodeDeclarationV2,
    node_index: usize,
    address: SpatialClipAddressV2,
    local_before: Option<usize>,
) -> Result<(), IrValidationError> {
    bindings::span(address.owner().span())?;
    let Some(owner_index) = context.node_indexes.get(address.owner().value()).copied() else {
        return Err(failure(
            IrValidationErrorKind::MissingSpatialClipOwner,
            address.owner().span(),
        ));
    };
    bindings::span(address.clip().span())?;
    let Some(clip_index) = context.clips[owner_index]
        .get(address.clip().value())
        .copied()
    else {
        return Err(failure(
            IrValidationErrorKind::MissingSpatialClip,
            address.clip().span(),
        ));
    };
    let owner_symbol = *node.symbol().value();
    if !is_ancestor(program, context, owner_symbol, *address.owner().value()) {
        return Err(failure(
            IrValidationErrorKind::SpatialClipOwnerNotAncestor,
            address.owner().span(),
        ));
    }
    if owner_index == node_index && local_before.is_some_and(|current| clip_index >= current) {
        return Err(failure(
            IrValidationErrorKind::SpatialClipParentNotEarlier,
            address.clip().span(),
        ));
    }
    Ok(())
}
