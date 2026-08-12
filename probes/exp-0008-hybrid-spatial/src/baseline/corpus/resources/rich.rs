use crate::baseline::literal_types::{
    BrushInputV2, ClipInputV2, CoverageInputV2, FIXED_ONE_V2, FillRuleV2, GradientStopInputV2,
    HitInputV2, ImageInputV2, PaintContentInputV2, PaintInputV2, PointV2, RectV2, SceneInputV2,
    SemanticInputV2, ShapeGeometryInputV2, ShapeInputV2,
};

pub(in crate::baseline::corpus) fn add_rich_content_v2(scene: &mut SceneInputV2) {
    scene.shapes.extend([
        shape(0, 1, rect(0, 0, 28, 20)),
        shape(1, 2, rect(0, 0, 28, 20)),
        shape(2, 3, rect(0, 0, 18, 14)),
        shape(3, 2, rect(3, 2, 20, 15)),
    ]);
    scene.clips.push(ClipInputV2 {
        key: 0,
        owner: 2,
        parent: None,
        shape: 3,
        rule: FillRuleV2::NonZero,
    });
    scene.brushes.extend([
        BrushInputV2::Solid {
            key: 0,
            color: [220, 48, 24, 160],
        },
        BrushInputV2::Linear {
            key: 1,
            stops: vec![
                GradientStopInputV2 {
                    offset: 0,
                    color: [16, 96, 208, 255],
                },
                GradientStopInputV2 {
                    offset: 32_768,
                    color: [96, 176, 64, 192],
                },
                GradientStopInputV2 {
                    offset: u16::MAX,
                    color: [224, 192, 32, 128],
                },
            ],
            start: point(0, 0),
            end: point(28, 20),
        },
    ]);
    scene.images.push(ImageInputV2 {
        key: 0,
        width: 2,
        height: 2,
        stride: 8,
        bytes: vec![255, 0, 0, 255, 0, 128, 0, 128, 0, 0, 64, 64, 32, 32, 32, 64],
    });
    scene.paints.extend([
        coverage_paint(1, 0, 0, 0, 192, None),
        coverage_paint(2, 0, 1, 1, 224, Some(0)),
        PaintInputV2 {
            owner: 3,
            item: 0,
            content: PaintContentInputV2::Image {
                image: 0,
                source: RectV2 {
                    x: 0,
                    y: 0,
                    width: 2 * FIXED_ONE_V2,
                    height: 2 * FIXED_ONE_V2,
                },
                destination: rect(0, 0, 18, 14),
                opacity: 192,
                clip: None,
            },
        },
    ]);
    scene.hits.extend([
        hit(1, 0, 0, None),
        hit(2, 0, 1, Some(0)),
        hit(3, 0, 2, None),
    ]);
    scene.semantics.extend([
        semantic(1, 0, 0, None),
        semantic(2, 0, 1, Some(0)),
        semantic(3, 0, 2, None),
    ]);
}

fn coverage_paint(
    owner: u32,
    item: u32,
    shape: u32,
    brush: u32,
    opacity: u8,
    clip: Option<u32>,
) -> PaintInputV2 {
    PaintInputV2 {
        owner,
        item,
        content: PaintContentInputV2::Coverage {
            coverage: CoverageInputV2::Fill {
                shape,
                rule: FillRuleV2::NonZero,
            },
            brush,
            opacity,
            clip,
        },
    }
}

fn hit(owner: u32, item: u32, shape: u32, clip: Option<u32>) -> HitInputV2 {
    HitInputV2 {
        owner,
        item,
        coverage: CoverageInputV2::Fill {
            shape,
            rule: FillRuleV2::NonZero,
        },
        clip,
        accepts: true,
    }
}

fn semantic(owner: u32, item: u32, shape: u32, clip: Option<u32>) -> SemanticInputV2 {
    SemanticInputV2 {
        owner,
        item,
        shape,
        rule: FillRuleV2::NonZero,
        clip,
    }
}

fn shape(key: u32, owner: u32, value: RectV2) -> ShapeInputV2 {
    ShapeInputV2 {
        key,
        owner,
        geometry: ShapeGeometryInputV2::Rect(value),
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
