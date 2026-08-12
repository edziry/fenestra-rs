use super::model::{Model, member_path};
use super::numeric::{Affine, Point, SCALE, about, compose, fixed, identity, translation};
use super::types::Aabb;

#[derive(Clone)]
pub struct Scene {
    pub nodes: Vec<SpatialNode>,
    pub shapes: Vec<Shape>,
    pub brushes: Vec<Brush>,
    pub clips: Vec<ClipPlan>,
    pub paints: Vec<PaintPlan>,
    pub hits: Vec<HitPlan>,
    pub semantics: Vec<SemanticPlan>,
    pub path: Vec<Point>,
}

#[derive(Clone)]
pub struct SpatialNode {
    pub path: Option<String>,
    pub base: [i64; 4],
    pub world: Affine,
}

#[derive(Clone)]
pub enum Shape {
    Rect {
        origin: Point,
        width: i64,
        height: i64,
    },
    Circle {
        center: Point,
        radius: i64,
    },
    Polygon(Vec<Point>),
    Path,
}

#[derive(Clone)]
pub enum Brush {
    Solid([u8; 4]),
    Gradient {
        start: Point,
        end: Point,
        stops: Vec<(u16, [u8; 4])>,
    },
}

#[derive(Clone, Copy)]
pub enum Rule {
    NonZero,
    EvenOdd,
}

#[derive(Clone, Copy)]
pub enum Coverage {
    Fill { shape: u32, rule: Rule },
    Stroke { shape: u32, width: i64 },
}

#[derive(Clone)]
pub struct ClipPlan {
    pub owner: u32,
    pub parent: Option<u32>,
    pub shape: u32,
    pub rule: Rule,
}

#[derive(Clone)]
pub enum PaintContent {
    Coverage {
        coverage: Coverage,
        brush: u32,
        opacity: u8,
        clip: Option<u32>,
    },
    Image {
        clip: Option<u32>,
    },
}

#[derive(Clone)]
pub struct PaintPlan {
    pub owner: u32,
    pub item: u32,
    pub bounds: Aabb,
    pub content: PaintContent,
}

#[derive(Clone)]
pub struct HitPlan {
    pub owner: u32,
    pub item: u32,
    pub bounds: Aabb,
    pub coverage: Coverage,
    pub clip: Option<u32>,
    pub accepts: bool,
}

#[derive(Clone)]
pub struct SemanticPlan {
    pub owner: u32,
    pub item: u32,
    pub bounds: Aabb,
    pub shape: u32,
    pub clip: Option<u32>,
}

impl Scene {
    pub fn build(model: &Model) -> Self {
        let nodes = nodes(model);
        let path = super::scene_static::path();
        let mut shapes = super::scene_static::shapes(model);
        let mut brushes = super::scene_static::brushes(model);
        let mut clips = super::scene_static::clips();
        let mut paints = super::scene_static::paints(model);
        let mut hits = super::scene_static::hits(model);
        let mut semantics = super::scene_static::semantics();
        for (tile_index, _) in model.keys.iter().enumerate() {
            let owner = u32::try_from(5 + tile_index).expect("tile owner should fit");
            let shape = u32::try_from(shapes.len()).expect("shape key should fit");
            shapes.push(Shape::Circle {
                center: [8 * SCALE, 6 * SCALE],
                radius: 4 * SCALE,
            });
            let brush = u32::try_from(brushes.len()).expect("brush key should fit");
            brushes.push(Brush::Solid([80, 120, 160, 192]));
            let clip = u32::try_from(clips.len()).expect("clip key should fit");
            clips.push(ClipPlan {
                owner,
                parent: Some(0),
                shape,
                rule: Rule::NonZero,
            });
            paints.push(PaintPlan {
                owner,
                item: 0,
                bounds: Aabb::new([4 * SCALE, 2 * SCALE, 12 * SCALE, 10 * SCALE]),
                content: PaintContent::Coverage {
                    coverage: Coverage::Fill {
                        shape,
                        rule: Rule::NonZero,
                    },
                    brush,
                    opacity: 128,
                    clip: Some(clip),
                },
            });
            hits.push(HitPlan {
                owner,
                item: 0,
                bounds: Aabb::new([7 * SCALE / 2, 3 * SCALE / 2, 25 * SCALE / 2, 21 * SCALE / 2]),
                coverage: Coverage::Stroke {
                    shape,
                    width: SCALE,
                },
                clip: Some(0),
                accepts: true,
            });
            semantics.push(SemanticPlan {
                owner,
                item: 0,
                bounds: Aabb::new([4 * SCALE, 2 * SCALE, 12 * SCALE, 10 * SCALE]),
                shape,
                clip: Some(clip),
            });
        }
        Self {
            nodes,
            shapes,
            brushes,
            clips,
            paints,
            hits,
            semantics,
            path,
        }
    }
}

fn nodes(model: &Model) -> Vec<SpatialNode> {
    let mut nodes = Vec::new();
    nodes.push(SpatialNode {
        path: None,
        base: [0, 0, fixed(model.viewport[0]), fixed(model.viewport[1])],
        world: identity(),
    });
    push_node(
        &mut nodes,
        "root",
        [4, 3, model.span_x, 120],
        0,
        identity(),
        [0, 0],
    );
    push_node(
        &mut nodes,
        "root/s:0",
        [6, 5, 80, 60],
        1,
        translation(SCALE, SCALE / 2),
        [0, 0],
    );
    push_node(
        &mut nodes,
        "root/s:0/s:0",
        [-15, 58, 40, 30],
        2,
        [SCALE, 0, 0, SCALE, 0, 0],
        [2 * SCALE, 2 * SCALE],
    );
    push_node(
        &mut nodes,
        "root/s:0/s:0/s:0",
        [26, 87, 12, 10],
        3,
        [0, SCALE, -SCALE, 0, 0, 0],
        [6 * SCALE, 5 * SCALE],
    );
    let mut tile_y = 60;
    for (key, height) in &model.keys {
        push_node(
            &mut nodes,
            &member_path(*key),
            [-13, tile_y, 16, *height],
            3,
            translation(SCALE, -SCALE / 2),
            [0, 0],
        );
        tile_y += *height + 2;
    }
    push_node(
        &mut nodes,
        "root/s:1",
        [6, 67, 50, 40],
        1,
        identity(),
        [0, 0],
    );
    let guide = nodes.len() - 1;
    push_node(
        &mut nodes,
        "root/s:1/s:0",
        [-18, -5, 20, 16],
        guide,
        translation(SCALE / 2, -SCALE / 2),
        [0, 0],
    );
    nodes
}

fn push_node(
    nodes: &mut Vec<SpatialNode>,
    path: &str,
    base: [i32; 4],
    parent: usize,
    local: Affine,
    origin: Point,
) {
    let parent_base = nodes[parent].base;
    let local_x = fixed(base[0]) - parent_base[0];
    let local_y = fixed(base[1]) - parent_base[1];
    let placed = compose(translation(local_x, local_y), about(local, origin));
    let world = compose(nodes[parent].world, placed);
    nodes.push(SpatialNode {
        path: Some(path.to_owned()),
        base: [
            fixed(base[0]),
            fixed(base[1]),
            fixed(base[2]),
            fixed(base[3]),
        ],
        world,
    });
}
