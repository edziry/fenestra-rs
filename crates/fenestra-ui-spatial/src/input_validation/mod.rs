//! Private aggregate input validation kernel.

// Staged validation proofs remain private until the resolver orchestration lands.
#![allow(dead_code)]

use crate::aggregate_input::SpatialInputV2;
use crate::direct_counts::preflight_spatial_direct_counts_v2;
use crate::error::SpatialErrorLocationV2;
use crate::limits::SpatialLimitsV2;
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

mod brush_structure;
mod dependencies;
mod effective_clip_aabbs;
mod flattened_paths;
mod geometry_k3_mapping;
mod islands;
mod local_bounds;
mod ordered_items;
mod paint_p2_mapping;
mod paint_p4_mapping;
mod paint_p5_mapping;
mod path_k2_mapping;
mod path_structure;
mod placement;
mod prepared;
mod prepared_brushes;
mod shape_k1_mapping;
mod shape_structure;
mod stroke_k1_mapping;
mod topology;
mod transforms;
mod validated_clips;
mod validated_hit_items;
mod validated_images;
mod validated_paint_items;
mod validated_paths;
mod validated_semantic_items;
mod validated_shapes;
mod world_aabbs;
mod world_transforms;

pub use prepared::{
    PreparedSpatialV2, SpatialHitResultV2, SpatialResolvedSnapshotV2,
    materialize_reference_spatial_v2, prepare_spatial_v2, resolve_spatial_v2,
    validate_spatial_output_v2,
};

#[cfg(test)]
use brush_structure::{prepare_brush_structure, validate_gradient_stop_range};

#[cfg(test)]
use dependencies::{
    execute_dependency_graph, map_layout_execution_error, prepare_dependency_graph,
    validate_dependency_fact,
};

#[cfg(test)]
use effective_clip_aabbs::prepare_effective_clip_aabbs;

#[cfg(test)]
use flattened_paths::prepare_flattened_paths;

#[cfg(test)]
use geometry_k3_mapping::map_geometry_k3_error;

#[cfg(test)]
use islands::{
    map_layout_preflight_error, prepare_island_plan, prepare_layout_preflight, validate_island_fact,
};

#[cfg(test)]
use local_bounds::prepare_local_bounds;

#[cfg(test)]
use placement::prepare_placement_input;

#[cfg(test)]
use path_structure::prepare_path_structure;

#[cfg(test)]
use prepared_brushes::prepare_prepared_brushes;

#[cfg(test)]
use paint_p4_mapping::map_image_p4_error;

#[cfg(test)]
use path_k2_mapping::map_path_k2_error;

#[cfg(test)]
use shape_structure::{prepare_shape_structure, validate_polygon_range};

#[cfg(test)]
use shape_k1_mapping::map_shape_k1_error;

#[cfg(test)]
use topology::{prepare_topology, prepare_topology_limits, validate_topology_fact};

#[cfg(test)]
use transforms::prepare_local_transforms;

#[cfg(test)]
use validated_clips::{prepare_validated_clips, validate_clip_depth};

#[cfg(test)]
use world_transforms::prepare_world_transforms;

#[cfg(test)]
use world_aabbs::prepare_world_aabbs;

#[cfg(test)]
use validated_images::prepare_validated_images;

#[cfg(test)]
use validated_hit_items::{prepare_validated_hit_items, validate_hit_item_limit};

#[cfg(test)]
use validated_paint_items::{prepare_validated_paint_items, validate_paint_item_limit};

#[cfg(test)]
use validated_semantic_items::prepare_validated_semantic_items;

#[cfg(test)]
use validated_paths::{map_path_k1_error, prepare_validated_paths};

#[cfg(test)]
use validated_shapes::prepare_validated_shapes;

struct DirectCountProof<'a> {
    input: SpatialInputV2<'a>,
    limits: SpatialLimitsV2,
}

impl<'a> DirectCountProof<'a> {
    pub(in crate::input_validation) const fn into_parts(
        self,
    ) -> (SpatialInputV2<'a>, SpatialLimitsV2) {
        (self.input, self.limits)
    }
}

fn prepare_direct_counts(
    input: SpatialInputV2<'_>,
    limits: SpatialLimitsV2,
) -> Result<DirectCountProof<'_>, SpatialResolveErrorV2> {
    let topology = input.topology();
    let geometry = input.geometry();
    let resources = input.resources();
    let items = input.items();
    let observed = [
        topology.nodes().len() as u128,
        geometry.shapes().len() as u128,
        resources.brushes().len() as u128,
        geometry.clips().len() as u128,
        items.paint_items().len() as u128,
        items.hit_items().len() as u128,
        items.semantic_items().len() as u128,
        geometry.paths().len() as u128,
        geometry.path_verbs().len() as u128,
        geometry.polygon_points().len() as u128,
        resources.gradient_stops().len() as u128,
        resources.images().len() as u128,
    ];

    preflight_spatial_direct_counts_v2(observed, limits)?;

    Ok(DirectCountProof { input, limits })
}

fn make_resolve_error(
    kind: SpatialResolveErrorKindV2,
    location: SpatialErrorLocationV2,
) -> SpatialResolveErrorV2 {
    SpatialResolveErrorV2::non_limit(kind, location)
}

#[cfg(test)]
mod tests;
