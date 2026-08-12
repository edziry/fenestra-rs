use crate::baseline::literal_types::{
    AnchorComponentV2, AnchorTargetV2, CoverageInputV2, FIXED_ONE_V2, PaintContentInputV2,
    PlacementInputV2, SceneInputV2, ShapeGeometryInputV2,
};

use super::numeric::{IDENTITY, compose, placed, transform_aabb};
use super::path::flatten;
use super::types::{
    Aabb, Base, ResolvedClip, ResolvedHit, ResolvedPaint, ResolvedSemantic, ScenePlan, ShapePlan,
};

pub(super) fn prepare(scene: &SceneInputV2) -> ScenePlan<'_> {
    let bases = resolve_bases(scene);
    let worlds = resolve_worlds(scene, &bases);
    let paths = scene.paths.iter().map(flatten).collect();
    let shapes = scene
        .shapes
        .iter()
        .map(|value| shape_plan(scene, value))
        .collect::<Vec<_>>();
    let mut clips: Vec<ResolvedClip> = Vec::new();
    for input in &scene.clips {
        let primitive = transform_aabb(
            worlds[input.owner as usize],
            shapes[input.shape as usize].fill,
        );
        let effective = input.parent.map_or(primitive, |parent| {
            primitive.intersection(clips[parent as usize].effective)
        });
        clips.push(ResolvedClip {
            primitive,
            effective,
        });
    }
    let paints = scene
        .paints
        .iter()
        .map(|input| {
            let local_bounds = paint_bounds(&shapes, &input.content);
            ResolvedPaint {
                input: &input.content,
                local_bounds,
                world_bounds: transform_aabb(worlds[input.owner as usize], local_bounds),
            }
        })
        .collect();
    let hits = scene
        .hits
        .iter()
        .map(|input| {
            let local_bounds = coverage_bounds(&shapes, input.coverage);
            ResolvedHit {
                coverage: input.coverage,
                local_bounds,
                world_bounds: transform_aabb(worlds[input.owner as usize], local_bounds),
            }
        })
        .collect();
    let semantics = scene
        .semantics
        .iter()
        .map(|input| ResolvedSemantic {
            shape: input.shape,
            rule: input.rule,
            world_bounds: transform_aabb(
                worlds[input.owner as usize],
                shapes[input.shape as usize].fill,
            ),
        })
        .collect();
    ScenePlan {
        scene,
        bases,
        worlds,
        paths,
        shapes,
        clips,
        paints,
        hits,
        semantics,
    }
}

fn resolve_bases(scene: &SceneInputV2) -> Vec<Base> {
    let count = scene.nodes.len();
    let mut output = vec![None; count];
    output[0] = Some(Base {
        x: 0,
        y: 0,
        width: scene.viewport.0 as i32,
        height: scene.viewport.1 as i32,
    });
    while output.iter().any(Option::is_none) {
        let before = output.iter().filter(|item| item.is_some()).count();
        for index in 1..count {
            if output[index].is_some() {
                continue;
            }
            let node = &scene.nodes[index];
            let parent_key = node.parent.expect("registered nonroot has a parent") as usize;
            let Some(parent) = output[parent_key] else {
                continue;
            };
            output[index] = match node.placement {
                PlacementInputV2::Layout { width, height, .. } => {
                    layout_base(scene, &output, index, parent, width, height)
                }
                PlacementInputV2::Free {
                    width,
                    height,
                    self_anchor,
                    target,
                    target_anchor,
                    offset,
                    ..
                } => {
                    let target_key = match target {
                        AnchorTargetV2::Viewport => 0,
                        AnchorTargetV2::Parent => parent_key,
                        AnchorTargetV2::Node(key) => key as usize,
                    };
                    output[target_key].map(|target| Base {
                        x: anchor(target.x, target.width, target_anchor[0]) + offset.x
                            - anchor(0, width, self_anchor[0]),
                        y: anchor(target.y, target.height, target_anchor[1]) + offset.y
                            - anchor(0, height, self_anchor[1]),
                        width,
                        height,
                    })
                }
                PlacementInputV2::Root => None,
            };
        }
        assert!(
            output.iter().filter(|item| item.is_some()).count() > before,
            "registered dependency graph should advance"
        );
    }
    output.into_iter().map(Option::unwrap).collect()
}

