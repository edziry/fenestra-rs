use fenestra_ui_ir::prototype::{
    SpatialBrushContentV2, SpatialClipDeclarationV2, SpatialCoverageRecipeV2, SpatialFillRuleV2,
    SpatialHitRecipeV2, SpatialNodeDeclarationV2, SpatialPaintRecipeV2, SpatialPathVerbRecipeV2,
    SpatialSemanticRecipeV2, SpatialShapeGeometryV2,
};

use crate::semantic::{InvalidRecord, Record};
use crate::vocabulary_v2::AnchorKindV2;

use super::catalog::SourceCatalog;
use super::field;

pub(super) fn collect(
    node: &SpatialNodeDeclarationV2,
    records: &mut Vec<Record>,
    catalog: &SourceCatalog<'_>,
) -> Result<(), InvalidRecord> {
    let node_symbol = node.symbol().value().get();
    collect_shapes(node, node_symbol, records, catalog)?;
    collect_brushes(node, node_symbol, records, catalog)?;
    for (order, clip) in node.clips().iter().enumerate() {
        collect_clip(*clip, node_symbol, order, records, catalog)?;
    }
    for (order, paint) in node.paint_items().iter().enumerate() {
        collect_paint(*paint, node_symbol, order, records, catalog)?;
    }
    for (order, hit) in node.hit_items().iter().enumerate() {
        collect_hit(*hit, node_symbol, order, records, catalog)?;
    }
    for (order, semantic) in node.semantic_items().iter().enumerate() {
        collect_semantic(*semantic, node_symbol, order, records, catalog)?;
    }
    Ok(())
}

fn collect_shapes(
    node: &SpatialNodeDeclarationV2,
    node_symbol: u32,
    records: &mut Vec<Record>,
    catalog: &SourceCatalog<'_>,
) -> Result<(), InvalidRecord> {
    for (order, shape) in node.shapes().iter().enumerate() {
        let (anchor, name) = catalog.named_anchor(shape.span(), AnchorKindV2::SpatialShape)?;
        let symbol = shape.symbol().value().get();
        if symbol != u32::try_from(order).map_err(|_| InvalidRecord)? {
            return Err(InvalidRecord);
        }
        let kind = match shape.geometry() {
            SpatialShapeGeometryV2::Rect { .. } => "rect",
            SpatialShapeGeometryV2::Circle { .. } => "circle",
            SpatialShapeGeometryV2::Polygon { .. } => "polygon",
            SpatialShapeGeometryV2::Path { .. } => "path",
        };
        records.push(Record::new(
            anchor,
            "spatial-shape",
            format!("node={node_symbol}|order={order}|name={name}|kind={kind}"),
        )?);
        field::push(records, catalog, anchor, "symbol", shape.symbol())?;
        match shape.geometry() {
            SpatialShapeGeometryV2::Rect {
                origin,
                width,
                height,
            } => {
                field::point(records, catalog, anchor, "origin-x", "origin-y", *origin)?;
                field::push(records, catalog, anchor, "width", *width)?;
                field::push(records, catalog, anchor, "height", *height)?;
            }
            SpatialShapeGeometryV2::Circle { center, radius } => {
                field::point(records, catalog, anchor, "center-x", "center-y", *center)?;
                field::push(records, catalog, anchor, "radius", *radius)?;
            }
            SpatialShapeGeometryV2::Polygon { points } => {
                for (point_order, point) in points.iter().enumerate() {
                    let point_anchor =
                        catalog.anchor(point.span(), AnchorKindV2::SpatialPolygonPoint)?;
                    records.push(Record::new(
                        point_anchor,
                        "spatial-polygon-point",
                        format!("node={node_symbol}|shape={symbol}|order={point_order}"),
                    )?);
                    field::point(records, catalog, point_anchor, "x", "y", point.point())?;
                }
            }
            SpatialShapeGeometryV2::Path { verbs } => {
                for (verb_order, verb) in verbs.iter().enumerate() {
                    collect_verb(*verb, node_symbol, symbol, verb_order, records, catalog)?;
                }
            }
        }
    }
    Ok(())
}

