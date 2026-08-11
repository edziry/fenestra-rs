//! Ordered semantic-item validation.

use super::make_resolve_error;
use super::ordered_items::OrderedItemCursor;
use super::validated_hit_items::ValidatedHitItemsProof;
use crate::aggregate_input::SpatialInputV2;
use crate::content_diagnostic::{
    SpatialClipErrorV2, SpatialContentReferenceV2, SpatialOrderedItemTableV2,
};
use crate::content_error::SpatialContentErrorKindV2;
use crate::coverage::SpatialFillRuleV2;
use crate::error::SpatialErrorLocationV2;
use crate::item_field::SpatialSemanticFieldV2;
use crate::limits::SpatialLimitsV2;
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

#[cfg(test)]
mod facts;

struct ValidatedSemanticItem {
    owner: u32,
    item_ordinal: u32,
    shape: u32,
    fill_rule: SpatialFillRuleV2,
    clip: Option<u32>,
}

pub(super) struct ValidatedSemanticItemsProof<'a> {
    hits: ValidatedHitItemsProof<'a>,
    semantics: Vec<ValidatedSemanticItem>,
}

impl<'a> ValidatedSemanticItemsProof<'a> {
    pub(super) fn input(&self) -> SpatialInputV2<'a> {
        self.hits.input()
    }

    pub(super) fn limits(&self) -> SpatialLimitsV2 {
        self.hits.limits()
    }
}

pub(super) fn prepare_validated_semantic_items<'a>(
    hits: ValidatedHitItemsProof<'a>,
) -> Result<ValidatedSemanticItemsProof<'a>, SpatialResolveErrorV2> {
    let input = hits.input();
    let semantics = input.items().semantic_items();
    let shapes = input.geometry().shapes();
    let node_count = input.topology().nodes().len() as u128;
    let mut cursor = OrderedItemCursor::new();
    let mut validated = Vec::with_capacity(semantics.len());

    for (index, semantic) in semantics.iter().copied().enumerate() {
        let ordinal = trusted_semantic_ordinal(index);
        let owner = semantic.owner().get();
        let candidate = cursor.validate(
            SpatialOrderedItemTableV2::Semantic,
            ordinal,
            owner,
            semantic.item_ordinal(),
            node_count,
        )?;

        let shape = semantic.shape().get();
        if u128::from(shape) >= shapes.len() as u128
            || shapes[shape as usize].owner().get() != owner
        {
            return Err(invalid_reference(
                SpatialContentReferenceV2::Shape,
                ordinal,
                SpatialSemanticFieldV2::Shape,
            ));
        }
        let fill_rule = semantic.fill_rule();
        let clip =
            validate_terminal_clip(&hits, ordinal, owner, semantic.clip().map(|key| key.get()))?;

        validated.push(ValidatedSemanticItem {
            owner: candidate.owner(),
            item_ordinal: candidate.item_ordinal(),
            shape,
            fill_rule,
            clip,
        });
        cursor.commit(candidate);
    }

    Ok(ValidatedSemanticItemsProof {
        hits,
        semantics: validated,
    })
}

fn validate_terminal_clip(
    hits: &ValidatedHitItemsProof<'_>,
    semantic: u32,
    owner: u32,
    clip: Option<u32>,
) -> Result<Option<u32>, SpatialResolveErrorV2> {
    let Some(clip) = clip else {
        return Ok(None);
    };
    match hits.clip_owner_is_same_or_ancestor_of(clip, owner) {
        None => Err(invalid_reference(
            SpatialContentReferenceV2::Clip,
            semantic,
            SpatialSemanticFieldV2::Clip,
        )),
        Some(false) => Err(content_error(
            SpatialContentErrorKindV2::InvalidClip(SpatialClipErrorV2::ItemOwnerNotDescendant),
            semantic_location(semantic, SpatialSemanticFieldV2::Clip),
        )),
        Some(true) => Ok(Some(clip)),
    }
}

fn trusted_semantic_ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("phase one validated the semantic-item row capacity")
}

const fn semantic_location(index: u32, field: SpatialSemanticFieldV2) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::Semantic { index, field }
}

fn invalid_reference(
    reference: SpatialContentReferenceV2,
    semantic: u32,
    field: SpatialSemanticFieldV2,
) -> SpatialResolveErrorV2 {
    content_error(
        SpatialContentErrorKindV2::InvalidReference(reference),
        semantic_location(semantic, field),
    )
}

fn content_error(
    kind: SpatialContentErrorKindV2,
    location: SpatialErrorLocationV2,
) -> SpatialResolveErrorV2 {
    make_resolve_error(SpatialResolveErrorKindV2::Content(kind), location)
}