fn layout_base(
    scene: &SceneInputV2,
    resolved: &[Option<Base>],
    index: usize,
    parent: Base,
    width: i32,
    height: i32,
) -> Option<Base> {
    let node = &scene.nodes[index];
    let parent_node = &scene.nodes[node.parent? as usize];
    let mut cursor = match parent_node.axis {
        crate::baseline::literal_types::AxisV2::Horizontal => {
            parent.x + i64::from(parent_node.padding[0]) * FIXED_ONE_V2
        }
        crate::baseline::literal_types::AxisV2::Vertical => {
            parent.y + i64::from(parent_node.padding[2]) * FIXED_ONE_V2
        }
    };
    for sibling in &scene.nodes[1..index] {
        if sibling.parent != node.parent
            || !matches!(sibling.placement, PlacementInputV2::Layout { .. })
        {
            continue;
        }
        let value = resolved[sibling.key as usize]?;
        cursor = match parent_node.axis {
            crate::baseline::literal_types::AxisV2::Horizontal => {
                value.x + i64::from(value.width + parent_node.gap) * FIXED_ONE_V2
            }
            crate::baseline::literal_types::AxisV2::Vertical => {
                value.y + i64::from(value.height + parent_node.gap) * FIXED_ONE_V2
            }
        };
    }
    let content_x = parent.x + i64::from(parent_node.padding[0]) * FIXED_ONE_V2;
    let content_y = parent.y + i64::from(parent_node.padding[2]) * FIXED_ONE_V2;
    Some(match parent_node.axis {
        crate::baseline::literal_types::AxisV2::Horizontal => Base {
            x: cursor,
            y: content_y,
            width,
            height,
        },
        crate::baseline::literal_types::AxisV2::Vertical => Base {
            x: content_x,
            y: cursor,
            width,
            height,
        },
    })
}

fn resolve_worlds(scene: &SceneInputV2, bases: &[Base]) -> Vec<super::types::Affine> {
    let mut output = vec![IDENTITY];
    for index in 1..scene.nodes.len() {
        let node = &scene.nodes[index];
        let parent = node.parent.expect("registered nonroot parent") as usize;
        let transform = match node.placement {
            PlacementInputV2::Layout { transform, .. }
            | PlacementInputV2::Free { transform, .. } => transform,
            PlacementInputV2::Root => unreachable!(),
        };
        let local_x = bases[index].x - bases[parent].x;
        let local_y = bases[index].y - bases[parent].y;
        output.push(compose(output[parent], placed(local_x, local_y, transform)));
    }
    output
}

fn shape_plan(
    scene: &SceneInputV2,
    value: &crate::baseline::literal_types::ShapeInputV2,
) -> ShapePlan {
    let (base, empty_fill) = match &value.geometry {
        ShapeGeometryInputV2::Rect(value) => (
            Aabb::closed([
                value.x,
                value.y,
                value.x + value.width,
                value.y + value.height,
            ]),
            value.width == 0 || value.height == 0,
        ),
        ShapeGeometryInputV2::Circle { center, radius } => (
            Aabb::closed([
                center.x - radius,
                center.y - radius,
                center.x + radius,
                center.y + radius,
            ]),
            *radius == 0,
        ),
        ShapeGeometryInputV2::Polygon { points } => (point_bounds(points.iter().copied()), false),
        ShapeGeometryInputV2::Path { path } => {
            let points = scene.paths[*path as usize]
                .verbs
                .iter()
                .flat_map(|verb| match *verb {
                    crate::baseline::literal_types::PathVerbInputV2::Move(point)
                    | crate::baseline::literal_types::PathVerbInputV2::Line(point) => vec![point],
                    crate::baseline::literal_types::PathVerbInputV2::Quadratic(first, second) => {
                        vec![first, second]
                    }
                    crate::baseline::literal_types::PathVerbInputV2::Cubic(
                        first,
                        second,
                        third,
                    ) => vec![first, second, third],
                    crate::baseline::literal_types::PathVerbInputV2::Close => Vec::new(),
                });
            (point_bounds(points), false)
        }
    };
    ShapePlan {
        base,
        fill: if empty_fill { Aabb::EMPTY } else { base },
    }
}

fn paint_bounds(shapes: &[ShapePlan], value: &PaintContentInputV2) -> Aabb {
    match value {
        PaintContentInputV2::Coverage { coverage, .. } => coverage_bounds(shapes, *coverage),
        PaintContentInputV2::Image { destination, .. } => Aabb::closed([
            destination.x,
            destination.y,
            destination.x + destination.width,
            destination.y + destination.height,
        ]),
    }
}

fn coverage_bounds(shapes: &[ShapePlan], value: CoverageInputV2) -> Aabb {
    match value {
        CoverageInputV2::Fill { shape, .. } => shapes[shape as usize].fill,
        CoverageInputV2::RoundStroke { shape, width } => {
            let base = shapes[shape as usize].base;
            let expansion = (width + 1) / 2;
            Aabb::closed([
                base.edges[0] - expansion,
                base.edges[1] - expansion,
                base.edges[2] + expansion,
                base.edges[3] + expansion,
            ])
        }
    }
}

fn point_bounds(points: impl Iterator<Item = crate::baseline::literal_types::PointV2>) -> Aabb {
    let mut points = points;
    let first = points.next().expect("registered bounded shape has a point");
    let mut edges = [first.x, first.y, first.x, first.y];
    for point in points {
        edges = [
            edges[0].min(point.x),
            edges[1].min(point.y),
            edges[2].max(point.x),
            edges[3].max(point.y),
        ];
    }
    Aabb::closed(edges)
}

fn anchor(start: i64, extent: i32, component: AnchorComponentV2) -> i64 {
    match component {
        AnchorComponentV2::Start => start,
        AnchorComponentV2::Center => start + i64::from(extent) * FIXED_ONE_V2 / 2,
        AnchorComponentV2::End => start + i64::from(extent) * FIXED_ONE_V2,
    }
}
