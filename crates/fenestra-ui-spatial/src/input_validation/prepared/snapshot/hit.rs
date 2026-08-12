//! Exact allocation-free reverse hit selection over accepted snapshot output.

use super::super::model::{PreparedCoverage, PreparedShapeGeometry, PreparedSpatialState};
use super::SpatialResolvedSnapshotV2;
use crate::aabb::SpatialAabbV2;
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::SpatialFillRuleV2;
use crate::geometry_kernel::{
    ValidatedPolygonK1, circle_fill_contains_k4, circle_round_stroke_contains_k5,
    path_fill_contains_k4, path_round_stroke_contains_k5, polygon_fill_contains_k4,
    polygon_round_stroke_contains_k5, rect_fill_contains_k4, rect_round_stroke_contains_k5,
};
use crate::model::{SpatialNodeKeyV2, SpatialPointV2};
use crate::output_aabb::SpatialOutputAabbV2;

/// Owned identity and local query point for the topmost exact hit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialHitResultV2 {
    key: u32,
    owner: SpatialNodeKeyV2,
    item_ordinal: u32,
    local_point: SpatialPointV2,
}

impl SpatialHitResultV2 {
    const fn new(
        key: u32,
        owner: SpatialNodeKeyV2,
        item_ordinal: u32,
        local_point: SpatialPointV2,
    ) -> Self {
        Self {
            key,
            owner,
            item_ordinal,
            local_point,
        }
    }

    /// Returns the dense global Hit output-row key.
    #[must_use]
    pub const fn key(self) -> u32 {
        self.key
    }

    /// Returns the trusted spatial owner of the winning hit item.
    #[must_use]
    pub const fn owner(self) -> SpatialNodeKeyV2 {
        self.owner
    }

    /// Returns the trusted owner-local item ordinal.
    #[must_use]
    pub const fn item_ordinal(self) -> u32 {
        self.item_ordinal
    }

    /// Returns the scene query mapped into the winning owner's local space.
    #[must_use]
    pub const fn local_point(self) -> SpatialPointV2 {
        self.local_point
    }
}

impl SpatialResolvedSnapshotV2 {
    /// Returns the topmost exact hit at one scene-logical point.
    #[must_use]
    pub fn hit_test(&self, scene_point: SpatialPointV2) -> Option<SpatialHitResultV2> {
        if !scene_point.x().is_in_domain() || !scene_point.y().is_in_domain() {
            return None;
        }

        let state = &self.prepared.state;
        for (row, hit) in self.hits.iter().zip(state.hits.iter()).rev() {
            if hit.input_policy == SpatialInputPolicyV2::Ignore {
                continue;
            }
            if let Some(terminal) = hit.clip
                && !self.clip_chain_contains(state, terminal, scene_point)
            {
                continue;
            }
            if !output_bounds_contains(row.world_aabb(), scene_point) {
                continue;
            }
            let Some(local_point) = row.world_from_local().inverse_point(scene_point) else {
                continue;
            };
            if !self.coverage_contains(state, &hit.coverage, hit.local_bounds, local_point) {
                continue;
            }
            return Some(SpatialHitResultV2::new(
                row.key(),
                SpatialNodeKeyV2::new(hit.owner),
                hit.item_ordinal,
                local_point,
            ));
        }
        None
    }

    fn clip_chain_contains(
        &self,
        state: &PreparedSpatialState,
        terminal: u32,
        scene_point: SpatialPointV2,
    ) -> bool {
        let effective = state.effective_clip_aabbs[terminal as usize];
        if !bounds_contains(effective, scene_point) {
            return false;
        }

        let mut current = Some(terminal);
        while let Some(key) = current {
            let index = key as usize;
            let clip = &state.clips[index];
            let row = self.clips[index];
            if !output_bounds_contains(row.primitive_world_aabb(), scene_point) {
                return false;
            }
            let Some(local_point) = row.world_from_local().inverse_point(scene_point) else {
                return false;
            };
            if !self.shape_fill_contains(state, clip.shape, clip.fill_rule, local_point) {
                return false;
            }
            current = clip.parent;
        }
        true
    }

