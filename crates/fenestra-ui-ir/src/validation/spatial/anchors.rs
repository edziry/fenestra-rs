use crate::error::{IrValidationError, IrValidationErrorKind};
use crate::spatial::{SpatialAnchorTargetRecipeV2, SpatialPlacementRecipeV2, SpatialProgramV2};

use super::context::{SpatialContext, signature_prefix};
use super::failure;

pub(super) fn validate_anchors(
    program: &SpatialProgramV2,
    context: &SpatialContext,
) -> Result<(), IrValidationError> {
    for node in program.nodes() {
        let SpatialPlacementRecipeV2::Free(free) = node.placement() else {
            continue;
        };
        let SpatialAnchorTargetRecipeV2::Node(target) = free.target() else {
            continue;
        };
        let Some(_) = context.node_indexes.get(target.value()) else {
            return Err(failure(
                IrValidationErrorKind::MissingSpatialAnchorTarget,
                target.span(),
            ));
        };
        if target.value() == node.symbol().value() {
            return Err(failure(
                IrValidationErrorKind::SelfAnchorTarget,
                target.span(),
            ));
        }
        let source_signature = &context.signatures[node.symbol().value()];
        let target_signature = &context.signatures[target.value()];
        if !signature_prefix(target_signature, source_signature) {
            return Err(failure(
                IrValidationErrorKind::SpatialAnchorContextMismatch,
                target.span(),
            ));
        }
    }
    Ok(())
}
