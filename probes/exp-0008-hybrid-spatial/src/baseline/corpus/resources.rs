use crate::baseline::literal_types::{
    BrushInputV2, ClipInputV2, CoverageInputV2, FIXED_ONE_V2, FillRuleV2, HitInputV2,
    PaintContentInputV2, PaintInputV2, PathInputV2, PathVerbInputV2 as V, PointV2, RectV2,
    SceneInputV2, SemanticInputV2, ShapeGeometryInputV2, ShapeInputV2,
};

#[path = "resources/rich.rs"]
mod rich;

pub(in crate::baseline::corpus) use rich::add_rich_content_v2;

pub(super) fn add_basic_content_v2(scene: &mut SceneInputV2, owner: u32, width: i32, height: i32) {
    let shape = next(scene.shapes.len());
    let brush = next(scene.brushes.len());
    let item = scene
        .paints
        .iter()
        .filter(|value| value.owner == owner)
        .count() as u32;
    scene.shapes.push(ShapeInputV2 {
        key: shape,
        owner,
        geometry: ShapeGeometryInputV2::Rect(rect(0, 0, width, height)),
    });
    scene.brushes.push(BrushInputV2::Solid {
        key: brush,
        color: [40, 88, 136, 224],
    });
    scene.paints.push(PaintInputV2 {
        owner,
        item,
        content: PaintContentInputV2::Coverage {
            coverage: fill(shape, FillRuleV2::NonZero),
            brush,
            opacity: 224,
            clip: None,
        },
    });
    scene.hits.push(HitInputV2 {
        owner,
        item,
        coverage: fill(shape, FillRuleV2::NonZero),
        clip: None,
        accepts: true,
    });
    scene.semantics.push(SemanticInputV2 {
        owner,
        item,
        shape,
        rule: FillRuleV2::NonZero,
        clip: None,
    });
}

pub(super) fn add_split_content_v2(scene: &mut SceneInputV2, owner: u32) {
    let paint_shape = next(scene.shapes.len());
    let hit_shape = paint_shape + 1;
    let semantic_shape = paint_shape + 2;
    scene.shapes.extend([
        ShapeInputV2 {
            key: paint_shape,
            owner,
            geometry: ShapeGeometryInputV2::Rect(rect(-4, -3, 56, 40)),
        },
        ShapeInputV2 {
            key: hit_shape,
            owner,
            geometry: ShapeGeometryInputV2::Circle {
                center: point(24, 16),
                radius: 10 * FIXED_ONE_V2,
            },
        },
        ShapeInputV2 {
            key: semantic_shape,
            owner,
            geometry: ShapeGeometryInputV2::Rect(rect(8, 6, 28, 18)),
        },
    ]);
    let brush = next(scene.brushes.len());
    scene.brushes.push(BrushInputV2::Solid {
        key: brush,
        color: [180, 70, 30, 192],
    });
    scene.paints.push(PaintInputV2 {
        owner,
        item: 0,
        content: PaintContentInputV2::Coverage {
            coverage: fill(paint_shape, FillRuleV2::NonZero),
            brush,
            opacity: 255,
            clip: None,
        },
    });
    scene.hits.push(HitInputV2 {
        owner,
        item: 0,
        coverage: fill(hit_shape, FillRuleV2::NonZero),
        clip: None,
        accepts: true,
    });
    scene.semantics.push(SemanticInputV2 {
        owner,
        item: 0,
        shape: semantic_shape,
        rule: FillRuleV2::NonZero,
        clip: None,
    });
}

pub(super) fn add_clipped_content_v2(scene: &mut SceneInputV2, owner: u32) {
    let outer = next(scene.shapes.len());
    let inner = outer + 1;
    let content = outer + 2;
    scene.shapes.extend([
        ShapeInputV2 {
            key: outer,
            owner,
            geometry: ShapeGeometryInputV2::Rect(rect(0, 0, 22, 18)),
        },
        ShapeInputV2 {
            key: inner,
            owner,
            geometry: ShapeGeometryInputV2::Circle {
                center: point(11, 9),
                radius: 8 * FIXED_ONE_V2,
            },
        },
        ShapeInputV2 {
            key: content,
            owner,
            geometry: ShapeGeometryInputV2::Rect(rect(-2, -2, 26, 22)),
        },
    ]);
    scene.clips.extend([
        ClipInputV2 {
            key: 0,
            owner,
            parent: None,
            shape: outer,
            rule: FillRuleV2::NonZero,
        },
        ClipInputV2 {
            key: 1,
            owner,
            parent: Some(0),
            shape: inner,
            rule: FillRuleV2::NonZero,
        },
    ]);
    let brush = next(scene.brushes.len());
    scene.brushes.push(BrushInputV2::Solid {
        key: brush,
        color: [72, 140, 96, 208],
    });
    scene.paints.push(PaintInputV2 {
        owner,
        item: 0,
        content: PaintContentInputV2::Coverage {
            coverage: fill(content, FillRuleV2::NonZero),
            brush,
            opacity: 255,
            clip: Some(1),
        },
    });
    scene.hits.push(HitInputV2 {
        owner,
        item: 0,
        coverage: fill(content, FillRuleV2::NonZero),
        clip: Some(1),
        accepts: true,
    });
    scene.semantics.push(SemanticInputV2 {
        owner,
        item: 0,
        shape: content,
        rule: FillRuleV2::NonZero,
        clip: Some(1),
    });
}

