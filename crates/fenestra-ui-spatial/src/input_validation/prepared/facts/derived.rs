use super::super::PreparedSpatialV2;
use super::super::model::PreparedBaseGeometry;
use crate::aabb::SpatialAabbV2;
use crate::limits::SpatialLimitsV2;
use crate::model::Affine2V2;

type AabbFact = (u32, SpatialAabbV2);

impl PreparedSpatialV2 {
    pub(in crate::input_validation) fn limits(&self) -> SpatialLimitsV2 {
        self.state.limits
    }

    pub(in crate::input_validation) fn topology_facts(&self) -> Vec<(u32, Option<u32>, usize)> {
        self.state
            .topology
            .iter()
            .enumerate()
            .map(|(index, node)| (ordinal(index), node.parent, node.depth))
            .collect()
    }

    pub(in crate::input_validation) fn base_geometry_facts(
        &self,
    ) -> Vec<(u32, i64, i64, i32, i32)> {
        self.state
            .base_geometry
            .iter()
            .enumerate()
            .map(base_fact)
            .collect()
    }

    pub(in crate::input_validation) fn world_transform_facts(&self) -> Vec<(u32, [i64; 6])> {
        self.state
            .world_transforms
            .iter()
            .copied()
            .enumerate()
            .map(|(index, affine)| (ordinal(index), affine_fact(affine)))
            .collect()
    }

    pub(in crate::input_validation) fn geometry_world_aabb_facts(&self) -> Vec<AabbFact> {
        aabb_facts(&self.state.world_aabbs.geometry)
    }

    pub(in crate::input_validation) fn clip_world_aabb_facts(&self) -> Vec<AabbFact> {
        aabb_facts(&self.state.world_aabbs.clips)
    }

    pub(in crate::input_validation) fn effective_clip_world_aabb_facts(&self) -> Vec<AabbFact> {
        aabb_facts(&self.state.effective_clip_aabbs)
    }

    pub(in crate::input_validation) fn paint_world_aabb_facts(&self) -> Vec<AabbFact> {
        aabb_facts(&self.state.world_aabbs.paints)
    }

    pub(in crate::input_validation) fn hit_world_aabb_facts(&self) -> Vec<AabbFact> {
        aabb_facts(&self.state.world_aabbs.hits)
    }

    pub(in crate::input_validation) fn semantic_world_aabb_facts(&self) -> Vec<AabbFact> {
        aabb_facts(&self.state.world_aabbs.semantics)
    }
}

fn base_fact((index, base): (usize, &PreparedBaseGeometry)) -> (u32, i64, i64, i32, i32) {
    (
        ordinal(index),
        base.x.raw(),
        base.y.raw(),
        base.width,
        base.height,
    )
}

fn affine_fact(affine: Affine2V2) -> [i64; 6] {
    [
        affine.a().raw(),
        affine.b().raw(),
        affine.c().raw(),
        affine.d().raw(),
        affine.tx().raw(),
        affine.ty().raw(),
    ]
}

fn aabb_facts(bounds: &[SpatialAabbV2]) -> Vec<AabbFact> {
    bounds
        .iter()
        .copied()
        .enumerate()
        .map(|(index, bounds)| (ordinal(index), bounds))
        .collect()
}

fn ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("prepared table ordinal fits u32")
}
