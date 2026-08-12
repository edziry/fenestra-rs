//! Lifetime-free ownership boundary after complete spatial phase-10 preparation.

use std::sync::Arc;

use fenestra_ui_layout::prototype::LayoutEngineV1;

use self::model::PreparedSpatialState;
use super::effective_clip_aabbs::{EffectiveClipAabbProof, prepare_effective_clip_aabbs};
use super::{
    brush_structure::prepare_brush_structure,
    dependencies::{execute_dependency_graph, prepare_dependency_graph},
    flattened_paths::prepare_flattened_paths,
    islands::{prepare_island_plan, prepare_layout_preflight},
    local_bounds::prepare_local_bounds,
    path_structure::prepare_path_structure,
    placement::prepare_placement_input,
    prepare_direct_counts,
    prepared_brushes::prepare_prepared_brushes,
    shape_structure::prepare_shape_structure,
    topology::{prepare_topology, prepare_topology_limits},
    transforms::prepare_local_transforms,
    validated_clips::prepare_validated_clips,
    validated_hit_items::prepare_validated_hit_items,
    validated_images::prepare_validated_images,
    validated_paint_items::prepare_validated_paint_items,
    validated_paths::prepare_validated_paths,
    validated_semantic_items::prepare_validated_semantic_items,
    validated_shapes::prepare_validated_shapes,
    world_aabbs::prepare_world_aabbs,
    world_transforms::prepare_world_transforms,
};
use crate::aggregate_input::SpatialInputV2;
use crate::limits::SpatialLimitsV2;
use crate::owned_input::SpatialOwnedInputV2;
use crate::resolve_error::SpatialResolveErrorV2;

mod extract;
#[cfg(test)]
mod facts;
mod model;
mod snapshot;

pub use snapshot::{
    SpatialHitResultV2, SpatialPaintFrameV2, SpatialResolvedSnapshotV2,
    materialize_reference_spatial_v2, validate_spatial_output_v2,
};

/// Opaque lifetime-free result of complete spatial phase-10 preparation.
pub struct PreparedSpatialV2 {
    state: PreparedSpatialState,
    source: Arc<SpatialOwnedInputV2>,
}

/// Validates and prepares one immutable owned spatial input through phase 10.
#[must_use = "spatial preparation errors must be handled before publication"]
pub fn prepare_spatial_v2<E: LayoutEngineV1 + ?Sized>(
    engine: &E,
    input: Arc<SpatialOwnedInputV2>,
    limits: SpatialLimitsV2,
) -> Result<PreparedSpatialV2, SpatialResolveErrorV2> {
    let state = {
        let borrowed = input.as_input();
        let proof = prepare_phase_10(engine, borrowed, limits)?;
        extract::extract_prepared_state(proof)
    };
    Ok(PreparedSpatialV2 {
        state,
        source: input,
    })
}

/// Prepares and materializes one immutable reference spatial snapshot.
#[must_use = "spatial resolution errors must be handled before publication"]
pub fn resolve_spatial_v2<E: LayoutEngineV1 + ?Sized>(
    engine: &E,
    input: Arc<SpatialOwnedInputV2>,
    limits: SpatialLimitsV2,
) -> Result<SpatialResolvedSnapshotV2, SpatialResolveErrorV2> {
    let prepared = prepare_spatial_v2(engine, input, limits)?;
    Ok(materialize_reference_spatial_v2(prepared))
}

fn prepare_phase_10<'a, E: LayoutEngineV1 + ?Sized>(
    engine: &E,
    input: SpatialInputV2<'a>,
    limits: SpatialLimitsV2,
) -> Result<EffectiveClipAabbProof<'a>, SpatialResolveErrorV2> {
    let direct = prepare_direct_counts(input, limits)?;
    let topology = prepare_topology(direct)?;
    let topology = prepare_topology_limits(topology)?;
    let placement = prepare_placement_input(topology)?;
    let islands = prepare_island_plan(placement)?;
    let preflight = prepare_layout_preflight(islands)?;
    let transforms = prepare_local_transforms(preflight)?;
    let paths = prepare_path_structure(transforms)?;
    let paths = prepare_validated_paths(paths)?;
    let shapes = prepare_shape_structure(paths)?;
    let shapes = prepare_validated_shapes(shapes)?;
    let brushes = prepare_brush_structure(shapes)?;
    let brushes = prepare_prepared_brushes(brushes)?;
    let images = prepare_validated_images(brushes)?;
    let clips = prepare_validated_clips(images)?;
    let paints = prepare_validated_paint_items(clips)?;
    let hits = prepare_validated_hit_items(paints)?;
    let semantics = prepare_validated_semantic_items(hits)?;
    let paths = prepare_flattened_paths(semantics)?;
    let bounds = prepare_local_bounds(paths)?;
    let graph = prepare_dependency_graph(bounds)?;
    let placements = execute_dependency_graph(graph, engine)?;
    let transforms = prepare_world_transforms(placements)?;
    let aabbs = prepare_world_aabbs(transforms)?;
    Ok(prepare_effective_clip_aabbs(aabbs))
}