pub(super) fn add_path_content_v2(scene: &mut SceneInputV2, owner: u32) {
    scene.paths.push(PathInputV2 {
        key: 0,
        owner,
        verbs: vec![
            V::Move(point(2, 2)),
            V::Line(point(28, 2)),
            V::Line(point(12, 12)),
            V::Line(point(28, 26)),
            V::Line(point(2, 26)),
            V::Line(point(2, 2)),
            V::Close,
            V::Move(point(7, 7)),
            V::Line(point(7, 7)),
            V::Quadratic(point(16, 1), point(25, 8)),
            V::Cubic(point(29, 14), point(4, 18), point(22, 24)),
            V::Line(point(7, 7)),
            V::Close,
        ],
    });
    scene.shapes.extend([
        ShapeInputV2 {
            key: 0,
            owner,
            geometry: ShapeGeometryInputV2::Polygon {
                points: vec![
                    point(2, 2),
                    point(26, 2),
                    point(14, 10),
                    point(26, 26),
                    point(2, 26),
                ],
            },
        },
        ShapeInputV2 {
            key: 1,
            owner,
            geometry: ShapeGeometryInputV2::Path { path: 0 },
        },
    ]);
    scene.brushes.push(BrushInputV2::Solid {
        key: 0,
        color: [150, 80, 190, 192],
    });
    scene.paints.extend([
        PaintInputV2 {
            owner,
            item: 0,
            content: PaintContentInputV2::Coverage {
                coverage: fill(0, FillRuleV2::EvenOdd),
                brush: 0,
                opacity: 180,
                clip: None,
            },
        },
        PaintInputV2 {
            owner,
            item: 1,
            content: PaintContentInputV2::Coverage {
                coverage: CoverageInputV2::RoundStroke {
                    shape: 1,
                    width: 3 * FIXED_ONE_V2 / 2,
                },
                brush: 0,
                opacity: 255,
                clip: None,
            },
        },
    ]);
    scene.hits.extend([
        hit(owner, 0, fill(0, FillRuleV2::NonZero)),
        hit(owner, 1, fill(1, FillRuleV2::EvenOdd)),
    ]);
    scene.semantics.push(SemanticInputV2 {
        owner,
        item: 0,
        shape: 1,
        rule: FillRuleV2::EvenOdd,
        clip: None,
    });
}

pub(super) fn complete_direct_queries_v2(scene: &mut SceneInputV2) {
    let mut queries = vec![point(0, 0), point(1, 1), point(8, 8), point(16, 12)];
    for shape in &scene.shapes {
        match &shape.geometry {
            ShapeGeometryInputV2::Rect(value) => {
                queries.extend([
                    PointV2 {
                        x: value.x,
                        y: value.y,
                    },
                    PointV2 {
                        x: value.x + value.width,
                        y: value.y,
                    },
                    PointV2 {
                        x: value.x,
                        y: value.y + value.height,
                    },
                    PointV2 {
                        x: value.x + value.width,
                        y: value.y + value.height,
                    },
                    PointV2 {
                        x: value.x + value.width / 2,
                        y: value.y + value.height / 2,
                    },
                ]);
            }
            ShapeGeometryInputV2::Circle { center, radius } => queries.extend([
                *center,
                PointV2 {
                    x: center.x + radius,
                    y: center.y,
                },
                PointV2 {
                    x: center.x + radius,
                    y: center.y + radius,
                },
            ]),
            ShapeGeometryInputV2::Polygon { points } => {
                for pair in points.windows(2) {
                    queries.push(pair[0]);
                    queries.push(midpoint(pair[0], pair[1]));
                }
            }
            ShapeGeometryInputV2::Path { .. } => queries.push(point(14, 10)),
        }
    }
    queries.push(queries[0]);
    queries.extend([
        PointV2 { x: -1, y: 0 },
        PointV2 { x: 0, y: -1 },
        PointV2 {
            x: i64::from(scene.viewport.0) * FIXED_ONE_V2,
            y: 0,
        },
        PointV2 {
            x: 0,
            y: i64::from(scene.viewport.1) * FIXED_ONE_V2,
        },
    ]);
    scene.queries = queries;
}

fn rect(x: i32, y: i32, width: i32, height: i32) -> RectV2 {
    RectV2 {
        x: i64::from(x) * FIXED_ONE_V2,
        y: i64::from(y) * FIXED_ONE_V2,
        width: i64::from(width) * FIXED_ONE_V2,
        height: i64::from(height) * FIXED_ONE_V2,
    }
}

fn point(x: i32, y: i32) -> PointV2 {
    PointV2 {
        x: i64::from(x) * FIXED_ONE_V2,
        y: i64::from(y) * FIXED_ONE_V2,
    }
}

fn midpoint(left: PointV2, right: PointV2) -> PointV2 {
    PointV2 {
        x: left.x + (right.x - left.x) / 2,
        y: left.y + (right.y - left.y) / 2,
    }
}

fn fill(shape: u32, rule: FillRuleV2) -> CoverageInputV2 {
    CoverageInputV2::Fill { shape, rule }
}

fn hit(owner: u32, item: u32, coverage: CoverageInputV2) -> HitInputV2 {
    HitInputV2 {
        owner,
        item,
        coverage,
        clip: None,
        accepts: true,
    }
}

fn next(value: usize) -> u32 {
    u32::try_from(value).expect("registered resource key should fit")
}
