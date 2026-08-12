use super::numeric::{SCALE, determinant, intersect, project};
use super::scene::{Coverage, PaintContent, Scene, Shape};
use super::types::{Aabb, Clip, Geometry, Item, Paint, PaintReference, Projection};

pub fn build(scene: &Scene) -> Projection {
    let mapping = scene
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (ordinal(index), node.path.clone()))
        .collect();
    let geometry = scene
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| Geometry {
            key: ordinal(index),
            path: node.path.clone(),
            base: node.base,
            affine: node.world,
            determinant: determinant(node.world),
            aabb: project(node.world, Aabb::new([0, 0, node.base[2], node.base[3]])),
        })
        .collect();
    let mut effective = Vec::with_capacity(scene.clips.len());
    let clips = scene
        .clips
        .iter()
        .enumerate()
        .map(|(index, clip)| {
            let node = &scene.nodes[usize::try_from(clip.owner).expect("owner should fit")];
            let primitive = project(node.world, shape_bounds(scene, clip.shape));
            let bound = clip.parent.map_or(primitive, |parent| {
                intersect(
                    primitive,
                    effective[usize::try_from(parent).expect("parent clip should fit")],
                )
            });
            effective.push(bound);
            Clip {
                key: ordinal(index),
                owner: clip.owner,
                path: owner_path(scene, clip.owner),
                parent: clip.parent,
                shape: clip.shape,
                affine: node.world,
                determinant: determinant(node.world),
                primitive,
                effective: bound,
            }
        })
        .collect();
    let paints = scene
        .paints
        .iter()
        .enumerate()
        .map(|(index, paint)| {
            let node = &scene.nodes[usize::try_from(paint.owner).expect("owner should fit")];
            let (reference, clip) = match paint.content {
                PaintContent::Coverage {
                    coverage,
                    brush,
                    clip,
                    ..
                } => (
                    PaintReference::Coverage {
                        shape: coverage_shape(coverage),
                        brush,
                    },
                    clip,
                ),
                PaintContent::Image { clip } => (PaintReference::Image { image: 0 }, clip),
            };
            Paint {
                key: ordinal(index),
                owner: paint.owner,
                path: owner_path(scene, paint.owner),
                affine: node.world,
                determinant: determinant(node.world),
                aabb: project(node.world, paint.bounds),
                reference,
                clip,
                stack: paint.owner,
                item: paint.item,
            }
        })
        .collect();
    let hits = scene
        .hits
        .iter()
        .enumerate()
        .map(|(index, hit)| {
            item(
                scene,
                index,
                hit.owner,
                hit.item,
                hit.bounds,
                coverage_shape(hit.coverage),
                hit.clip,
            )
        })
        .collect();
    let semantics = scene
        .semantics
        .iter()
        .enumerate()
        .map(|(index, semantic)| {
            item(
                scene,
                index,
                semantic.owner,
                semantic.item,
                semantic.bounds,
                semantic.shape,
                semantic.clip,
            )
        })
        .collect();
    Projection {
        mapping,
        geometry,
        clips,
        paints,
        hits,
        semantics,
    }
}

#[allow(clippy::too_many_arguments)]
fn item(
    scene: &Scene,
    index: usize,
    owner: u32,
    item: u32,
    bounds: Aabb,
    shape: u32,
    clip: Option<u32>,
) -> Item {
    let node = &scene.nodes[usize::try_from(owner).expect("owner should fit")];
    Item {
        key: ordinal(index),
        owner,
        path: owner_path(scene, owner),
        affine: node.world,
        determinant: determinant(node.world),
        aabb: project(node.world, bounds),
        shape,
        clip,
        stack: owner,
        item,
    }
}

pub fn shape_bounds(scene: &Scene, shape: u32) -> Aabb {
    match &scene.shapes[usize::try_from(shape).expect("shape should fit")] {
        Shape::Rect {
            origin,
            width,
            height,
        } => Aabb::new([origin[0], origin[1], origin[0] + width, origin[1] + height]),
        Shape::Circle { center, radius } => Aabb::new([
            center[0] - radius,
            center[1] - radius,
            center[0] + radius,
            center[1] + radius,
        ]),
        Shape::Polygon(points) => points.iter().fold(
            Aabb::new([i64::MAX, i64::MAX, i64::MIN, i64::MIN]),
            |bounds, point| {
                Aabb::new([
                    bounds.edges[0].min(point[0]),
                    bounds.edges[1].min(point[1]),
                    bounds.edges[2].max(point[0]),
                    bounds.edges[3].max(point[1]),
                ])
            },
        ),
        Shape::Path => Aabb::new([0, 0, 15 * SCALE, 15 * SCALE]),
    }
}

fn coverage_shape(coverage: Coverage) -> u32 {
    match coverage {
        Coverage::Fill { shape, .. } | Coverage::Stroke { shape, .. } => shape,
    }
}

fn owner_path(scene: &Scene, owner: u32) -> String {
    scene.nodes[usize::try_from(owner).expect("owner should fit")]
        .path
        .clone()
        .expect("an item owner should not be the sentinel")
}

fn ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("fixture ordinal should fit")
}
