use super::super::world_aabb_support::{aabb, fact};
use super::super::world_transform_support::{SCALE, world};
use super::support::{requested_limits, root_only_owned, zero_call_engine};
use super::*;

#[test]
fn root_only_input_prepares_without_layout_or_synthetic_resource_rows() {
    let engine = zero_call_engine();
    let prepared = prepare_spatial_v2(&engine, root_only_owned(), requested_limits())
        .expect("a root-only owned input is valid");

    assert_eq!(engine.call_count(), 0);
    assert_eq!(prepared.topology_facts(), vec![(0, None, 1)]);
    assert_eq!(prepared.base_geometry_facts(), vec![(0, 0, 0, 20, 20)]);
    assert_eq!(
        prepared.world_transform_facts(),
        vec![world(0, [SCALE, 0, 0, SCALE, 0, 0])]
    );
    assert_eq!(
        prepared.geometry_world_aabb_facts(),
        vec![fact(0, aabb(0, 0, 20 * SCALE, 20 * SCALE))]
    );
    assert!(prepared.clip_world_aabb_facts().is_empty());
    assert!(prepared.effective_clip_world_aabb_facts().is_empty());
    assert!(prepared.paint_world_aabb_facts().is_empty());
    assert!(prepared.hit_world_aabb_facts().is_empty());
    assert!(prepared.semantic_world_aabb_facts().is_empty());
    assert!(prepared.path_range_facts().is_empty());
    assert!(prepared.flattened_path_facts().is_empty());
    assert!(prepared.polygon_range_facts().is_empty());
    assert!(prepared.shape_plan_facts().is_empty());
    assert!(prepared.gradient_range_facts().is_empty());
    assert!(prepared.prepared_brush_facts().is_empty());
    assert!(prepared.image_plan_facts().is_empty());
    assert!(prepared.validated_clip_facts().is_empty());
    assert!(prepared.validated_paint_facts().is_empty());
    assert!(prepared.validated_hit_facts().is_empty());
    assert!(prepared.validated_semantic_facts().is_empty());
}
