use super::model::Model;
use super::numeric::{Point, SCALE, fixed};
use super::scene::{
    Brush, ClipPlan, Coverage, HitPlan, PaintContent, PaintPlan, Rule, SemanticPlan, Shape,
};
use super::types::Aabb;

pub(super) fn shapes(model: &Model) -> Vec<Shape> {
    vec![
        Shape::Rect {
            origin: [0, 0],
            width: fixed(model.span_x),
            height: fixed(120),
        },
        Shape::Circle {
            center: [5 * SCALE, 5 * SCALE],
            radius: 2 * SCALE,
        },
        Shape::Polygon(vec![[0, 0], [10 * SCALE, 0], [5 * SCALE, 8 * SCALE]]),
        Shape::Path,
    ]
}

pub(super) fn brushes(model: &Model) -> Vec<Brush> {
    vec![
        Brush::Solid(model.tone),
        Brush::Gradient {
            start: [0, 0],
            end: [10 * SCALE, 0],
            stops: vec![
                (0, [16, 64, 96, 192]),
                (32_768, [128, 64, 32, 255]),
                (65_535, model.tone),
            ],
        },
    ]
}

pub(super) fn clips() -> Vec<ClipPlan> {
    vec![
        ClipPlan {
            owner: 1,
            parent: None,
            shape: 0,
            rule: Rule::NonZero,
        },
        ClipPlan {
            owner: 1,
            parent: Some(0),
            shape: 2,
            rule: Rule::EvenOdd,
        },
    ]
}

pub(super) fn paints(model: &Model) -> Vec<PaintPlan> {
    vec![
        PaintPlan {
            owner: 1,
            item: 0,
            bounds: Aabb::new([0, 0, fixed(model.span_x), fixed(120)]),
            content: PaintContent::Coverage {
                coverage: Coverage::Fill {
                    shape: 0,
                    rule: Rule::NonZero,
                },
                brush: 0,
                opacity: 255,
                clip: Some(1),
            },
        },
        PaintPlan {
            owner: 1,
            item: 1,
            bounds: Aabb::new([-SCALE, -SCALE, 16 * SCALE, 16 * SCALE]),
            content: PaintContent::Coverage {
                coverage: Coverage::Stroke {
                    shape: 3,
                    width: 2 * SCALE,
                },
                brush: 1,
                opacity: 200,
                clip: None,
            },
        },
        PaintPlan {
            owner: 1,
            item: 2,
            bounds: Aabb::new([2 * SCALE, 2 * SCALE, 10 * SCALE, 10 * SCALE]),
            content: PaintContent::Image { clip: Some(0) },
        },
    ]
}

pub(super) fn hits(model: &Model) -> Vec<HitPlan> {
    vec![
        HitPlan {
            owner: 1,
            item: 0,
            bounds: Aabb::new([0, 0, 10 * SCALE, 8 * SCALE]),
            coverage: Coverage::Fill {
                shape: 2,
                rule: Rule::EvenOdd,
            },
            clip: Some(1),
            accepts: model.policy,
        },
        HitPlan {
            owner: 1,
            item: 1,
            bounds: Aabb::new([-SCALE / 2, -SCALE / 2, 31 * SCALE / 2, 31 * SCALE / 2]),
            coverage: Coverage::Stroke {
                shape: 3,
                width: SCALE,
            },
            clip: None,
            accepts: true,
        },
        HitPlan {
            owner: 1,
            item: 2,
            bounds: Aabb::new([3 * SCALE, 3 * SCALE, 7 * SCALE, 7 * SCALE]),
            coverage: Coverage::Fill {
                shape: 1,
                rule: Rule::NonZero,
            },
            clip: None,
            accepts: false,
        },
    ]
}

pub(super) fn semantics() -> Vec<SemanticPlan> {
    vec![
        SemanticPlan {
            owner: 1,
            item: 0,
            bounds: Aabb::new([3 * SCALE, 3 * SCALE, 7 * SCALE, 7 * SCALE]),
            shape: 1,
            clip: Some(0),
        },
        SemanticPlan {
            owner: 1,
            item: 1,
            bounds: Aabb::new([0, 0, 10 * SCALE, 8 * SCALE]),
            shape: 2,
            clip: None,
        },
    ]
}

pub(super) fn path() -> Vec<Point> {
    super::coverage::flatten_fixture_path()
}
