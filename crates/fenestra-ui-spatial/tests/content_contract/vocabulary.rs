use crate::*;

#[test]
fn content_vocabularies_are_closed_in_format_order() {
    assert_eq!(
        SpatialBrushKindV2::ALL,
        [
            SpatialBrushKindV2::Solid,
            SpatialBrushKindV2::LinearGradient
        ]
    );
    assert_eq!(
        SpatialPaintKindV2::ALL,
        [
            SpatialPaintKindV2::CoveragePaint,
            SpatialPaintKindV2::ImagePaint,
        ]
    );
    assert_eq!(
        SpatialInputPolicyV2::ALL,
        [SpatialInputPolicyV2::Accept, SpatialInputPolicyV2::Ignore]
    );
}

#[test]
fn fieldless_content_vocabularies_remain_exhaustively_matchable() {
    for kind in SpatialBrushKindV2::ALL {
        match kind {
            SpatialBrushKindV2::Solid | SpatialBrushKindV2::LinearGradient => {}
        }
    }
    for kind in SpatialPaintKindV2::ALL {
        match kind {
            SpatialPaintKindV2::CoveragePaint | SpatialPaintKindV2::ImagePaint => {}
        }
    }
    for policy in SpatialInputPolicyV2::ALL {
        match policy {
            SpatialInputPolicyV2::Accept | SpatialInputPolicyV2::Ignore => {}
        }
    }
}

#[test]
fn content_payloads_remain_exhaustively_matchable() {
    fn brush_tag(value: SpatialBrushContentV2) -> u8 {
        match value {
            SpatialBrushContentV2::Solid { .. } => 0,
            SpatialBrushContentV2::LinearGradient { .. } => 1,
        }
    }
    fn paint_tag(value: SpatialPaintContentV2) -> u8 {
        match value {
            SpatialPaintContentV2::CoveragePaint { .. } => 0,
            SpatialPaintContentV2::ImagePaint { .. } => 1,
        }
    }

    let _ = (
        brush_tag as fn(SpatialBrushContentV2) -> u8,
        paint_tag as fn(SpatialPaintContentV2) -> u8,
    );
}
