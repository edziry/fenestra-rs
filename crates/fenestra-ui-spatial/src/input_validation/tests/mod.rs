use std::error::Error;

use super::{
    execute_dependency_graph as execute_dependency_graph_stage, make_resolve_error,
    map_geometry_k3_error as map_geometry_k3_error_stage,
    map_image_p4_error as map_image_p4_error_stage,
    map_layout_execution_error as map_layout_execution_error_stage,
    map_layout_preflight_error as map_layout_preflight_error_stage,
    map_path_k1_error as map_path_k1_error_stage, map_path_k2_error as map_path_k2_error_stage,
    map_shape_k1_error as map_shape_k1_error_stage,
    prepare_brush_structure as prepare_brush_structure_stage,
    prepare_dependency_graph as prepare_dependency_graph_stage, prepare_direct_counts,
    prepare_effective_clip_aabbs as prepare_effective_clip_aabbs_stage,
    prepare_flattened_paths as prepare_flattened_paths_stage,
    prepare_island_plan as prepare_island_plan_stage,
    prepare_layout_preflight as prepare_layout_preflight_stage,
    prepare_local_bounds as prepare_local_bounds_stage,
    prepare_local_transforms as prepare_local_transforms_stage,
    prepare_path_structure as prepare_path_structure_stage,
    prepare_prepared_brushes as prepare_prepared_brushes_stage,
    prepare_shape_structure as prepare_shape_structure_stage, prepare_topology,
    prepare_validated_clips as prepare_validated_clips_stage,
    prepare_validated_hit_items as prepare_validated_hit_items_stage,
    prepare_validated_images as prepare_validated_images_stage,
    prepare_validated_paint_items as prepare_validated_paint_items_stage,
    prepare_validated_paths as prepare_validated_paths_stage,
    prepare_validated_semantic_items as prepare_validated_semantic_items_stage,
    prepare_validated_shapes as prepare_validated_shapes_stage,
    prepare_world_aabbs as prepare_world_aabbs_stage,
    prepare_world_transforms as prepare_world_transforms_stage,
    validate_clip_depth as validate_clip_depth_stage,
    validate_dependency_fact as validate_dependency_fact_stage, validate_direct_count,
    validate_gradient_stop_range as validate_gradient_stop_range_stage,
    validate_hit_item_limit as validate_hit_item_limit_stage,
    validate_island_fact as validate_island_fact_stage,
    validate_paint_item_limit as validate_paint_item_limit_stage,
    validate_polygon_range as validate_polygon_range_stage,
};
use crate::error::SpatialErrorLocationV2;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

const DIRECT_COUNT: usize = SpatialLimitKindV2::DIRECT_ALL.len();
const U32_ROW_CAPACITY: u128 = u32::MAX as u128 + 1;

const GLOBALLY_INDEXED_DIRECT_INDICES: [usize; 9] = [0, 1, 2, 3, 4, 5, 6, 7, 11];
const PAYLOAD_DIRECT_INDICES: [usize; 3] = [8, 9, 10];

#[macro_use]
mod pipeline;