fn collect_verb(
    verb: SpatialPathVerbRecipeV2,
    node: u32,
    shape: u32,
    order: usize,
    records: &mut Vec<Record>,
    catalog: &SourceCatalog<'_>,
) -> Result<(), InvalidRecord> {
    let anchor = catalog.anchor(verb.span(), AnchorKindV2::SpatialPathVerb)?;
    let kind = match verb {
        SpatialPathVerbRecipeV2::MoveTo { .. } => "move-to",
        SpatialPathVerbRecipeV2::LineTo { .. } => "line-to",
        SpatialPathVerbRecipeV2::QuadraticTo { .. } => "quadratic-to",
        SpatialPathVerbRecipeV2::CubicTo { .. } => "cubic-to",
        SpatialPathVerbRecipeV2::Close { .. } => "close",
    };
    records.push(Record::new(
        anchor,
        "spatial-path-verb",
        format!("node={node}|shape={shape}|order={order}|kind={kind}"),
    )?);
    match verb {
        SpatialPathVerbRecipeV2::MoveTo { to, .. } | SpatialPathVerbRecipeV2::LineTo { to, .. } => {
            field::point(records, catalog, anchor, "to-x", "to-y", to)
        }
        SpatialPathVerbRecipeV2::QuadraticTo { control, to, .. } => {
            field::point(records, catalog, anchor, "control-x", "control-y", control)?;
            field::point(records, catalog, anchor, "to-x", "to-y", to)
        }
        SpatialPathVerbRecipeV2::CubicTo {
            control1,
            control2,
            to,
            ..
        } => {
            field::point(
                records,
                catalog,
                anchor,
                "control1-x",
                "control1-y",
                control1,
            )?;
            field::point(
                records,
                catalog,
                anchor,
                "control2-x",
                "control2-y",
                control2,
            )?;
            field::point(records, catalog, anchor, "to-x", "to-y", to)
        }
        SpatialPathVerbRecipeV2::Close { .. } => Ok(()),
    }
}

fn collect_brushes(
    node: &SpatialNodeDeclarationV2,
    node_symbol: u32,
    records: &mut Vec<Record>,
    catalog: &SourceCatalog<'_>,
) -> Result<(), InvalidRecord> {
    for (order, brush) in node.brushes().iter().enumerate() {
        let (anchor, name) = catalog.named_anchor(brush.span(), AnchorKindV2::SpatialBrush)?;
        let symbol = brush.symbol().value().get();
        if symbol != u32::try_from(order).map_err(|_| InvalidRecord)? {
            return Err(InvalidRecord);
        }
        let kind = match brush.content() {
            SpatialBrushContentV2::Solid { .. } => "solid",
            SpatialBrushContentV2::LinearGradient { .. } => "linear-gradient",
        };
        records.push(Record::new(
            anchor,
            "spatial-brush",
            format!("node={node_symbol}|order={order}|name={name}|kind={kind}"),
        )?);
        field::push(records, catalog, anchor, "symbol", brush.symbol())?;
        match brush.content() {
            SpatialBrushContentV2::Solid { color } => {
                field::push(records, catalog, anchor, "color", *color)?;
            }
            SpatialBrushContentV2::LinearGradient { start, end, stops } => {
                field::point(records, catalog, anchor, "start-x", "start-y", *start)?;
                field::point(records, catalog, anchor, "end-x", "end-y", *end)?;
                for (stop_order, stop) in stops.iter().enumerate() {
                    let stop_anchor =
                        catalog.anchor(stop.span(), AnchorKindV2::SpatialGradientStop)?;
                    records.push(Record::new(
                        stop_anchor,
                        "spatial-gradient-stop",
                        format!("node={node_symbol}|brush={symbol}|order={stop_order}"),
                    )?);
                    field::push(records, catalog, stop_anchor, "offset", stop.offset())?;
                    field::push(records, catalog, stop_anchor, "color", stop.color())?;
                }
            }
        }
    }
    Ok(())
}

fn collect_clip(
    clip: SpatialClipDeclarationV2,
    node: u32,
    order: usize,
    records: &mut Vec<Record>,
    catalog: &SourceCatalog<'_>,
) -> Result<(), InvalidRecord> {
    let (anchor, name) = catalog.named_anchor(clip.span(), AnchorKindV2::SpatialClip)?;
    if clip.symbol().value().get() != u32::try_from(order).map_err(|_| InvalidRecord)? {
        return Err(InvalidRecord);
    }
    records.push(Record::new(
        anchor,
        "spatial-clip",
        format!(
            "node={node}|order={order}|name={name}|parent={}|fill-rule={}",
            optional_clip(clip.parent()),
            fill_rule(clip.fill_rule()),
        ),
    )?);
    field::push(records, catalog, anchor, "symbol", clip.symbol())?;
    if let Some(parent) = clip.parent() {
        field::push(records, catalog, anchor, "parent-owner", parent.owner())?;
        field::push(records, catalog, anchor, "parent-clip", parent.clip())?;
    }
    field::push(records, catalog, anchor, "shape", clip.shape())
}

