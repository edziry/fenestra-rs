use super::world_aabb_support::{
    ProjectionTable, ScriptedLayoutEngine, VIEWPORT, expect_aabb_error, geometry_fault_fixture,
    limits, projection_fault_fixture,
};
use crate::numeric_error::SpatialArithmeticOperationV2;

const OPERATIONS: [SpatialArithmeticOperationV2; 4] = [
    SpatialArithmeticOperationV2::AabbMinX,
    SpatialArithmeticOperationV2::AabbMinY,
    SpatialArithmeticOperationV2::AabbMaxX,
    SpatialArithmeticOperationV2::AabbMaxY,
];

#[test]
fn every_geometry_edge_failure_maps_to_its_owning_nonroot_node() {
    for operation in OPERATIONS {
        let fixture = geometry_fault_fixture(operation);
        let engine = ScriptedLayoutEngine::new(Vec::new());
        expect_aabb_error(
            prepare_world_aabbs!(&fixture, VIEWPORT, limits(), &engine),
            1,
            operation,
        );
    }
}

#[test]
fn every_clip_edge_failure_maps_the_clip_record_to_its_owner() {
    assert_projection_errors(ProjectionTable::Clip);
}

#[test]
fn every_paint_edge_failure_maps_the_paint_record_to_its_owner() {
    assert_projection_errors(ProjectionTable::Paint);
}

#[test]
fn every_hit_edge_failure_maps_the_hit_record_to_its_owner() {
    assert_projection_errors(ProjectionTable::Hit);
}

#[test]
fn every_semantic_edge_failure_maps_the_semantic_record_to_its_owner() {
    assert_projection_errors(ProjectionTable::Semantic);
}

fn assert_projection_errors(table: ProjectionTable) {
    for operation in OPERATIONS {
        let fixture = projection_fault_fixture(table, operation);
        let engine = ScriptedLayoutEngine::new(Vec::new());
        expect_aabb_error(
            prepare_world_aabbs!(&fixture, VIEWPORT, limits(), &engine),
            2,
            operation,
        );
    }
}
