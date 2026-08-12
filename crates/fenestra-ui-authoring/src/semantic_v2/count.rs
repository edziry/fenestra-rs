use fenestra_ui_ir::prototype::{
    SpatialBrushContentV2, SpatialCoverageRecipeV2, SpatialNodeDeclarationV2, SpatialNodeParentV2,
    SpatialPaintRecipeV2, SpatialPathVerbRecipeV2, SpatialPlacementRecipeV2, SpatialProgramV2,
    SpatialShapeGeometryV2,
};

use crate::resolved_v2::ResolvedDocumentV2;
use crate::semantic::logical_record_count_v1;

pub(super) fn record_count(
    resolved: &ResolvedDocumentV2,
    spatial: &SpatialProgramV2,
) -> Option<usize> {
    let mut count = logical_record_count_v1(&resolved.core)?;
    add(&mut count, 8)?;
    for _ in spatial.images() {
        add(&mut count, 5)?;
    }
    for node in spatial.nodes() {
        add(&mut count, node_count(node)?)?;
    }
    Some(count)
}

fn node_count(node: &SpatialNodeDeclarationV2) -> Option<usize> {
    let mut count = 4usize;
    add(&mut count, 2)?;
    if matches!(node.parent(), SpatialNodeParentV2::Node(_)) {
        add(&mut count, 1)?;
    }
    add(&mut count, 5)?;
    match node.placement() {
        SpatialPlacementRecipeV2::Layout(_) => add(&mut count, 6)?,
        SpatialPlacementRecipeV2::Free(free) => {
            add(&mut count, 4)?;
            if matches!(
                free.target(),
                fenestra_ui_ir::prototype::SpatialAnchorTargetRecipeV2::Node(_)
            ) {
                add(&mut count, 1)?;
            }
        }
    }
    add(&mut count, 8)?;

    for shape in node.shapes() {
        add(&mut count, 2)?;
        match shape.geometry() {
            SpatialShapeGeometryV2::Rect { .. } => add(&mut count, 4)?,
            SpatialShapeGeometryV2::Circle { .. } => add(&mut count, 3)?,
            SpatialShapeGeometryV2::Polygon { points } => {
                add(&mut count, checked_mul(points.len(), 3)?)?;
            }
            SpatialShapeGeometryV2::Path { verbs } => {
                for verb in verbs {
                    add(&mut count, verb_count(*verb)?)?;
                }
            }
        }
    }
    for brush in node.brushes() {
        add(&mut count, 2)?;
        match brush.content() {
            SpatialBrushContentV2::Solid { .. } => add(&mut count, 1)?,
            SpatialBrushContentV2::LinearGradient { stops, .. } => {
                add(&mut count, 4)?;
                add(&mut count, checked_mul(stops.len(), 3)?)?;
            }
        }
    }
    for clip in node.clips() {
        add(&mut count, if clip.parent().is_some() { 5 } else { 3 })?;
    }
    for paint in node.paint_items() {
        add(&mut count, paint_count(*paint))?;
    }
    for hit in node.hit_items() {
        let fields = coverage_fields(hit.coverage())
            .checked_add(1)?
            .checked_add(usize::from(hit.clip().is_some()) * 2)?;
        add(&mut count, 1usize.checked_add(fields)?)?;
    }
    for semantic in node.semantic_items() {
        let fields = 1usize.checked_add(usize::from(semantic.clip().is_some()) * 2)?;
        add(&mut count, 1usize.checked_add(fields)?)?;
    }
    Some(count)
}

fn verb_count(verb: SpatialPathVerbRecipeV2) -> Option<usize> {
    let fields = match verb {
        SpatialPathVerbRecipeV2::MoveTo { .. } | SpatialPathVerbRecipeV2::LineTo { .. } => 2,
        SpatialPathVerbRecipeV2::QuadraticTo { .. } => 4,
        SpatialPathVerbRecipeV2::CubicTo { .. } => 6,
        SpatialPathVerbRecipeV2::Close { .. } => 0,
    };
    1usize.checked_add(fields)
}

fn paint_count(paint: SpatialPaintRecipeV2) -> usize {
    match paint {
        SpatialPaintRecipeV2::CoveragePaint { coverage, clip, .. } => {
            1 + coverage_fields(coverage) + 2 + usize::from(clip.is_some()) * 2
        }
        SpatialPaintRecipeV2::ImagePaint { clip, .. } => 1 + 10 + usize::from(clip.is_some()) * 2,
    }
}

const fn coverage_fields(coverage: SpatialCoverageRecipeV2) -> usize {
    match coverage {
        SpatialCoverageRecipeV2::Fill { .. } => 1,
        SpatialCoverageRecipeV2::RoundStroke { .. } => 2,
    }
}

fn add(total: &mut usize, value: usize) -> Option<()> {
    *total = total.checked_add(value)?;
    Some(())
}

fn checked_mul(left: usize, right: usize) -> Option<usize> {
    left.checked_mul(right)
}