fn collect_paint(
    paint: SpatialPaintRecipeV2,
    node: u32,
    order: usize,
    records: &mut Vec<Record>,
    catalog: &SourceCatalog<'_>,
) -> Result<(), InvalidRecord> {
    let anchor = catalog.anchor(paint.span(), AnchorKindV2::SpatialPaint)?;
    let payload = match paint {
        SpatialPaintRecipeV2::CoveragePaint { coverage, clip, .. } => format!(
            "node={node}|order={order}|kind=coverage|{}|clip={}",
            coverage_name(coverage),
            optional_clip(clip)
        ),
        SpatialPaintRecipeV2::ImagePaint { clip, .. } => {
            format!(
                "node={node}|order={order}|kind=image|clip={}",
                optional_clip(clip)
            )
        }
    };
    records.push(Record::new(anchor, "spatial-paint", payload)?);
    match paint {
        SpatialPaintRecipeV2::CoveragePaint {
            coverage,
            brush,
            opacity,
            clip,
            ..
        } => {
            field::coverage(records, catalog, anchor, coverage)?;
            field::push(records, catalog, anchor, "brush", brush)?;
            field::push(records, catalog, anchor, "opacity", opacity)?;
            if let Some(clip) = clip {
                field::clip_address(records, catalog, anchor, clip)?;
            }
        }
        SpatialPaintRecipeV2::ImagePaint {
            image,
            source_x,
            source_y,
            source_width,
            source_height,
            destination_origin,
            destination_width,
            destination_height,
            opacity,
            clip,
            ..
        } => {
            field::push(records, catalog, anchor, "image", image)?;
            field::push(records, catalog, anchor, "source-x", source_x)?;
            field::push(records, catalog, anchor, "source-y", source_y)?;
            field::push(records, catalog, anchor, "source-width", source_width)?;
            field::push(records, catalog, anchor, "source-height", source_height)?;
            field::point(
                records,
                catalog,
                anchor,
                "destination-x",
                "destination-y",
                destination_origin,
            )?;
            field::push(
                records,
                catalog,
                anchor,
                "destination-width",
                destination_width,
            )?;
            field::push(
                records,
                catalog,
                anchor,
                "destination-height",
                destination_height,
            )?;
            field::push(records, catalog, anchor, "opacity", opacity)?;
            if let Some(clip) = clip {
                field::clip_address(records, catalog, anchor, clip)?;
            }
        }
    }
    Ok(())
}

fn collect_hit(
    hit: SpatialHitRecipeV2,
    node: u32,
    order: usize,
    records: &mut Vec<Record>,
    catalog: &SourceCatalog<'_>,
) -> Result<(), InvalidRecord> {
    let anchor = catalog.anchor(hit.span(), AnchorKindV2::SpatialHit)?;
    records.push(Record::new(
        anchor,
        "spatial-hit",
        format!(
            "node={node}|order={order}|{}|clip={}",
            coverage_name(hit.coverage()),
            optional_clip(hit.clip())
        ),
    )?);
    field::coverage(records, catalog, anchor, hit.coverage())?;
    if let Some(clip) = hit.clip() {
        field::clip_address(records, catalog, anchor, clip)?;
    }
    field::push(records, catalog, anchor, "input-policy", hit.input_policy())
}

fn collect_semantic(
    semantic: SpatialSemanticRecipeV2,
    node: u32,
    order: usize,
    records: &mut Vec<Record>,
    catalog: &SourceCatalog<'_>,
) -> Result<(), InvalidRecord> {
    let anchor = catalog.anchor(semantic.span(), AnchorKindV2::SpatialSemantic)?;
    records.push(Record::new(
        anchor,
        "spatial-semantic",
        format!(
            "node={node}|order={order}|fill-rule={}|clip={}",
            fill_rule(semantic.fill_rule()),
            optional_clip(semantic.clip())
        ),
    )?);
    field::push(records, catalog, anchor, "shape", semantic.shape())?;
    if let Some(clip) = semantic.clip() {
        field::clip_address(records, catalog, anchor, clip)?;
    }
    Ok(())
}

fn coverage_name(coverage: SpatialCoverageRecipeV2) -> String {
    match coverage {
        SpatialCoverageRecipeV2::Fill { rule, .. } => {
            format!("coverage=fill|fill-rule={}", fill_rule(rule))
        }
        SpatialCoverageRecipeV2::RoundStroke { .. } => "coverage=round-stroke".to_owned(),
    }
}

const fn fill_rule(rule: SpatialFillRuleV2) -> &'static str {
    match rule {
        SpatialFillRuleV2::NonZero => "non-zero",
        SpatialFillRuleV2::EvenOdd => "even-odd",
    }
}

fn optional_clip<T>(clip: Option<T>) -> &'static str {
    if clip.is_some() { "qualified" } else { "none" }
}
