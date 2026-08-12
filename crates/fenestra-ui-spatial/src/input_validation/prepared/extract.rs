//! Consuming conversion from borrowed phase-10 proofs to lifetime-free state.

use super::model::{
    PreparedBaseGeometry, PreparedSpatialState, PreparedTopologyNode, PreparedWorldAabbs,
};
use crate::input_validation::effective_clip_aabbs::EffectiveClipAabbProof;

mod items;
mod resources;

pub(super) fn extract_prepared_state(proof: EffectiveClipAabbProof<'_>) -> PreparedSpatialState {
    let (world_aabbs, effective_clip_aabbs) = proof.into_parts();
    let (world_transforms, geometry_aabbs, clip_aabbs, paint_aabbs, hit_aabbs, semantic_aabbs) =
        world_aabbs.into_parts();
    let (base_placements, world_transforms) = world_transforms.into_parts();
    let (dependency_graph, base_placements) = base_placements.into_parts();
    let local_bounds = dependency_graph.into_parts();
    let (flattened_paths, shape_bounds, paint_bounds, hit_bounds) = local_bounds.into_parts();
    let (semantic_items, flattened_paths) = flattened_paths.into_parts();
    let (hit_items, semantic_items) = semantic_items.into_parts();
    let (paint_items, hit_items) = hit_items.into_parts();
    let (clips, paint_items) = paint_items.into_parts();
    let (images, clips) = clips.into_parts();
    let (brushes, images) = images.into_parts();
    let (brush_structure, brushes) = brushes.into_parts();
    let (shapes, gradient_ranges) = brush_structure.into_parts();
    let (shape_structure, shapes) = shapes.into_parts();
    let (paths, polygon_ranges) = shape_structure.into_parts();
    let (path_structure, paths) = paths.into_parts();
    let (local_transforms, path_ranges) = path_structure.into_parts();
    let preflight = local_transforms.into_parts();
    let placement = preflight.into_parts();
    let topology_limits = placement.into_parts();
    let topology = topology_limits.into_parts();
    let (direct_counts, depths, _) = topology.into_parts();
    let (input, limits) = direct_counts.into_parts();

    let nodes = input.topology().nodes();
    assert_eq!(
        nodes.len(),
        depths.len(),
        "topology depths remain key aligned"
    );
    let topology = nodes
        .iter()
        .zip(depths)
        .map(|(node, depth)| PreparedTopologyNode {
            parent: node.parent().map(|parent| parent.get()),
            depth,
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();

    let base_geometry = base_placements
        .into_iter()
        .map(|placement| {
            let (x, y, width, height, _, _, _, _) = placement.into_parts();
            PreparedBaseGeometry {
                x,
                y,
                width,
                height,
            }
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();

    let paths = resources::extract_paths(path_ranges, paths, flattened_paths);
    let shapes = resources::extract_shapes(shapes, polygon_ranges, shape_bounds);
    let brushes = resources::extract_brushes(brushes, gradient_ranges);
    let images = resources::extract_images(images);
    let clips = items::extract_clips(clips);
    let paints = items::extract_paints(paint_items, paint_bounds);
    let hits = items::extract_hits(hit_items, hit_bounds);
    let semantics = items::extract_semantics(semantic_items);

    PreparedSpatialState {
        viewport: input.topology().viewport(),
        limits,
        topology,
        paths,
        shapes,
        brushes,
        images,
        clips,
        paints,
        hits,
        semantics,
        base_geometry,
        world_transforms: world_transforms.into_boxed_slice(),
        world_aabbs: PreparedWorldAabbs {
            geometry: geometry_aabbs.into_boxed_slice(),
            clips: clip_aabbs.into_boxed_slice(),
            paints: paint_aabbs.into_boxed_slice(),
            hits: hit_aabbs.into_boxed_slice(),
            semantics: semantic_aabbs.into_boxed_slice(),
        },
        effective_clip_aabbs: effective_clip_aabbs.into_boxed_slice(),
    }
}
