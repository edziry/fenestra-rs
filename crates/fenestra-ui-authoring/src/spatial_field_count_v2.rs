use fenestra_ui_ir::prototype::{
    SpatialBrushContentV2, SpatialCoverageRecipeV2, SpatialNodeDeclarationV2, SpatialNodeParentV2,
    SpatialPaintRecipeV2, SpatialPathVerbRecipeV2, SpatialPlacementRecipeV2, SpatialProgramV2,
    SpatialShapeGeometryV2,
};

pub(crate) fn spatial_field_count(program: &SpatialProgramV2) -> usize {
    let mut count = 5;
    count += program.images().len() * 4;
    for node in program.nodes() {
        count += node_fields(node);
    }
    count
}

fn node_fields(node: &SpatialNodeDeclarationV2) -> usize {
    let mut count = 2 + usize::from(matches!(node.parent(), SpatialNodeParentV2::Node(_))) + 5;
    count += match node.placement() {
        SpatialPlacementRecipeV2::Layout(_) => 6 + 8,
        SpatialPlacementRecipeV2::Free(free) => {
            2 + usize::from(matches!(
                free.target(),
                fenestra_ui_ir::prototype::SpatialAnchorTargetRecipeV2::Node(_)
            )) + 2
                + 8
        }
    };
    for shape in node.shapes() {
        count += 1;
        count += match shape.geometry() {
            SpatialShapeGeometryV2::Rect { .. } => 4,
            SpatialShapeGeometryV2::Circle { .. } => 3,
            SpatialShapeGeometryV2::Polygon { points } => points.len() * 2,
            SpatialShapeGeometryV2::Path { verbs } => verbs.iter().map(verb_fields).sum(),
        };
    }
    for brush in node.brushes() {
        count += 1;
        count += match brush.content() {
            SpatialBrushContentV2::Solid { .. } => 1,
            SpatialBrushContentV2::LinearGradient { stops, .. } => 4 + stops.len() * 2,
        };
    }
    for clip in node.clips() {
        count += 2 + clip.parent().map_or(0, |_| 2);
    }
    for paint in node.paint_items() {
        count += match paint {
            SpatialPaintRecipeV2::CoveragePaint { coverage, clip, .. } => {
                coverage_fields(*coverage) + 2 + clip.map_or(0, |_| 2)
            }
            SpatialPaintRecipeV2::ImagePaint { clip, .. } => 10 + clip.map_or(0, |_| 2),
        };
    }
    for hit in node.hit_items() {
        count += coverage_fields(hit.coverage()) + hit.clip().map_or(0, |_| 2) + 1;
    }
    for semantic in node.semantic_items() {
        count += 1 + semantic.clip().map_or(0, |_| 2);
    }
    count
}

fn coverage_fields(value: SpatialCoverageRecipeV2) -> usize {
    match value {
        SpatialCoverageRecipeV2::Fill { .. } => 1,
        SpatialCoverageRecipeV2::RoundStroke { .. } => 2,
    }
}

fn verb_fields(value: &SpatialPathVerbRecipeV2) -> usize {
    match value {
        SpatialPathVerbRecipeV2::MoveTo { .. } | SpatialPathVerbRecipeV2::LineTo { .. } => 2,
        SpatialPathVerbRecipeV2::QuadraticTo { .. } => 4,
        SpatialPathVerbRecipeV2::CubicTo { .. } => 6,
        SpatialPathVerbRecipeV2::Close { .. } => 0,
    }
}
