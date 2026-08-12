//! Closed world-space AABB projection in output-table order.

use super::make_resolve_error;
use super::world_transforms::WorldTransformProof;
use crate::aabb::SpatialAabbV2;
use crate::aggregate_input::SpatialInputV2;
use crate::error::SpatialErrorLocationV2;
use crate::limits::SpatialLimitsV2;
use crate::model::Affine2V2;
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

#[cfg(test)]
mod facts;

pub(super) struct WorldAabbProof<'a> {
    transforms: WorldTransformProof<'a>,
    geometry: Vec<SpatialAabbV2>,
    clips: Vec<SpatialAabbV2>,
    paints: Vec<SpatialAabbV2>,
    hits: Vec<SpatialAabbV2>,
    semantics: Vec<SpatialAabbV2>,
}

pub(in crate::input_validation) type WorldAabbParts<'a> = (
    WorldTransformProof<'a>,
    Vec<SpatialAabbV2>,
    Vec<SpatialAabbV2>,
    Vec<SpatialAabbV2>,
    Vec<SpatialAabbV2>,
    Vec<SpatialAabbV2>,
);

impl<'a> WorldAabbProof<'a> {
    pub(super) fn input(&self) -> SpatialInputV2<'a> {
        self.transforms.input()
    }

    pub(super) fn limits(&self) -> SpatialLimitsV2 {
        self.transforms.limits()
    }

    pub(super) fn clip_world_aabb(&self, index: usize) -> SpatialAabbV2 {
        *self
            .clips
            .get(index)
            .expect("clip projection retained one primitive bound per record")
    }

    pub(in crate::input_validation) fn into_parts(self) -> WorldAabbParts<'a> {
        (
            self.transforms,
            self.geometry,
            self.clips,
            self.paints,
            self.hits,
            self.semantics,
        )
    }
}

pub(super) fn prepare_world_aabbs(
    transforms: WorldTransformProof<'_>,
) -> Result<WorldAabbProof<'_>, SpatialResolveErrorV2> {
    let input = transforms.input();

    let mut geometry = Vec::with_capacity(input.topology().nodes().len());
    for (index, _) in input.topology().nodes().iter().enumerate() {
        let owner = trusted_ordinal(index, "node");
        geometry.push(project(
            transforms.world_transform(index),
            transforms.base_local_bounds(index),
            owner,
        )?);
    }

    let mut clips = Vec::with_capacity(input.geometry().clips().len());
    for clip in input.geometry().clips() {
        let owner = clip.owner().get();
        clips.push(project(
            transforms.world_transform(trusted_reference(owner, "node")),
            transforms.shape_clip_bounds(clip.shape().get()),
            owner,
        )?);
    }

    let mut paints = Vec::with_capacity(input.items().paint_items().len());
    for (index, paint) in input.items().paint_items().iter().enumerate() {
        let owner = paint.owner().get();
        paints.push(project(
            transforms.world_transform(trusted_reference(owner, "node")),
            transforms.paint_local_bounds(index),
            owner,
        )?);
    }

    let mut hits = Vec::with_capacity(input.items().hit_items().len());
    for (index, hit) in input.items().hit_items().iter().enumerate() {
        let owner = hit.owner().get();
        hits.push(project(
            transforms.world_transform(trusted_reference(owner, "node")),
            transforms.hit_local_bounds(index),
            owner,
        )?);
    }

    let mut semantics = Vec::with_capacity(input.items().semantic_items().len());
    for semantic in input.items().semantic_items() {
        let owner = semantic.owner().get();
        semantics.push(project(
            transforms.world_transform(trusted_reference(owner, "node")),
            transforms.shape_fill_bounds(semantic.shape().get()),
            owner,
        )?);
    }

    Ok(WorldAabbProof {
        transforms,
        geometry,
        clips,
        paints,
        hits,
        semantics,
    })
}

fn project(
    world: Affine2V2,
    local: SpatialAabbV2,
    owner: u32,
) -> Result<SpatialAabbV2, SpatialResolveErrorV2> {
    world.checked_transform_aabb(local).map_err(|operation| {
        make_resolve_error(
            SpatialResolveErrorKindV2::Arithmetic(operation),
            SpatialErrorLocationV2::Node { index: owner },
        )
    })
}

fn trusted_reference(index: u32, table: &str) -> usize {
    usize::try_from(index).unwrap_or_else(|_| panic!("validated {table} reference fits usize"))
}

fn trusted_ordinal(index: usize, table: &str) -> u32 {
    u32::try_from(index).unwrap_or_else(|_| panic!("phase one validated the {table} row capacity"))
}
