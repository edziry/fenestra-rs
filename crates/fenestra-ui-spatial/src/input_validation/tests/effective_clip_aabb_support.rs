use super::dependency_support::{
    free as dependency_free, layout as dependency_layout, root as dependency_root,
};
use super::fixture::RawInputFixture;
use super::flattened_path_support::{line_to, move_to, path, point};
use super::prepared_brush_support::{color, gradient, solid_color, stop_color, valid_stops};
pub(super) use super::validated_clip_support::clip;
use super::validated_clip_support::root_clip;
use super::validated_image_support::blank_image;
pub(super) use super::validated_shape_support::rect_values;
use super::validated_shape_support::{circle_values, path_shape, polygon};
pub(super) use super::world_aabb_support::{
    SCALE, ScriptedLayoutEngine, VIEWPORT, aabb, empty, expect_valid, fact, fill, fixture_with,
    free, hit, identity, limits, owner_node, root, semantic_fill,
};
use super::world_transform_support::{fixture, logical};
use crate::content_item::{SpatialHitV2, SpatialSemanticGeometryV2};
use crate::coverage::{SpatialClipV2, SpatialFillRuleV2};
use crate::model::SpatialAnchorTargetV2;
use crate::paint::SpatialPaintV2;
use crate::shape::SpatialShapeV2;

pub(super) fn clips_only_fixture(
    shapes: Vec<SpatialShapeV2>,
    clips: Vec<SpatialClipV2>,
) -> RawInputFixture {
    fixture_with(
        vec![root(), owner_node(1, identity(), 20, 20)],
        shapes,
        clips,
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}

pub(super) fn retained_phase_ten_fixture(
    paints: Vec<SpatialPaintV2>,
    hits: Vec<SpatialHitV2>,
    semantics: Vec<SpatialSemanticGeometryV2>,
) -> RawInputFixture {
    retained_fixture(paints, hits, semantics, false)
}

pub(super) fn retained_prepared_fixture(
    paints: Vec<SpatialPaintV2>,
    hits: Vec<SpatialHitV2>,
    semantics: Vec<SpatialSemanticGeometryV2>,
) -> RawInputFixture {
    retained_fixture(paints, hits, semantics, true)
}

fn retained_fixture(
    paints: Vec<SpatialPaintV2>,
    hits: Vec<SpatialHitV2>,
    semantics: Vec<SpatialSemanticGeometryV2>,
    extended_clips: bool,
) -> RawInputFixture {
    let layout = if extended_clips {
        super::placement_execution_support::layout(2, 1, 3, 4)
    } else {
        dependency_layout(2, 1)
    };
    let (paths, verbs, shape_path) = if extended_clips {
        (
            vec![path(0, 0, 2), path(1, 2, 2)],
            vec![
                move_to(logical(5), logical(5)),
                line_to(logical(6), logical(5)),
                move_to(0, 0),
                line_to(logical(2), 0),
            ],
            1,
        )
    } else {
        (
            vec![path(0, 0, 2)],
            vec![move_to(0, 0), line_to(logical(2), 0)],
            0,
        )
    };
    let mut shapes = vec![
        rect_values(0, 1, logical(1), logical(2), logical(3), logical(4)),
        polygon(1, 2, 0, 3),
        path_shape(2, 3, shape_path),
    ];
    let mut clips = vec![root_clip(0, 1, 0)];
    if extended_clips {
        shapes.push(circle_values(3, 1, logical(5), logical(5), logical(5)));
        shapes.push(polygon(4, 1, 3, 3));
        clips.push(clip(1, 1, Some(0), 3, SpatialFillRuleV2::EvenOdd));
        clips.push(clip(2, 1, Some(1), 3, SpatialFillRuleV2::NonZero));
    }
    let (brushes, stops) = if extended_clips {
        (
            vec![
                solid_color(0, color(17, 89, 203, 137)),
                gradient(1, 0, 2),
                gradient(2, 2, 2),
            ],
            vec![
                stop_color(0, color(255, 0, 0, 128)),
                stop_color(u16::MAX, color(0, 0, 255, 64)),
                stop_color(0, color(10, 20, 30, 255)),
                stop_color(u16::MAX, color(40, 50, 60, 255)),
            ],
        )
    } else {
        (
            vec![solid_color(0, color(10, 20, 30, 255)), gradient(1, 0, 2)],
            valid_stops(),
        )
    };

    fixture(vec![
        dependency_root(),
        dependency_free(1, 0, SpatialAnchorTargetV2::Viewport),
        layout,
        dependency_free(3, 2, SpatialAnchorTargetV2::Parent),
    ])
    .with_paths(paths, verbs)
    .with_shapes(
        shapes,
        vec![
            point(-logical(2), logical(3)),
            point(logical(4), -logical(1)),
            point(logical(1), logical(5)),
            point(logical(20), logical(20)),
            point(logical(22), logical(20)),
            point(logical(20), logical(22)),
        ],
    )
    .with_brushes(brushes, stops)
    .with_images(if extended_clips {
        vec![blank_image(0, 1, 1), blank_image(1, 2, 1)]
    } else {
        vec![blank_image(0, 1, 1)]
    })
    .with_clips(clips)
    .with_paint_items(paints)
    .with_hit_items(hits)
    .with_semantic_items(semantics)
}

pub(super) const fn nonzero_clip(
    key: u32,
    owner: u32,
    parent: Option<u32>,
    shape: u32,
) -> SpatialClipV2 {
    clip(key, owner, parent, shape, SpatialFillRuleV2::NonZero)
}

pub(super) const fn even_odd_clip(
    key: u32,
    owner: u32,
    parent: Option<u32>,
    shape: u32,
) -> SpatialClipV2 {
    clip(key, owner, parent, shape, SpatialFillRuleV2::EvenOdd)
}
