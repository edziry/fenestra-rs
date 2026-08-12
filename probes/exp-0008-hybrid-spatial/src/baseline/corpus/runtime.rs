use crate::baseline::literal_types::{
    AffineV2, AnchorComponentV2 as A, AnchorTargetV2 as T, AxisV2, BrushInputV2, ClipInputV2,
    CoverageInputV2, FIXED_ONE_V2, FillRuleV2, HitInputV2, LiteralObservationInputV2, NodeInputV2,
    PaintContentInputV2, PaintInputV2, PlacementInputV2, PointV2, ReceiptInputV2, RectV2,
    SceneInputV2, SemanticInputV2, ShapeGeometryInputV2, ShapeInputV2,
};

pub(super) fn build_mutation_observations_v2() -> Vec<LiteralObservationInputV2> {
    (0..9)
        .map(|step| LiteralObservationInputV2 {
            step,
            scene: scene_for_step_v2(step),
        })
        .collect()
}

pub(super) fn build_rollback_observation_v2() -> Vec<LiteralObservationInputV2> {
    vec![LiteralObservationInputV2 {
        step: 0,
        scene: scene_for_step_v2(8),
    }]
}

fn scene_for_step_v2(step: u8) -> SceneInputV2 {
    let viewport = if step == 0 { (192, 128) } else { (224, 160) };
    let span_x = if step < 2 { 180 } else { 176 };
    let tone = if step < 3 {
        [96, 72, 48, 255]
    } else {
        [80, 40, 24, 255]
    };
    let policy = step >= 4;
    let keys = match step {
        0..=4 => vec![(10, 12), (20, 12)],
        5 => vec![(10, 12), (30, 12), (20, 12)],
        6 => vec![(10, 12), (20, 12), (30, 12)],
        7 => vec![(10, 12), (20, 12), (30, 14)],
        8 => vec![(10, 12), (30, 14)],
        _ => unreachable!("only registered successful runtime steps are built"),
    };
    let mut scene = SceneInputV2 {
        viewport,
        receipt: ReceiptInputV2 {
            generation: Some(u64::from(step)),
            mutation_count: u64::from(step != 0),
            invalidation: invalidation(step),
        },
        nodes: runtime_nodes(span_x, &keys),
        paths: Vec::new(),
        shapes: Vec::new(),
        clips: Vec::new(),
        brushes: Vec::new(),
        images: Vec::new(),
        paints: Vec::new(),
        hits: Vec::new(),
        semantics: Vec::new(),
        queries: Vec::new(),
    };
    add_runtime_content(&mut scene, tone, policy);
    scene.queries = runtime_queries(viewport);
    scene
}

fn runtime_nodes(span_x: i32, keys: &[(u64, i32)]) -> Vec<NodeInputV2> {
    let mut nodes = vec![
        root(),
        layout(1, 0, "root", span_x, 120),
        layout(2, 1, "root/s:0", 80, 60),
        free(3, 2, "root/s:0/s:0", 40, 30, 10, 8),
        layout(4, 3, "root/s:0/s:0/s:0", 12, 10),
    ];
    for (key, height) in keys {
        let spatial = u32::try_from(nodes.len()).expect("runtime key should fit");
        nodes.push(layout(
            spatial,
            3,
            &format!("root/s:0/s:0/m:1:{key}"),
            16,
            *height,
        ));
    }
    let guide = u32::try_from(nodes.len()).expect("guide key should fit");
    nodes.push(free(guide, 1, "root/s:1", 50, 40, 90, 12));
    nodes.push(free(guide + 1, guide, "root/s:1/s:0", 20, 16, 6, 5));
    nodes
}

fn root() -> NodeInputV2 {
    NodeInputV2 {
        key: 0,
        path: None,
        parent: None,
        placement: PlacementInputV2::Root,
        axis: AxisV2::Vertical,
        padding: [2, 2, 2, 2],
        gap: 2,
    }
}

fn layout(key: u32, parent: u32, path: &str, width: i32, height: i32) -> NodeInputV2 {
    node(
        key,
        parent,
        path,
        PlacementInputV2::Layout {
            width,
            height,
            transform: AffineV2::IDENTITY,
        },
    )
}

