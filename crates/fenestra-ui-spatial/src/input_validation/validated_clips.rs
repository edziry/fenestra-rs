//! Dense clip keys and record-major clip validation.

use super::make_resolve_error;
use super::validated_images::ValidatedImagesProof;
use super::validated_shapes::ShapeLocalBoundsInput;
use crate::aggregate_input::SpatialInputV2;
use crate::content_diagnostic::{
    SpatialClipErrorV2, SpatialContentReferenceV2, SpatialKeyedContentTableV2,
};
use crate::content_error::SpatialContentErrorKindV2;
use crate::coverage::SpatialFillRuleV2;
use crate::error::SpatialErrorLocationV2;
use crate::item_field::SpatialClipFieldV2;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::paint_kernel::ValidatedImageP4;
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};
use crate::topology::SpatialNodeV2;

struct ValidatedClipPlan {
    owner: u32,
    parent: Option<u32>,
    shape: u32,
    fill_rule: SpatialFillRuleV2,
    depth: usize,
}

pub(super) struct ValidatedClipsProof<'a> {
    images: ValidatedImagesProof<'a>,
    clips: Vec<ValidatedClipPlan>,
}

impl<'a> ValidatedClipsProof<'a> {
    pub(super) fn input(&self) -> SpatialInputV2<'a> {
        self.images.input()
    }

    pub(super) fn limits(&self) -> SpatialLimitsV2 {
        self.images.limits()
    }

    pub(super) fn dependency_islands(
        &self,
    ) -> impl Iterator<Item = super::islands::preflight::DependencyIslandInput<'_>> + '_ {
        self.images.dependency_islands()
    }

    pub(super) fn validated_paths(&self) -> &[crate::geometry_kernel::ValidatedPathK1<'a>] {
        self.images.validated_paths()
    }

    pub(super) fn shape_local_bounds_inputs(
        &self,
    ) -> impl Iterator<Item = ShapeLocalBoundsInput<'a>> + '_ {
        self.images.shape_local_bounds_inputs()
    }

    pub(super) fn validated_image(&self, index: u32) -> Option<ValidatedImageP4<'a>> {
        self.images.validated_image(index)
    }

    pub(super) fn clip_owner_is_same_or_ancestor_of(&self, clip: u32, owner: u32) -> Option<bool> {
        let clip = self.clips.get(clip as usize)?;
        Some(is_same_or_ancestor(
            self.input().topology().nodes(),
            clip.owner,
            owner,
        ))
    }
}

pub(super) fn prepare_validated_clips<'a>(
    images: ValidatedImagesProof<'a>,
) -> Result<ValidatedClipsProof<'a>, SpatialResolveErrorV2> {
    let input = images.input();
    let nodes = input.topology().nodes();
    let geometry = input.geometry();
    let shapes = geometry.shapes();
    let clips = geometry.clips();

    for (index, clip) in clips.iter().copied().enumerate() {
        let ordinal = trusted_clip_ordinal(index);
        if clip.key().get() != ordinal {
            return Err(content_error(
                SpatialContentErrorKindV2::NonDenseKey(SpatialKeyedContentTableV2::Clip),
                clip_location(ordinal, SpatialClipFieldV2::Key),
            ));
        }
    }

    let node_count = nodes.len() as u128;
    let shape_count = shapes.len() as u128;
    let clip_count = clips.len() as u128;
    let limits = images.limits();
    let mut validated: Vec<ValidatedClipPlan> = Vec::with_capacity(clips.len());

    for (index, clip) in clips.iter().copied().enumerate() {
        let ordinal = trusted_clip_ordinal(index);
        let owner = clip.owner().get();
        if owner == 0 || u128::from(owner) >= node_count {
            return Err(invalid_reference(
                SpatialContentReferenceV2::Owner,
                ordinal,
                SpatialClipFieldV2::Owner,
            ));
        }

        let parent = clip.parent().map(|key| key.get());
        let parent_plan = if let Some(parent) = parent {
            if u128::from(parent) >= clip_count {
                return Err(invalid_reference(
                    SpatialContentReferenceV2::Clip,
                    ordinal,
                    SpatialClipFieldV2::Parent,
                ));
            }
            if parent >= ordinal {
                return Err(invalid_clip(
                    SpatialClipErrorV2::ForwardParent,
                    ordinal,
                    SpatialClipFieldV2::Parent,
                ));
            }
            Some(
                validated
                    .get(parent as usize)
                    .expect("an earlier clip has a validated plan"),
            )
        } else {
            None
        };

        let shape = clip.shape().get();
        if u128::from(shape) >= shape_count {
            return Err(invalid_reference(
                SpatialContentReferenceV2::Shape,
                ordinal,
                SpatialClipFieldV2::Shape,
            ));
        }
        if shapes[shape as usize].owner().get() != owner {
            return Err(invalid_clip(
                SpatialClipErrorV2::ShapeOwnerMismatch,
                ordinal,
                SpatialClipFieldV2::Shape,
            ));
        }

        if let Some(parent_plan) = parent_plan
            && !is_same_or_ancestor(nodes, parent_plan.owner, owner)
        {
            return Err(invalid_clip(
                SpatialClipErrorV2::OwnerNotAncestor,
                ordinal,
                SpatialClipFieldV2::Parent,
            ));
        }

        let depth = parent_plan.map_or(1, |parent| {
            parent
                .depth
                .checked_add(1)
                .expect("a validated clip chain fits the clip table")
        });
        validate_clip_depth(ordinal, depth, limits)?;
        validated.push(ValidatedClipPlan {
            owner,
            parent,
            shape,
            fill_rule: clip.fill_rule(),
            depth,
        });
    }

    Ok(ValidatedClipsProof {
        images,
        clips: validated,
    })
}

