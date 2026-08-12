use std::error::Error;

use super::fixture::RawInputFixture;
use super::prepared_brush_support::{color, solid_color};
use super::validated_clip_support::root_clip;
use super::validated_hit_support::fill as hit_fill;
use super::validated_paint_support::fill as paint_fill;
use super::validated_semantic_support::semantic;
use super::validated_shape_support::rect_values;
pub(super) use super::world_transform_support::{
    MAXIMUM, SCALE, ScriptedLayoutEngine, VIEWPORT, expect_valid, free, identity, limits, root,
    scalar, transform,
};
use crate::aabb::SpatialAabbV2;
use crate::content_item::{SpatialHitV2, SpatialInputPolicyV2, SpatialSemanticGeometryV2};
use crate::coverage::{SpatialClipV2, SpatialFillRuleV2};
use crate::error::SpatialErrorLocationV2;
use crate::model::SpatialAnchorTargetV2;
use crate::numeric_error::SpatialArithmeticOperationV2;
use crate::paint::SpatialPaintV2;
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};
use crate::shape::SpatialShapeV2;
use crate::topology::SpatialNodeV2;

#[derive(Clone, Copy)]
pub(super) enum ProjectionTable {
    Clip,
    Paint,
    Hit,
    Semantic,
}

pub(super) type AabbFact = (u32, SpatialAabbV2);

pub(super) fn fixture_with(
    nodes: Vec<SpatialNodeV2>,
    shapes: Vec<SpatialShapeV2>,
    clips: Vec<SpatialClipV2>,
    paints: Vec<SpatialPaintV2>,
    hits: Vec<SpatialHitV2>,
    semantics: Vec<SpatialSemanticGeometryV2>,
) -> RawInputFixture {
    super::world_transform_support::fixture(nodes)
        .with_paths(Vec::new(), Vec::new())
        .with_shapes(shapes, Vec::new())
        .with_brushes(vec![solid_color(0, color(10, 20, 30, 255))], Vec::new())
        .with_images(Vec::new())
        .with_clips(clips)
        .with_paint_items(paints)
        .with_hit_items(hits)
        .with_semantic_items(semantics)
}

pub(super) fn projection_fault_fixture(
    table: ProjectionTable,
    operation: SpatialArithmeticOperationV2,
) -> RawInputFixture {
    let nodes = vec![
        root(),
        owner_node(1, identity(), 1, 1),
        owner_node(2, projection_transform(operation), 1, 1),
    ];
    let shapes = vec![valid_shape(0, 1), fault_shape(1, 2, operation)];
    let mut clips = Vec::new();
    let mut paints = Vec::new();
    let mut hits = Vec::new();
    let mut semantics = Vec::new();
    match table {
        ProjectionTable::Clip => {
            clips.push(root_clip(0, 1, 0));
            clips.push(root_clip(1, 2, 1));
        }
        ProjectionTable::Paint => {
            paints.push(fill(1, 0, 0, None));
            paints.push(fill(2, 0, 1, None));
        }
        ProjectionTable::Hit => {
            hits.push(hit(1, 0, 0, None));
            hits.push(hit(2, 0, 1, None));
        }
        ProjectionTable::Semantic => {
            semantics.push(semantic_fill(1, 0, 0, None));
            semantics.push(semantic_fill(2, 0, 1, None));
        }
    }
    fixture_with(nodes, shapes, clips, paints, hits, semantics)
}

