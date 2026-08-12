use crate::baseline::literal_types::{PaintContentInputV2, SceneInputV2};

use super::super::model_projection::{
    NormalizedAabbV2, NormalizedClipV2, NormalizedGeometryV2, NormalizedHitV2, NormalizedItemV2,
    NormalizedPaintReferenceV2, NormalizedPaintV2, NormalizedProjectionV2, NormalizedQueryV2,
    NormalizedRasterV2,
};
use super::coverage::hit_test;
use super::paint::raster;
use super::types::{Aabb, ScenePlan};

pub(super) fn normalize(plan: &ScenePlan<'_>) -> NormalizedProjectionV2 {
    let scene = plan.scene;
    NormalizedProjectionV2 {
        mapping: mapping(scene),
        geometry: scene
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| NormalizedGeometryV2 {
                key: node.key,
                path: node.path.clone(),
                base: [
                    plan.bases[index].x,
                    plan.bases[index].y,
                    i64::from(plan.bases[index].width) * 65_536,
                    i64::from(plan.bases[index].height) * 65_536,
                ],
                affine: plan.worlds[index].values,
                determinant: super::numeric::determinant(plan.worlds[index]),
                aabb: aabb(super::numeric::transform_aabb(
                    plan.worlds[index],
                    Aabb::closed([
                        0,
                        0,
                        i64::from(plan.bases[index].width) * 65_536,
                        i64::from(plan.bases[index].height) * 65_536,
                    ]),
                )),
            })
            .collect(),
        clips: scene
            .clips
            .iter()
            .enumerate()
            .map(|(index, input)| NormalizedClipV2 {
                key: input.key,
                owner: input.owner,
                path: path(scene, input.owner),
                parent: input.parent,
                shape: input.shape,
                affine: plan.worlds[input.owner as usize].values,
                determinant: super::numeric::determinant(plan.worlds[input.owner as usize]),
                primitive: aabb(plan.clips[index].primitive),
                effective: aabb(plan.clips[index].effective),
            })
            .collect(),
        paints: scene
            .paints
            .iter()
            .enumerate()
            .map(|(index, input)| {
                let reference = match input.content {
                    PaintContentInputV2::Coverage {
                        coverage, brush, ..
                    } => NormalizedPaintReferenceV2::Coverage {
                        shape: coverage.shape(),
                        brush,
                    },
                    PaintContentInputV2::Image { image, .. } => {
                        NormalizedPaintReferenceV2::Image { image }
                    }
                };
                NormalizedPaintV2 {
                    key: index as u32,
                    owner: input.owner,
                    path: path(scene, input.owner),
                    affine: plan.worlds[input.owner as usize].values,
                    determinant: super::numeric::determinant(plan.worlds[input.owner as usize]),
                    aabb: aabb(plan.paints[index].world_bounds),
                    reference,
                    clip: paint_clip(&input.content),
                    stack: input.owner,
                    item: input.item,
                }
            })
            .collect(),
        hits: scene
            .hits
            .iter()
            .enumerate()
            .map(|(index, input)| {
                item(
                    scene,
                    index,
                    input.owner,
                    input.item,
                    input.coverage.shape(),
                    input.clip,
                    plan,
                    plan.hits[index].world_bounds,
                )
            })
            .collect(),
        semantics: scene
            .semantics
            .iter()
            .enumerate()
            .map(|(index, input)| {
                item(
                    scene,
                    index,
                    input.owner,
                    input.item,
                    input.shape,
                    input.clip,
                    plan,
                    plan.semantics[index].world_bounds,
                )
            })
            .collect(),
        queries: scene
            .queries
            .iter()
            .copied()
            .map(|scene_point| NormalizedQueryV2 {
                scene: scene_point,
                result: hit_test(plan, scene_point).map(|(key, owner, item, local)| {
                    NormalizedHitV2 {
                        key,
                        owner,
                        path: path(scene, owner),
                        item,
                        local,
                    }
                }),
            })
            .collect(),
        raster: NormalizedRasterV2 {
            width: scene.viewport.0,
            height: scene.viewport.1,
            stride: u64::from(scene.viewport.0) * 4,
            bytes: raster(plan),
        },
    }
}

fn mapping(scene: &SceneInputV2) -> Vec<(u32, Option<String>)> {
    if scene.nodes.iter().all(|node| node.path.is_none()) {
        Vec::new()
    } else {
        scene
            .nodes
            .iter()
            .map(|node| (node.key, node.path.clone()))
            .collect()
    }
}

#[allow(clippy::too_many_arguments)]
fn item(
    scene: &SceneInputV2,
    key: usize,
    owner: u32,
    item: u32,
    shape: u32,
    clip: Option<u32>,
    plan: &ScenePlan<'_>,
    bounds: Aabb,
) -> NormalizedItemV2 {
    NormalizedItemV2 {
        key: key as u32,
        owner,
        path: path(scene, owner),
        affine: plan.worlds[owner as usize].values,
        determinant: super::numeric::determinant(plan.worlds[owner as usize]),
        aabb: aabb(bounds),
        shape,
        clip,
        stack: owner,
        item,
    }
}

fn aabb(value: Aabb) -> NormalizedAabbV2 {
    NormalizedAabbV2 {
        empty: value.empty,
        min_x: value.edges[0],
        min_y: value.edges[1],
        max_x: value.edges[2],
        max_y: value.edges[3],
    }
}

fn path(scene: &SceneInputV2, owner: u32) -> Option<String> {
    scene.nodes[owner as usize].path.clone()
}

fn paint_clip(value: &PaintContentInputV2) -> Option<u32> {
    match value {
        PaintContentInputV2::Coverage { clip, .. } | PaintContentInputV2::Image { clip, .. } => {
            *clip
        }
    }
}
