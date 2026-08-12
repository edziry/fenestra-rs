use super::literal_types::{
    AnchorComponentV2, AnchorTargetV2, AxisV2, BrushInputV2, CoverageInputV2, FillRuleV2,
    PaintContentInputV2, PathVerbInputV2, PlacementInputV2, PointV2, SceneInputV2,
    ShapeGeometryInputV2,
};
use super::model::{EvidenceFieldV2 as F, EvidenceRecordV2 as R};

pub(crate) fn source_records_v2(scene: &SceneInputV2) -> Vec<R> {
    let mut records = Vec::new();
    records.extend(scene.nodes.iter().map(node_record));
    records.extend(scene.paths.iter().map(|value| {
        R::new(vec![
            F::tag("record-tag", 1),
            F::u32("key", value.key),
            F::u32("owner", value.owner),
            F::raw("verbs", encode_verbs(&value.verbs)),
        ])
    }));
    records.extend(scene.shapes.iter().map(shape_record));
    records.extend(scene.clips.iter().map(|value| {
        R::new(vec![
            F::tag("record-tag", 3),
            F::u32("key", value.key),
            F::u32("owner", value.owner),
            F::optional_u32("parent", value.parent),
            F::u32("shape", value.shape),
            F::tag("fill-rule", rule_tag(value.rule)),
        ])
    }));
    records.extend(scene.brushes.iter().map(brush_record));
    records.extend(scene.images.iter().map(|value| {
        R::new(vec![
            F::tag("record-tag", 5),
            F::u32("key", value.key),
            F::u32("width", value.width),
            F::u32("height", value.height),
            F::u32("stride", value.stride),
            F::bytes("bytes", &value.bytes),
        ])
    }));
    records.extend(scene.paints.iter().map(paint_record));
    records.extend(scene.hits.iter().map(|value| {
        let (tag, shape, rule, width) = coverage_parts(value.coverage);
        R::new(vec![
            F::tag("record-tag", 7),
            F::u32("owner", value.owner),
            F::u32("item", value.item),
            F::tag("coverage-tag", tag),
            F::u32("shape", shape),
            F::tag("fill-rule", rule),
            F::i64("stroke-width", width),
            F::optional_u32("clip", value.clip),
            F::bool("accepts", value.accepts),
        ])
    }));
    records.extend(scene.semantics.iter().map(|value| {
        R::new(vec![
            F::tag("record-tag", 8),
            F::u32("owner", value.owner),
            F::u32("item", value.item),
            F::u32("shape", value.shape),
            F::tag("fill-rule", rule_tag(value.rule)),
            F::optional_u32("clip", value.clip),
        ])
    }));
    records
}

fn node_record(value: &super::literal_types::NodeInputV2) -> R {
    let mut fields = vec![
        F::tag("record-tag", 0),
        F::u32("key", value.key),
        optional_string("path", &value.path),
        F::optional_u32("parent", value.parent),
    ];
    fields.extend(placement_fields(value.placement));
    fields.extend([
        F::tag(
            "axis",
            match value.axis {
                AxisV2::Horizontal => 0,
                AxisV2::Vertical => 1,
            },
        ),
        F::i32("padding-left", value.padding[0]),
        F::i32("padding-right", value.padding[1]),
        F::i32("padding-top", value.padding[2]),
        F::i32("padding-bottom", value.padding[3]),
        F::i32("gap", value.gap),
    ]);
    R::new(fields)
}

fn placement_fields(value: PlacementInputV2) -> Vec<F> {
    let (tag, width, height, self_anchor, target, target_anchor, offset, affine) = match value {
        PlacementInputV2::Root => (
            0,
            0,
            0,
            [0, 0],
            (0, None),
            [0, 0],
            PointV2 { x: 0, y: 0 },
            super::literal_types::AffineV2::IDENTITY,
        ),
        PlacementInputV2::Layout {
            width,
            height,
            transform,
        } => (
            1,
            width,
            height,
            [0, 0],
            (0, None),
            [0, 0],
            PointV2 { x: 0, y: 0 },
            transform,
        ),
        PlacementInputV2::Free {
            width,
            height,
            self_anchor,
            target,
            target_anchor,
            offset,
            transform,
        } => {
            let target = match target {
                AnchorTargetV2::Viewport => (0, None),
                AnchorTargetV2::Parent => (1, None),
                AnchorTargetV2::Node(key) => (2, Some(key)),
            };
            (
                2,
                width,
                height,
                anchors(self_anchor),
                target,
                anchors(target_anchor),
                offset,
                transform,
            )
        }
    };
    let mut fields = vec![
        F::tag("placement-tag", tag),
        F::i32("width", width),
        F::i32("height", height),
        F::tag("self-anchor-x", self_anchor[0]),
        F::tag("self-anchor-y", self_anchor[1]),
        F::tag("target-tag", target.0),
        F::optional_u32("target-key", target.1),
        F::tag("target-anchor-x", target_anchor[0]),
        F::tag("target-anchor-y", target_anchor[1]),
        F::i64("offset-x", offset.x),
        F::i64("offset-y", offset.y),
    ];
    fields.extend(
        [
            "affine-a",
            "affine-b",
            "affine-c",
            "affine-d",
            "affine-tx",
            "affine-ty",
        ]
        .into_iter()
        .zip(affine.values)
        .map(|(name, value)| F::i64(name, value)),
    );
    fields.extend([
        F::i64("origin-x", affine.origin.x),
        F::i64("origin-y", affine.origin.y),
    ]);
    fields
}

