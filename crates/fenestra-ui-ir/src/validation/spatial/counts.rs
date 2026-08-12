use crate::error::{IrValidationError, IrValidationErrorKind, ValidationLimitKind};
use crate::limits::SpatialValidationLimitsV2;
use crate::source::SourceSpan;
use crate::spatial::{SpatialBrushContentV2, SpatialProgramV2, SpatialShapeGeometryV2};

pub(super) fn preflight_spatial_counts(
    program: &SpatialProgramV2,
    limits: SpatialValidationLimitsV2,
) -> Result<(), IrValidationError> {
    let mut counts = [0usize; 13];
    for node in program.nodes() {
        add_count(
            &mut counts[0],
            limits.get(0),
            ValidationLimitKind::SpatialNodes,
            node.span(),
        )?;
    }
    for node in program.nodes() {
        for shape in node.shapes() {
            add_count(
                &mut counts[1],
                limits.get(1),
                ValidationLimitKind::SpatialShapes,
                shape.span(),
            )?;
        }
    }
    for node in program.nodes() {
        for brush in node.brushes() {
            add_count(
                &mut counts[2],
                limits.get(2),
                ValidationLimitKind::SpatialBrushes,
                brush.span(),
            )?;
        }
    }
    for node in program.nodes() {
        for clip in node.clips() {
            add_count(
                &mut counts[3],
                limits.get(3),
                ValidationLimitKind::SpatialClips,
                clip.span(),
            )?;
        }
    }
    for node in program.nodes() {
        for paint in node.paint_items() {
            add_count(
                &mut counts[4],
                limits.get(4),
                ValidationLimitKind::SpatialPaintItems,
                paint.span(),
            )?;
        }
    }
    for node in program.nodes() {
        for hit in node.hit_items() {
            add_count(
                &mut counts[5],
                limits.get(5),
                ValidationLimitKind::SpatialHitItems,
                hit.span(),
            )?;
        }
    }
    for node in program.nodes() {
        for semantic in node.semantic_items() {
            add_count(
                &mut counts[6],
                limits.get(6),
                ValidationLimitKind::SpatialSemanticItems,
                semantic.span(),
            )?;
        }
    }
    for node in program.nodes() {
        for shape in node.shapes() {
            if let SpatialShapeGeometryV2::Path { .. } = shape.geometry() {
                add_count(
                    &mut counts[7],
                    limits.get(7),
                    ValidationLimitKind::SpatialPaths,
                    shape.span(),
                )?;
            }
        }
    }
    for node in program.nodes() {
        for shape in node.shapes() {
            if let SpatialShapeGeometryV2::Path { verbs } = shape.geometry() {
                for verb in verbs {
                    add_count(
                        &mut counts[8],
                        limits.get(8),
                        ValidationLimitKind::SpatialPathVerbs,
                        verb.span(),
                    )?;
                }
            }
        }
    }
    for node in program.nodes() {
        for shape in node.shapes() {
            if let SpatialShapeGeometryV2::Polygon { points } = shape.geometry() {
                for point in points {
                    add_count(
                        &mut counts[9],
                        limits.get(9),
                        ValidationLimitKind::SpatialPolygonPoints,
                        point.span(),
                    )?;
                }
            }
        }
    }
    for node in program.nodes() {
        for brush in node.brushes() {
            if let SpatialBrushContentV2::LinearGradient { stops, .. } = brush.content() {
                for stop in stops {
                    add_count(
                        &mut counts[10],
                        limits.get(10),
                        ValidationLimitKind::SpatialGradientStops,
                        stop.span(),
                    )?;
                }
            }
        }
    }
    for image in program.images() {
        add_count(
            &mut counts[11],
            limits.get(11),
            ValidationLimitKind::SpatialImages,
            image.span(),
        )?;
    }
    for image in program.images() {
        add_amount(
            &mut counts[12],
            image.bytes().len(),
            limits.get(12),
            ValidationLimitKind::SpatialImageBytes,
            image.span(),
        )?;
    }
    Ok(())
}

fn add_count(
    count: &mut usize,
    limit: usize,
    kind: ValidationLimitKind,
    span: SourceSpan,
) -> Result<(), IrValidationError> {
    add_amount(count, 1, limit, kind, span)
}

fn add_amount(
    count: &mut usize,
    amount: usize,
    limit: usize,
    kind: ValidationLimitKind,
    span: SourceSpan,
) -> Result<(), IrValidationError> {
    *count = count
        .checked_add(amount)
        .ok_or_else(|| super::failure(IrValidationErrorKind::LimitExceeded(kind), span))?;
    if *count > limit {
        return Err(super::failure(
            IrValidationErrorKind::LimitExceeded(kind),
            span,
        ));
    }
    Ok(())
}
