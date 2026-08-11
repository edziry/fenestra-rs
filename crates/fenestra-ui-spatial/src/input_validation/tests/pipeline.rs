//! Reusable test-only validation pipeline stages.

macro_rules! prepare_island_plan {
    ($fixture:expr, $limits:expr) => {{
        prepare_island_plan!(
            $fixture,
            $crate::input_validation::tests::island_support::zero_viewport(),
            $limits
        )
    }};
    ($fixture:expr, $viewport:expr, $limits:expr) => {{
        $crate::input_validation::prepare_direct_counts(
            ($fixture).input_with_viewport($viewport),
            $limits,
        )
        .and_then($crate::input_validation::prepare_topology)
        .and_then($crate::input_validation::prepare_topology_limits)
        .and_then($crate::input_validation::prepare_placement_input)
        .and_then($crate::input_validation::tests::prepare_island_plan_stage)
    }};
}

macro_rules! prepare_layout_preflight {
    ($fixture:expr, $viewport:expr, $limits:expr) => {{
        prepare_island_plan!($fixture, $viewport, $limits)
            .and_then($crate::input_validation::tests::prepare_layout_preflight_stage)
    }};
}

macro_rules! map_layout_preflight_error {
    ($plan:expr, $item:expr, $kind:expr, $location:expr) => {{
        $crate::input_validation::tests::map_layout_preflight_error_stage(
            &$plan, $item, $kind, $location,
        )
    }};
}

macro_rules! prepare_local_transforms {
    ($fixture:expr, $viewport:expr, $limits:expr) => {{
        prepare_layout_preflight!($fixture, $viewport, $limits)
            .and_then($crate::input_validation::tests::prepare_local_transforms_stage)
    }};
}

macro_rules! prepare_path_structure {
    ($fixture:expr, $viewport:expr, $limits:expr) => {{
        prepare_local_transforms!($fixture, $viewport, $limits)
            .and_then($crate::input_validation::tests::prepare_path_structure_stage)
    }};
}

macro_rules! prepare_validated_paths {
    ($fixture:expr, $viewport:expr, $limits:expr) => {{
        prepare_path_structure!($fixture, $viewport, $limits)
            .and_then($crate::input_validation::tests::prepare_validated_paths_stage)
    }};
}

macro_rules! prepare_shape_structure {
    ($fixture:expr, $viewport:expr, $limits:expr) => {{
        prepare_validated_paths!($fixture, $viewport, $limits)
            .and_then($crate::input_validation::tests::prepare_shape_structure_stage)
    }};
}

macro_rules! prepare_validated_shapes {
    ($fixture:expr, $viewport:expr, $limits:expr) => {{
        prepare_shape_structure!($fixture, $viewport, $limits)
            .and_then($crate::input_validation::tests::prepare_validated_shapes_stage)
    }};
}

macro_rules! prepare_brush_structure {
    ($fixture:expr, $viewport:expr, $limits:expr) => {{
        prepare_validated_shapes!($fixture, $viewport, $limits)
            .and_then($crate::input_validation::tests::prepare_brush_structure_stage)
    }};
}

macro_rules! prepare_prepared_brushes {
    ($fixture:expr, $viewport:expr, $limits:expr) => {{
        prepare_brush_structure!($fixture, $viewport, $limits)
            .and_then($crate::input_validation::tests::prepare_prepared_brushes_stage)
    }};
}

macro_rules! prepare_validated_images {
    ($fixture:expr, $viewport:expr, $limits:expr) => {{
        prepare_prepared_brushes!($fixture, $viewport, $limits)
            .and_then($crate::input_validation::tests::prepare_validated_images_stage)
    }};
}

macro_rules! prepare_validated_clips {
    ($fixture:expr, $viewport:expr, $limits:expr) => {{
        prepare_validated_images!($fixture, $viewport, $limits)
            .and_then($crate::input_validation::tests::prepare_validated_clips_stage)
    }};
}

macro_rules! prepare_validated_paint_items {
    ($fixture:expr, $viewport:expr, $limits:expr) => {{
        prepare_validated_clips!($fixture, $viewport, $limits)
            .and_then($crate::input_validation::tests::prepare_validated_paint_items_stage)
    }};
}

macro_rules! prepare_validated_hit_items {
    ($fixture:expr, $viewport:expr, $limits:expr) => {{
        prepare_validated_paint_items!($fixture, $viewport, $limits)
            .and_then($crate::input_validation::tests::prepare_validated_hit_items_stage)
    }};
}

macro_rules! prepare_validated_semantic_items {
    ($fixture:expr, $viewport:expr, $limits:expr) => {{
        prepare_validated_hit_items!($fixture, $viewport, $limits)
            .and_then($crate::input_validation::tests::prepare_validated_semantic_items_stage)
    }};
}

macro_rules! prepare_flattened_paths {
    ($fixture:expr, $viewport:expr, $limits:expr) => {{
        prepare_validated_semantic_items!($fixture, $viewport, $limits)
            .and_then($crate::input_validation::tests::prepare_flattened_paths_stage)
    }};
}

macro_rules! prepare_local_bounds {
    ($fixture:expr, $viewport:expr, $limits:expr) => {{
        prepare_flattened_paths!($fixture, $viewport, $limits)
            .and_then($crate::input_validation::tests::prepare_local_bounds_stage)
    }};
}

macro_rules! prepare_dependency_graph {
    ($fixture:expr, $viewport:expr, $limits:expr) => {{
        prepare_local_bounds!($fixture, $viewport, $limits)
            .and_then($crate::input_validation::tests::prepare_dependency_graph_stage)
    }};
}

macro_rules! execute_dependency_graph {
    ($fixture:expr, $viewport:expr, $limits:expr, $engine:expr) => {{
        prepare_dependency_graph!($fixture, $viewport, $limits).and_then(|graph| {
            $crate::input_validation::tests::execute_dependency_graph_stage(graph, $engine)
        })
    }};
}

macro_rules! map_layout_execution_error {
    ($graph:expr, $island:expr, $kind:expr, $location:expr) => {{
        $crate::input_validation::tests::map_layout_execution_error_stage(
            &$graph, $island, $kind, $location,
        )
    }};
}

macro_rules! prepare_world_transforms {
    ($fixture:expr, $viewport:expr, $limits:expr, $engine:expr) => {{
        execute_dependency_graph!($fixture, $viewport, $limits, $engine)
            .and_then($crate::input_validation::tests::prepare_world_transforms_stage)
    }};
}

macro_rules! prepare_world_aabbs {
    ($fixture:expr, $viewport:expr, $limits:expr, $engine:expr) => {{
        prepare_world_transforms!($fixture, $viewport, $limits, $engine)
            .and_then($crate::input_validation::tests::prepare_world_aabbs_stage)
    }};
}

macro_rules! prepare_effective_clip_aabbs {
    ($fixture:expr, $viewport:expr, $limits:expr, $engine:expr) => {{
        prepare_world_aabbs!($fixture, $viewport, $limits, $engine)
            .map($crate::input_validation::tests::prepare_effective_clip_aabbs_stage)
    }};
}