pub(super) fn validate_clip_depth(
    clip: u32,
    observed: usize,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    let observed = observed as u128;
    let maximum = limits.limit(SpatialLimitKindV2::ClipDepth) as u128;
    if observed > maximum {
        return Err(SpatialResolveErrorV2::limit_exceeded(
            SpatialLimitKindV2::ClipDepth,
            clip_location(clip, SpatialClipFieldV2::Parent),
            observed,
            maximum,
        ));
    }
    Ok(())
}

fn is_same_or_ancestor(nodes: &[SpatialNodeV2], ancestor: u32, mut node: u32) -> bool {
    loop {
        if node == ancestor {
            return true;
        }
        let Some(parent) = nodes[trusted_node_index(node)].parent() else {
            return false;
        };
        node = parent.get();
    }
}

fn trusted_clip_ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("phase one validated the clip row capacity")
}

fn trusted_node_index(index: u32) -> usize {
    usize::try_from(index).expect("phase two validated the spatial node ordinal")
}

const fn clip_location(index: u32, field: SpatialClipFieldV2) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::Clip { index, field }
}

fn invalid_reference(
    reference: SpatialContentReferenceV2,
    clip: u32,
    field: SpatialClipFieldV2,
) -> SpatialResolveErrorV2 {
    content_error(
        SpatialContentErrorKindV2::InvalidReference(reference),
        clip_location(clip, field),
    )
}

fn invalid_clip(
    kind: SpatialClipErrorV2,
    clip: u32,
    field: SpatialClipFieldV2,
) -> SpatialResolveErrorV2 {
    content_error(
        SpatialContentErrorKindV2::InvalidClip(kind),
        clip_location(clip, field),
    )
}

fn content_error(
    kind: SpatialContentErrorKindV2,
    location: SpatialErrorLocationV2,
) -> SpatialResolveErrorV2 {
    make_resolve_error(SpatialResolveErrorKindV2::Content(kind), location)
}

#[cfg(test)]
impl ValidatedClipsProof<'_> {
    pub(super) fn validated_clip_facts(
        &self,
    ) -> Vec<(u32, u32, Option<u32>, u32, SpatialFillRuleV2, usize)> {
        self.clips
            .iter()
            .enumerate()
            .map(|(index, clip)| {
                (
                    trusted_clip_ordinal(index),
                    clip.owner,
                    clip.parent,
                    clip.shape,
                    clip.fill_rule,
                    clip.depth,
                )
            })
            .collect()
    }

    pub(super) fn validated_image_facts(&self) -> Vec<(u32, u32, u32, u32, Vec<u8>)> {
        self.images.validated_image_facts()
    }

    pub(super) fn accepted_pixel_total(&self) -> u128 {
        self.images.accepted_pixel_total()
    }

    pub(super) fn prepared_brush_facts(
        &self,
    ) -> Vec<(u32, crate::brush::SpatialBrushKindV2, usize)> {
        self.images.prepared_brush_facts()
    }

    pub(super) fn gradient_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.images.gradient_range_facts()
    }

    pub(super) fn validated_shape_facts(
        &self,
    ) -> Vec<(u32, crate::shape::SpatialShapeKindV2, usize)> {
        self.images.validated_shape_facts()
    }

    pub(super) fn polygon_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.images.polygon_range_facts()
    }

    pub(super) fn validated_path_facts(&self) -> Vec<(u32, usize, usize)> {
        self.images.validated_path_facts()
    }

    pub(super) fn subpath_total(&self) -> usize {
        self.images.subpath_total()
    }

    pub(super) fn path_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.images.path_range_facts()
    }

    pub(super) fn prepared_island_facts(&self) -> Vec<(u32, Vec<u32>)> {
        self.images.prepared_island_facts()
    }
}