    fn coverage_contains(
        &self,
        state: &PreparedSpatialState,
        coverage: &PreparedCoverage,
        bounds: SpatialAabbV2,
        local_point: SpatialPointV2,
    ) -> bool {
        match coverage {
            PreparedCoverage::Fill { shape, rule } => {
                self.shape_fill_contains(state, *shape, *rule, local_point)
            }
            PreparedCoverage::RoundStroke { shape, stroke } => {
                self.shape_stroke_contains(state, *shape, bounds, *stroke, local_point)
            }
        }
    }

    fn shape_fill_contains(
        &self,
        state: &PreparedSpatialState,
        shape: u32,
        rule: SpatialFillRuleV2,
        local_point: SpatialPointV2,
    ) -> bool {
        let shape = &state.shapes[shape as usize];
        let bounds = shape.fill_clip_bounds;
        match &shape.geometry {
            PreparedShapeGeometry::Rect { rect } => {
                rect_fill_contains_k4(*rect, bounds, local_point)
            }
            PreparedShapeGeometry::Circle { circle } => {
                circle_fill_contains_k4(*circle, bounds, local_point)
            }
            PreparedShapeGeometry::Polygon { point_range } => {
                let input = self.prepared.source.as_input();
                let points = &input.geometry().polygon_points()[point_range.clone()];
                polygon_fill_contains_k4(
                    ValidatedPolygonK1::from_trusted_points(points),
                    bounds,
                    rule,
                    local_point,
                )
            }
            PreparedShapeGeometry::Path { path } => path_fill_contains_k4(
                &state.paths[*path as usize].flattened,
                bounds,
                rule,
                local_point,
            ),
        }
    }

    fn shape_stroke_contains(
        &self,
        state: &PreparedSpatialState,
        shape: u32,
        bounds: SpatialAabbV2,
        stroke: crate::geometry_kernel::ValidatedStrokeK1,
        local_point: SpatialPointV2,
    ) -> bool {
        match &state.shapes[shape as usize].geometry {
            PreparedShapeGeometry::Rect { rect } => {
                rect_round_stroke_contains_k5(*rect, bounds, stroke, local_point)
            }
            PreparedShapeGeometry::Circle { circle } => {
                circle_round_stroke_contains_k5(*circle, bounds, stroke, local_point)
            }
            PreparedShapeGeometry::Polygon { point_range } => {
                let input = self.prepared.source.as_input();
                let points = &input.geometry().polygon_points()[point_range.clone()];
                polygon_round_stroke_contains_k5(
                    ValidatedPolygonK1::from_trusted_points(points),
                    bounds,
                    stroke,
                    local_point,
                )
            }
            PreparedShapeGeometry::Path { path } => path_round_stroke_contains_k5(
                &state.paths[*path as usize].flattened,
                bounds,
                stroke,
                local_point,
            ),
        }
    }
}

const fn output_bounds_contains(bounds: SpatialOutputAabbV2, point: SpatialPointV2) -> bool {
    if bounds.is_empty() {
        return false;
    }
    bounds.min_x().raw() <= point.x().raw()
        && point.x().raw() <= bounds.max_x().raw()
        && bounds.min_y().raw() <= point.y().raw()
        && point.y().raw() <= bounds.max_y().raw()
}

const fn bounds_contains(bounds: SpatialAabbV2, point: SpatialPointV2) -> bool {
    if bounds.is_empty() {
        return false;
    }
    bounds.min_x().raw() <= point.x().raw()
        && point.x().raw() <= bounds.max_x().raw()
        && bounds.min_y().raw() <= point.y().raw()
        && point.y().raw() <= bounds.max_y().raw()
}
