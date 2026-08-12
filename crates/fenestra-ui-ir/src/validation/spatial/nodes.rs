use std::collections::HashSet;

use crate::error::{IrValidationError, IrValidationErrorKind};
use crate::spatial::{SpatialNodeParentV2, SpatialProgramV2};
use crate::validated::ValidatedStyleProgram;

use super::context::{SpatialContext, signature_prefix};
use super::{bindings, failure};

pub(super) fn validate_nodes(
    style: &ValidatedStyleProgram,
    program: &SpatialProgramV2,
    context: &SpatialContext,
) -> Result<(), IrValidationError> {
    let mut symbols = HashSet::new();
    let mut templates = HashSet::new();
    let mut active = Vec::new();
    for (index, node) in program.nodes().iter().enumerate() {
        bindings::span(node.span())?;
        bindings::span(node.symbol().span())?;
        if !symbols.insert(*node.symbol().value()) {
            return Err(failure(
                IrValidationErrorKind::DuplicateSpatialNode,
                node.symbol().span(),
            ));
        }
        bindings::span(node.template().span())?;
        if !templates.insert(*node.template().value()) {
            return Err(failure(
                IrValidationErrorKind::DuplicateSpatialTemplate,
                node.template().span(),
            ));
        }
        if style
            .construction()
            .template(*node.template().value())
            .is_none()
        {
            return Err(failure(
                IrValidationErrorKind::MissingSpatialTemplate,
                node.template().span(),
            ));
        }
        let SpatialNodeParentV2::Node(parent) = node.parent() else {
            active.clear();
            active.push(*node.symbol().value());
            continue;
        };
        bindings::span(parent.span())?;
        let Some(parent_index) = context.node_indexes.get(parent.value()).copied() else {
            return Err(failure(
                IrValidationErrorKind::MissingSpatialParent,
                parent.span(),
            ));
        };
        let parent_signature = &context.signatures[parent.value()];
        let source_signature = &context.signatures[node.symbol().value()];
        if !signature_prefix(parent_signature, source_signature) {
            return Err(failure(
                IrValidationErrorKind::SpatialParentContextMismatch,
                parent.span(),
            ));
        }
        if parent_index >= index {
            return Err(failure(
                IrValidationErrorKind::SpatialParentNotEarlier,
                parent.span(),
            ));
        }
        while active.last().is_some_and(|symbol| symbol != parent.value()) {
            active.pop();
        }
        if active.last() != Some(parent.value()) {
            return Err(failure(
                IrValidationErrorKind::InvalidSpatialPreorder,
                parent.span(),
            ));
        }
        active.push(*node.symbol().value());
    }
    Ok(())
}