mod brush_structure_keys;
mod brush_structure_priority;
mod brush_structure_ranges;
mod brush_structure_success;
mod brush_structure_support;
mod counts;
mod dependency_cycles;
mod dependency_graph;
mod dependency_limits;
mod dependency_priority;
mod dependency_success;
mod dependency_support;
mod dependency_targets;
mod effective_clip_aabb_retention;
mod effective_clip_aabb_success;
mod effective_clip_aabb_support;
mod errors;
mod fixture;
mod flattened_path_errors;
mod flattened_path_limits;
mod flattened_path_priority;
mod flattened_path_retention;
mod flattened_path_success;
mod flattened_path_support;
mod input;
mod island_limits;
mod island_support;
mod islands;
mod layout_preflight;
mod layout_preflight_bridge;
mod layout_preflight_mappings;
mod layout_preflight_support;
mod local_bounds_errors;
mod local_bounds_priority;
mod local_bounds_success;
mod local_bounds_support;
mod local_transform_deferral;
mod local_transform_determinants;
mod local_transform_priority;
mod local_transform_scalars;
mod local_transform_support;
mod path_structure_keys;
mod path_structure_ranges;
mod path_structure_success;
mod path_structure_support;
mod placement;
mod placement_execution_arithmetic;
mod placement_execution_arithmetic_priority;
mod placement_execution_bridge;
mod placement_execution_layout;
mod placement_execution_output;
mod placement_execution_priority;
mod placement_execution_retention;
mod placement_execution_success;
mod placement_execution_support;
mod prepared_brush_limits;
mod prepared_brush_priority;
mod prepared_brush_scalars;
mod prepared_brush_semantics;
mod prepared_brush_success;
mod prepared_brush_support;
mod prepared_spatial_contract;
mod shape_structure_keys;
mod shape_structure_priority;
mod shape_structure_ranges;
mod shape_structure_success;
mod shape_structure_support;
mod topology;
mod topology_limits;
mod validated_clip_ancestry;
mod validated_clip_keys;
mod validated_clip_limits;
mod validated_clip_priority;
mod validated_clip_references;
mod validated_clip_success;
mod validated_clip_support;
mod validated_hit_coverage;
mod validated_hit_limits;
mod validated_hit_order;
mod validated_hit_priority;
mod validated_hit_success;
mod validated_hit_support;
mod validated_image_extents;
mod validated_image_keys;
mod validated_image_layout;
mod validated_image_limits;
mod validated_image_pixels;
mod validated_image_priority;
mod validated_image_success;
mod validated_image_support;
mod validated_paint_coverage;
mod validated_paint_image_destination;
mod validated_paint_image_source;
mod validated_paint_limits;
mod validated_paint_order;
mod validated_paint_priority;
mod validated_paint_success;
mod validated_paint_support;
mod validated_path_grammar;
mod validated_path_limits;
mod validated_path_priority;
mod validated_path_scalars;
mod validated_path_success;
mod validated_path_support;
mod validated_semantic_order;
mod validated_semantic_priority;
mod validated_semantic_references;
mod validated_semantic_success;
mod validated_semantic_support;
mod validated_shape_limits;
mod validated_shape_priority;
mod validated_shape_scalars;
mod validated_shape_semantics;
mod validated_shape_success;
mod validated_shape_support;
mod world_aabb_errors;
mod world_aabb_priority;
mod world_aabb_retention;
mod world_aabb_success;
mod world_aabb_support;
mod world_transform_arithmetic;
mod world_transform_priority;
mod world_transform_retention;
mod world_transform_success;
mod world_transform_support;

fn check_island_fact(
    kind: SpatialLimitKindV2,
    index: Option<u32>,
    observed: u128,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    validate_island_fact_stage(kind, index, observed, limits)
}

fn check_polygon_range(
    shape: u32,
    cursor: u128,
    start: u32,
    length: u32,
    point_count: u128,
) -> Result<u128, SpatialResolveErrorV2> {
    validate_polygon_range_stage(shape, cursor, start, length, point_count)
}

fn check_gradient_stop_range(
    brush: u32,
    cursor: u128,
    start: u32,
    length: u32,
    stop_count: u128,
) -> Result<u128, SpatialResolveErrorV2> {
    validate_gradient_stop_range_stage(brush, cursor, start, length, stop_count)
}

fn check_clip_depth(
    clip: u32,
    observed: usize,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    validate_clip_depth_stage(clip, observed, limits)
}

fn check_paint_item_limit(
    paint: u32,
    observed: usize,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    validate_paint_item_limit_stage(paint, observed, limits)
}

fn check_hit_item_limit(
    hit: u32,
    observed: usize,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    validate_hit_item_limit_stage(hit, observed, limits)
}

fn check_dependency_fact(
    kind: SpatialLimitKindV2,
    observed: u128,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    validate_dependency_fact_stage(kind, observed, limits)
}

fn limits_with_direct(maxima: [usize; DIRECT_COUNT]) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    values[..DIRECT_COUNT].copy_from_slice(&maxima);
    SpatialLimitsV2::new(values)
}

fn expect_valid<T>(result: Result<T, SpatialResolveErrorV2>) {
    if let Err(error) = result {
        panic!("expected direct-count validation success, got {error:?}");
    }
}

fn expect_limit<T>(
    result: Result<T, SpatialResolveErrorV2>,
    limit: SpatialLimitKindV2,
    observed: u128,
    maximum: u128,
) {
    let error = match result {
        Ok(_) => panic!("expected direct-count limit failure"),
        Err(error) => error,
    };

    assert_eq!(
        error.kind(),
        SpatialResolveErrorKindV2::LimitExceeded(limit)
    );
    assert_eq!(error.location(), SpatialErrorLocationV2::Input);
    assert_eq!(error.observed(), Some(observed));
    assert_eq!(error.maximum(), Some(maximum));
    assert_eq!(error.to_string(), "spatial-resolve-error(limit-exceeded)");
    assert_eq!(
        format!("{error:?}"),
        "SpatialResolveErrorV2(spatial-resolve-error(limit-exceeded))"
    );
    assert!(Error::source(&error).is_none());
}