fn free(key: u32, parent: u32, path: &str, width: i32, height: i32, x: i32, y: i32) -> NodeInputV2 {
    node(
        key,
        parent,
        path,
        PlacementInputV2::Free {
            width,
            height,
            self_anchor: [A::Start, A::Start],
            target: T::Parent,
            target_anchor: [A::Start, A::Start],
            offset: point(x, y),
            transform: AffineV2::IDENTITY,
        },
    )
}

fn node(key: u32, parent: u32, path: &str, placement: PlacementInputV2) -> NodeInputV2 {
    NodeInputV2 {
        key,
        path: Some(path.to_owned()),
        parent: Some(parent),
        placement,
        axis: AxisV2::Vertical,
        padding: [1, 1, 1, 1],
        gap: 1,
    }
}

fn add_runtime_content(scene: &mut SceneInputV2, tone: [u8; 4], policy: bool) {
    let owners = [1, 2, 3, 4, 5, 6];
    for (index, owner) in owners.into_iter().enumerate() {
        let key = u32::try_from(index).expect("shape key should fit");
        scene.shapes.push(ShapeInputV2 {
            key,
            owner,
            geometry: ShapeGeometryInputV2::Rect(rect(0, 0, 10 + index as i32, 8 + index as i32)),
        });
    }
    for index in 0..5_u32 {
        let shape = 6 + index;
        let owner = owners[index as usize];
        scene.shapes.push(ShapeInputV2 {
            key: shape,
            owner,
            geometry: ShapeGeometryInputV2::Rect(rect(1, 1, 8 + index as i32, 6 + index as i32)),
        });
        scene.clips.push(ClipInputV2 {
            key: index,
            owner,
            parent: None,
            shape,
            rule: FillRuleV2::NonZero,
        });
    }
    scene.brushes.push(BrushInputV2::Solid {
        key: 0,
        color: tone,
    });
    for (index, owner) in owners.into_iter().enumerate() {
        let shape = u32::try_from(index).expect("shape key should fit");
        let clip = (index < 5).then_some(shape);
        scene.paints.push(PaintInputV2 {
            owner,
            item: 0,
            content: PaintContentInputV2::Coverage {
                coverage: coverage(shape),
                brush: 0,
                opacity: 224,
                clip,
            },
        });
        scene.hits.push(HitInputV2 {
            owner,
            item: 0,
            coverage: coverage(shape),
            clip,
            accepts: policy || index != 4,
        });
        if index < 5 {
            scene.semantics.push(SemanticInputV2 {
                owner,
                item: 0,
                shape,
                rule: FillRuleV2::NonZero,
                clip,
            });
        }
    }
}

fn runtime_queries(viewport: (u32, u32)) -> Vec<PointV2> {
    let count = viewport.0 as usize * viewport.1 as usize + 4;
    let mut queries = Vec::with_capacity(count);
    for y in 0..viewport.1 {
        for x in 0..viewport.0 {
            queries.push(PointV2 {
                x: i64::from(x) * FIXED_ONE_V2 + FIXED_ONE_V2 / 2,
                y: i64::from(y) * FIXED_ONE_V2 + FIXED_ONE_V2 / 2,
            });
        }
    }
    queries.extend([
        PointV2 { x: -1, y: 0 },
        PointV2 { x: 0, y: -1 },
        PointV2 {
            x: i64::from(viewport.0) * FIXED_ONE_V2,
            y: 0,
        },
        PointV2 {
            x: 0,
            y: i64::from(viewport.1) * FIXED_ONE_V2,
        },
    ]);
    queries
}

fn coverage(shape: u32) -> CoverageInputV2 {
    CoverageInputV2::Fill {
        shape,
        rule: FillRuleV2::NonZero,
    }
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

const fn invalidation(step: u8) -> u64 {
    match step {
        0 => 0,
        1 | 2 | 7 => 0xff,
        3 => 1 << 6,
        4 => 1 << 5,
        5 | 6 | 8 => 0xffff,
        _ => 0,
    }
}
