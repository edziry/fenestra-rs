use super::validated_clip_support::root_clip;
use super::world_aabb_support::{
    MAXIMUM, SCALE, ScriptedLayoutEngine, VIEWPORT, expect_aabb_error, fault_shape, fill,
    fixture_with, hit, limits, owner_node, projection_transform, root, semantic_fill, transform,
};
use super::world_transform_support::{expect_arithmetic, free};
use crate::model::SpatialAnchorTargetV2;
use crate::numeric_error::SpatialArithmeticOperationV2;
use crate::vocabulary::{SpatialAffineComponentV2, SpatialTransformStageV2};

#[test]
fn every_world_transform_finishes_before_the_first_geometry_aabb() {
    let fixture = fixture_with(
        vec![
            root(),
            owner_node(1, transform([SCALE, 0, 0, MAXIMUM, 0, 0], 0, 0), 1, 2),
            free(
                2,
                0,
                SpatialAnchorTargetV2::Viewport,
                SCALE,
                0,
                1,
                1,
                transform([SCALE, 0, 0, SCALE, MAXIMUM, 0], 0, 0),
            ),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let engine = ScriptedLayoutEngine::new(Vec::new());

    expect_arithmetic(
        prepare_world_aabbs!(&fixture, VIEWPORT, limits(), &engine),
        2,
        SpatialTransformStageV2::Placed,
        SpatialAffineComponentV2::Tx,
    );
}

#[test]
fn complete_output_table_passes_use_geometry_clip_paint_hit_semantic_order() {
    let fixtures = [
        geometry_before_clip(),
        clip_before_paint(),
        paint_before_hit(),
        hit_before_semantic(),
    ];
    for fixture in fixtures {
        let engine = ScriptedLayoutEngine::new(Vec::new());
        expect_aabb_error(
            prepare_world_aabbs!(&fixture, VIEWPORT, limits(), &engine),
            2,
            SpatialArithmeticOperationV2::AabbMaxY,
        );
    }
}

#[test]
fn paint_records_finish_before_the_next_records_earlier_edge_failure() {
    let fixture = projection_pair(
        vec![fill(1, 0, 0, None), fill(2, 0, 1, None)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let engine = ScriptedLayoutEngine::new(Vec::new());
    expect_aabb_error(
        prepare_world_aabbs!(&fixture, VIEWPORT, limits(), &engine),
        1,
        SpatialArithmeticOperationV2::AabbMaxY,
    );
}

#[test]
fn each_record_checks_min_x_min_y_max_x_then_max_y() {
    let cases = [
        (
            super::validated_shape_support::rect_values(0, 1, -2 * SCALE, -2 * SCALE, SCALE, SCALE),
            SpatialArithmeticOperationV2::AabbMinX,
        ),
        (
            super::validated_shape_support::rect_values(0, 1, SCALE, -2 * SCALE, SCALE, SCALE),
            SpatialArithmeticOperationV2::AabbMinY,
        ),
        (
            super::validated_shape_support::rect_values(0, 1, SCALE, SCALE, SCALE, SCALE),
            SpatialArithmeticOperationV2::AabbMaxX,
        ),
    ];
    for (shape, expected) in cases {
        let fixture = fixture_with(
            vec![
                root(),
                owner_node(1, transform([MAXIMUM, 0, 0, MAXIMUM, 0, 0], 0, 0), 1, 1),
            ],
            vec![shape],
            Vec::new(),
            vec![fill(1, 0, 0, None)],
            Vec::new(),
            Vec::new(),
        );
        let engine = ScriptedLayoutEngine::new(Vec::new());
        expect_aabb_error(
            prepare_world_aabbs!(&fixture, VIEWPORT, limits(), &engine),
            1,
            expected,
        );
    }
}

fn projection_pair(
    paints: Vec<crate::paint::SpatialPaintV2>,
    hits: Vec<crate::content_item::SpatialHitV2>,
    semantics: Vec<crate::content_item::SpatialSemanticGeometryV2>,
    clips: Vec<crate::coverage::SpatialClipV2>,
) -> super::fixture::RawInputFixture {
    fixture_with(
        vec![
            root(),
            owner_node(
                1,
                projection_transform(SpatialArithmeticOperationV2::AabbMaxY),
                1,
                1,
            ),
            owner_node(
                2,
                projection_transform(SpatialArithmeticOperationV2::AabbMinX),
                1,
                1,
            ),
        ],
        vec![
            fault_shape(0, 1, SpatialArithmeticOperationV2::AabbMaxY),
            fault_shape(1, 2, SpatialArithmeticOperationV2::AabbMinX),
        ],
        clips,
        paints,
        hits,
        semantics,
    )
}

fn geometry_before_clip() -> super::fixture::RawInputFixture {
    fixture_with(
        vec![
            root(),
            owner_node(
                1,
                projection_transform(SpatialArithmeticOperationV2::AabbMinX),
                1,
                1,
            ),
            owner_node(
                2,
                projection_transform(SpatialArithmeticOperationV2::AabbMaxY),
                1,
                2,
            ),
        ],
        vec![fault_shape(0, 1, SpatialArithmeticOperationV2::AabbMinX)],
        vec![root_clip(0, 1, 0)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}

fn clip_before_paint() -> super::fixture::RawInputFixture {
    cross_table_pair(
        vec![fill(1, 0, 1, None)],
        Vec::new(),
        Vec::new(),
        vec![root_clip(0, 2, 0)],
    )
}

fn paint_before_hit() -> super::fixture::RawInputFixture {
    cross_table_pair(
        vec![fill(2, 0, 0, None)],
        vec![hit(1, 0, 1, None)],
        Vec::new(),
        Vec::new(),
    )
}

fn hit_before_semantic() -> super::fixture::RawInputFixture {
    cross_table_pair(
        Vec::new(),
        vec![hit(2, 0, 0, None)],
        vec![semantic_fill(1, 0, 1, None)],
        Vec::new(),
    )
}

fn cross_table_pair(
    paints: Vec<crate::paint::SpatialPaintV2>,
    hits: Vec<crate::content_item::SpatialHitV2>,
    semantics: Vec<crate::content_item::SpatialSemanticGeometryV2>,
    clips: Vec<crate::coverage::SpatialClipV2>,
) -> super::fixture::RawInputFixture {
    fixture_with(
        vec![
            root(),
            owner_node(
                1,
                projection_transform(SpatialArithmeticOperationV2::AabbMinX),
                1,
                1,
            ),
            owner_node(
                2,
                projection_transform(SpatialArithmeticOperationV2::AabbMaxY),
                1,
                1,
            ),
        ],
        vec![
            fault_shape(0, 2, SpatialArithmeticOperationV2::AabbMaxY),
            fault_shape(1, 1, SpatialArithmeticOperationV2::AabbMinX),
        ],
        clips,
        paints,
        hits,
        semantics,
    )
}