fn shape_record(value: &super::literal_types::ShapeInputV2) -> R {
    let (tag, payload) = match &value.geometry {
        ShapeGeometryInputV2::Rect(value) => {
            (0, scalars(&[value.x, value.y, value.width, value.height]))
        }
        ShapeGeometryInputV2::Circle { center, radius } => {
            (1, scalars(&[center.x, center.y, *radius]))
        }
        ShapeGeometryInputV2::Polygon { points } => {
            let values = points
                .iter()
                .flat_map(|value| [value.x, value.y])
                .collect::<Vec<_>>();
            (2, scalars(&values))
        }
        ShapeGeometryInputV2::Path { path } => (3, path.to_le_bytes().to_vec()),
    };
    R::new(vec![
        F::tag("record-tag", 2),
        F::u32("key", value.key),
        F::u32("owner", value.owner),
        F::tag("shape-tag", tag),
        F::raw("payload", payload),
    ])
}

fn brush_record(value: &BrushInputV2) -> R {
    let (tag, payload) = match value {
        BrushInputV2::Solid { color, .. } => (0, color.to_vec()),
        BrushInputV2::Linear {
            stops, start, end, ..
        } => {
            let mut bytes = (stops.len() as u64).to_le_bytes().to_vec();
            for stop in stops {
                bytes.extend_from_slice(&stop.offset.to_le_bytes());
                bytes.extend_from_slice(&stop.color);
            }
            bytes.extend_from_slice(&scalars(&[start.x, start.y, end.x, end.y]));
            (1, bytes)
        }
    };
    R::new(vec![
        F::tag("record-tag", 4),
        F::u32("key", value.key()),
        F::tag("brush-tag", tag),
        F::raw("payload", payload),
    ])
}

fn paint_record(value: &super::literal_types::PaintInputV2) -> R {
    let (tag, shape, rule, width, brush, image, source, destination, opacity, clip) =
        match &value.content {
            PaintContentInputV2::Coverage {
                coverage,
                brush,
                opacity,
                clip,
            } => {
                let (coverage_tag, shape, rule, width) = coverage_parts(*coverage);
                (
                    coverage_tag,
                    Some(shape),
                    rule,
                    width,
                    Some(*brush),
                    None,
                    None,
                    None,
                    *opacity,
                    *clip,
                )
            }
            PaintContentInputV2::Image {
                image,
                source,
                destination,
                opacity,
                clip,
            } => (
                2,
                None,
                0,
                0,
                None,
                Some(*image),
                Some(*source),
                Some(*destination),
                *opacity,
                *clip,
            ),
        };
    R::new(vec![
        F::tag("record-tag", 6),
        F::u32("owner", value.owner),
        F::u32("item", value.item),
        F::tag("paint-tag", tag),
        F::optional_u32("shape", shape),
        F::tag("fill-rule", rule),
        F::i64("stroke-width", width),
        F::optional_u32("brush", brush),
        F::optional_u32("image", image),
        optional_rect("source", source),
        optional_rect("destination", destination),
        F::tag("opacity", opacity),
        F::optional_u32("clip", clip),
    ])
}

fn encode_verbs(values: &[PathVerbInputV2]) -> Vec<u8> {
    let mut output = (values.len() as u64).to_le_bytes().to_vec();
    for value in values {
        let (tag, points): (u8, &[PointV2]) = match value {
            PathVerbInputV2::Move(point) => (0, std::slice::from_ref(point)),
            PathVerbInputV2::Line(point) => (1, std::slice::from_ref(point)),
            PathVerbInputV2::Quadratic(first, _) => (2, std::slice::from_ref(first)),
            PathVerbInputV2::Cubic(first, _, _) => (3, std::slice::from_ref(first)),
            PathVerbInputV2::Close => (4, &[]),
        };
        output.push(tag);
        match value {
            PathVerbInputV2::Quadratic(first, second) => {
                output.extend_from_slice(&scalars(&[first.x, first.y, second.x, second.y]))
            }
            PathVerbInputV2::Cubic(first, second, third) => output.extend_from_slice(&scalars(&[
                first.x, first.y, second.x, second.y, third.x, third.y,
            ])),
            _ => {
                for point in points {
                    output.extend_from_slice(&scalars(&[point.x, point.y]));
                }
            }
        }
    }
    output
}

fn coverage_parts(value: CoverageInputV2) -> (u8, u32, u8, i64) {
    match value {
        CoverageInputV2::Fill { shape, rule } => (0, shape, rule_tag(rule), 0),
        CoverageInputV2::RoundStroke { shape, width } => (1, shape, 0, width),
    }
}

fn anchors(values: [AnchorComponentV2; 2]) -> [u8; 2] {
    values.map(|value| match value {
        AnchorComponentV2::Start => 0,
        AnchorComponentV2::Center => 1,
        AnchorComponentV2::End => 2,
    })
}

fn rule_tag(value: FillRuleV2) -> u8 {
    match value {
        FillRuleV2::NonZero => 0,
        FillRuleV2::EvenOdd => 1,
    }
}

fn scalars(values: &[i64]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

fn optional_string(name: &'static str, value: &Option<String>) -> F {
    let mut bytes = vec![u8::from(value.is_some())];
    if let Some(value) = value {
        bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
        bytes.extend_from_slice(value.as_bytes());
    }
    F::raw(name, bytes)
}

fn optional_rect(name: &'static str, value: Option<super::literal_types::RectV2>) -> F {
    let mut bytes = vec![u8::from(value.is_some())];
    if let Some(value) = value {
        bytes.extend_from_slice(&scalars(&[value.x, value.y, value.width, value.height]));
    }
    F::raw(name, bytes)
}
