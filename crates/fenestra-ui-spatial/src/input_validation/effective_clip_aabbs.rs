//! Effective clip-chain AABBs resolved in dense clip order.

use super::world_aabbs::WorldAabbProof;
use crate::aabb::SpatialAabbV2;
use crate::aggregate_input::SpatialInputV2;
use crate::limits::SpatialLimitsV2;

#[cfg(test)]
mod facts;

pub(super) struct EffectiveClipAabbProof<'a> {
    world: WorldAabbProof<'a>,
    effective_clips: Vec<SpatialAabbV2>,
}

impl<'a> EffectiveClipAabbProof<'a> {
    pub(super) fn input(&self) -> SpatialInputV2<'a> {
        self.world.input()
    }

    pub(super) fn limits(&self) -> SpatialLimitsV2 {
        self.world.limits()
    }
}

pub(super) fn prepare_effective_clip_aabbs(
    world: WorldAabbProof<'_>,
) -> EffectiveClipAabbProof<'_> {
    let input = world.input();
    let clips = input.geometry().clips();
    let mut effective_clips = Vec::with_capacity(clips.len());

    for (index, clip) in clips.iter().copied().enumerate() {
        let primitive = world.clip_world_aabb(index);
        let effective = match clip.parent() {
            None => primitive,
            Some(parent) => primitive.intersection(
                *effective_clips
                    .get(trusted_reference(parent.get()))
                    .expect("clip validation retained only earlier parents"),
            ),
        };
        effective_clips.push(effective);
    }

    EffectiveClipAabbProof {
        world,
        effective_clips,
    }
}

fn trusted_reference(index: u32) -> usize {
    usize::try_from(index).expect("phase seven validated the clip parent capacity")
}