pub(super) fn geometry_fault_fixture(operation: SpatialArithmeticOperationV2) -> RawInputFixture {
    let (matrix, width, height) = match operation {
        SpatialArithmeticOperationV2::AabbMinX => ([-MAXIMUM, 0, 0, SCALE, 0, 0], 2, 1),
        SpatialArithmeticOperationV2::AabbMinY => ([SCALE, 0, 0, -MAXIMUM, 0, 0], 1, 2),
        SpatialArithmeticOperationV2::AabbMaxX => ([MAXIMUM, 0, 0, SCALE, 0, 0], 2, 1),
        SpatialArithmeticOperationV2::AabbMaxY => ([SCALE, 0, 0, MAXIMUM, 0, 0], 1, 2),
        _ => panic!("world-AABB fixtures require an AABB operation"),
    };
    fixture_with(
        vec![
            root(),
            owner_node(1, transform(matrix, 0, 0), width, height),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}

pub(super) fn owner_node(
    key: u32,
    local: crate::model::SpatialLocalTransformV2,
    width: i32,
    height: i32,
) -> SpatialNodeV2 {
    free(
        key,
        0,
        SpatialAnchorTargetV2::Viewport,
        0,
        0,
        width,
        height,
        local,
    )
}

pub(super) fn projection_transform(
    operation: SpatialArithmeticOperationV2,
) -> crate::model::SpatialLocalTransformV2 {
    match operation {
        SpatialArithmeticOperationV2::AabbMinX | SpatialArithmeticOperationV2::AabbMaxX => {
            transform([MAXIMUM, 0, 0, SCALE, 0, 0], 0, 0)
        }
        SpatialArithmeticOperationV2::AabbMinY | SpatialArithmeticOperationV2::AabbMaxY => {
            transform([SCALE, 0, 0, MAXIMUM, 0, 0], 0, 0)
        }
        _ => panic!("world-AABB fixtures require an AABB operation"),
    }
}

pub(super) fn valid_shape(key: u32, owner: u32) -> SpatialShapeV2 {
    rect_values(key, owner, 0, 0, SCALE, SCALE)
}

pub(super) fn fault_shape(
    key: u32,
    owner: u32,
    operation: SpatialArithmeticOperationV2,
) -> SpatialShapeV2 {
    match operation {
        SpatialArithmeticOperationV2::AabbMinX => {
            rect_values(key, owner, -2 * SCALE, 0, SCALE, SCALE)
        }
        SpatialArithmeticOperationV2::AabbMinY => {
            rect_values(key, owner, 0, -2 * SCALE, SCALE, SCALE)
        }
        SpatialArithmeticOperationV2::AabbMaxX => rect_values(key, owner, SCALE, 0, SCALE, SCALE),
        SpatialArithmeticOperationV2::AabbMaxY => rect_values(key, owner, 0, SCALE, SCALE, SCALE),
        _ => panic!("world-AABB fixtures require an AABB operation"),
    }
}

pub(super) fn fill(owner: u32, ordinal: u32, shape: u32, clip: Option<u32>) -> SpatialPaintV2 {
    paint_fill(owner, ordinal, shape, 0, clip, SpatialFillRuleV2::NonZero)
}

pub(super) fn hit(owner: u32, ordinal: u32, shape: u32, clip: Option<u32>) -> SpatialHitV2 {
    hit_fill(
        owner,
        ordinal,
        shape,
        clip,
        SpatialFillRuleV2::EvenOdd,
        SpatialInputPolicyV2::Accept,
    )
}

pub(super) const fn semantic_fill(
    owner: u32,
    ordinal: u32,
    shape: u32,
    clip: Option<u32>,
) -> SpatialSemanticGeometryV2 {
    semantic(owner, ordinal, shape, SpatialFillRuleV2::NonZero, clip)
}

pub(super) fn aabb(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> SpatialAabbV2 {
    SpatialAabbV2::from_edges(scalar(min_x), scalar(min_y), scalar(max_x), scalar(max_y))
        .expect("test bounds are canonical and ordered")
}

pub(super) const fn fact(index: u32, bounds: SpatialAabbV2) -> AabbFact {
    (index, bounds)
}

pub(super) fn expect_aabb_error<T>(
    result: Result<T, SpatialResolveErrorV2>,
    owner: u32,
    operation: SpatialArithmeticOperationV2,
) {
    let error = match result {
        Ok(_) => panic!("expected world-AABB failure"),
        Err(error) => error,
    };
    assert_eq!(
        error.kind(),
        SpatialResolveErrorKindV2::Arithmetic(operation)
    );
    assert_eq!(
        error.location(),
        SpatialErrorLocationV2::Node { index: owner }
    );
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
    assert_eq!(error.to_string(), "spatial-resolve-error(arithmetic)");
    assert_eq!(
        format!("{error:?}"),
        "SpatialResolveErrorV2(spatial-resolve-error(arithmetic))"
    );
    assert!(Error::source(&error).is_none());
}

pub(super) fn empty() -> SpatialAabbV2 {
    SpatialAabbV2::empty()
}
