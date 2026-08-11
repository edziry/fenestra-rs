use crate::*;

#[test]
fn geometry_vocabularies_are_closed_in_format_order() {
    assert_eq!(
        SpatialPathVerbKindV2::ALL,
        [
            SpatialPathVerbKindV2::MoveTo,
            SpatialPathVerbKindV2::LineTo,
            SpatialPathVerbKindV2::QuadraticTo,
            SpatialPathVerbKindV2::CubicTo,
            SpatialPathVerbKindV2::Close,
        ]
    );
    assert_eq!(
        SpatialShapeKindV2::ALL,
        [
            SpatialShapeKindV2::Rect,
            SpatialShapeKindV2::Circle,
            SpatialShapeKindV2::Polygon,
            SpatialShapeKindV2::Path,
        ]
    );
    assert_eq!(
        SpatialFillRuleV2::ALL,
        [SpatialFillRuleV2::NonZero, SpatialFillRuleV2::EvenOdd]
    );
    assert_eq!(
        SpatialCoverageKindV2::ALL,
        [
            SpatialCoverageKindV2::Fill,
            SpatialCoverageKindV2::RoundStroke
        ]
    );
}

#[test]
fn fieldless_vocabularies_remain_exhaustively_matchable() {
    for kind in SpatialPathVerbKindV2::ALL {
        match kind {
            SpatialPathVerbKindV2::MoveTo
            | SpatialPathVerbKindV2::LineTo
            | SpatialPathVerbKindV2::QuadraticTo
            | SpatialPathVerbKindV2::CubicTo
            | SpatialPathVerbKindV2::Close => {}
        }
    }
    for kind in SpatialShapeKindV2::ALL {
        match kind {
            SpatialShapeKindV2::Rect
            | SpatialShapeKindV2::Circle
            | SpatialShapeKindV2::Polygon
            | SpatialShapeKindV2::Path => {}
        }
    }
    for rule in SpatialFillRuleV2::ALL {
        match rule {
            SpatialFillRuleV2::NonZero | SpatialFillRuleV2::EvenOdd => {}
        }
    }
    for kind in SpatialCoverageKindV2::ALL {
        match kind {
            SpatialCoverageKindV2::Fill | SpatialCoverageKindV2::RoundStroke => {}
        }
    }
}

#[test]
fn payload_vocabularies_remain_exhaustively_matchable() {
    fn path_verb_tag(value: SpatialPathVerbV2) -> u8 {
        match value {
            SpatialPathVerbV2::MoveTo { .. } => 0,
            SpatialPathVerbV2::LineTo { .. } => 1,
            SpatialPathVerbV2::QuadraticTo { .. } => 2,
            SpatialPathVerbV2::CubicTo { .. } => 3,
            SpatialPathVerbV2::Close => 4,
        }
    }
    fn shape_tag(value: SpatialShapeGeometryV2) -> u8 {
        match value {
            SpatialShapeGeometryV2::Rect { .. } => 0,
            SpatialShapeGeometryV2::Circle { .. } => 1,
            SpatialShapeGeometryV2::Polygon { .. } => 2,
            SpatialShapeGeometryV2::Path { .. } => 3,
        }
    }
    fn coverage_tag(value: SpatialCoverageV2) -> u8 {
        match value {
            SpatialCoverageV2::Fill { .. } => 0,
            SpatialCoverageV2::RoundStroke { .. } => 1,
        }
    }

    let _ = (
        path_verb_tag as fn(SpatialPathVerbV2) -> u8,
        shape_tag as fn(SpatialShapeGeometryV2) -> u8,
        coverage_tag as fn(SpatialCoverageV2) -> u8,
    );
}
