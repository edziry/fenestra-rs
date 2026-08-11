use crate::*;

#[test]
fn field_vocabularies_have_exact_registered_all_arrays() {
    let _: [SpatialColorChannelV2; 4] = SpatialColorChannelV2::ALL;
    assert_eq!(
        SpatialColorChannelV2::ALL,
        [
            SpatialColorChannelV2::R,
            SpatialColorChannelV2::G,
            SpatialColorChannelV2::B,
            SpatialColorChannelV2::A,
        ]
    );
    assert_eq!(
        SpatialPathFieldV2::ALL,
        [
            SpatialPathFieldV2::Key,
            SpatialPathFieldV2::VerbStart,
            SpatialPathFieldV2::VerbLength,
        ]
    );
    assert_eq!(
        SpatialPathVerbFieldV2::ALL,
        [
            SpatialPathVerbFieldV2::Kind,
            SpatialPathVerbFieldV2::ControlX,
            SpatialPathVerbFieldV2::ControlY,
            SpatialPathVerbFieldV2::Control1X,
            SpatialPathVerbFieldV2::Control1Y,
            SpatialPathVerbFieldV2::Control2X,
            SpatialPathVerbFieldV2::Control2Y,
            SpatialPathVerbFieldV2::ToX,
            SpatialPathVerbFieldV2::ToY,
        ]
    );
    assert_eq!(
        SpatialShapeFieldV2::ALL,
        [
            SpatialShapeFieldV2::Key,
            SpatialShapeFieldV2::Owner,
            SpatialShapeFieldV2::Kind,
            SpatialShapeFieldV2::RectX,
            SpatialShapeFieldV2::RectY,
            SpatialShapeFieldV2::RectWidth,
            SpatialShapeFieldV2::RectHeight,
            SpatialShapeFieldV2::CircleCenterX,
            SpatialShapeFieldV2::CircleCenterY,
            SpatialShapeFieldV2::CircleRadius,
            SpatialShapeFieldV2::PolygonPointStart,
            SpatialShapeFieldV2::PolygonPointLength,
            SpatialShapeFieldV2::Path,
        ]
    );
    assert_eq!(
        SpatialPolygonPointFieldV2::ALL,
        [SpatialPolygonPointFieldV2::X, SpatialPolygonPointFieldV2::Y]
    );
    assert_eq!(
        SpatialBrushFieldV2::ALL,
        [
            SpatialBrushFieldV2::Key,
            SpatialBrushFieldV2::Kind,
            SpatialBrushFieldV2::GradientStopStart,
            SpatialBrushFieldV2::GradientStopLength,
            SpatialBrushFieldV2::ColorR,
            SpatialBrushFieldV2::ColorG,
            SpatialBrushFieldV2::ColorB,
            SpatialBrushFieldV2::ColorA,
            SpatialBrushFieldV2::GradientStartX,
            SpatialBrushFieldV2::GradientStartY,
            SpatialBrushFieldV2::GradientEndX,
            SpatialBrushFieldV2::GradientEndY,
        ]
    );
    assert_eq!(
        SpatialGradientStopFieldV2::ALL,
        [
            SpatialGradientStopFieldV2::Offset,
            SpatialGradientStopFieldV2::R,
            SpatialGradientStopFieldV2::G,
            SpatialGradientStopFieldV2::B,
            SpatialGradientStopFieldV2::A,
        ]
    );
    assert_eq!(
        SpatialImageFieldV2::ALL,
        [
            SpatialImageFieldV2::Key,
            SpatialImageFieldV2::Width,
            SpatialImageFieldV2::Height,
            SpatialImageFieldV2::Stride,
            SpatialImageFieldV2::ByteLength,
            SpatialImageFieldV2::Pixel,
        ]
    );
    assert_eq!(
        SpatialClipFieldV2::ALL,
        [
            SpatialClipFieldV2::Key,
            SpatialClipFieldV2::Owner,
            SpatialClipFieldV2::Parent,
            SpatialClipFieldV2::Shape,
            SpatialClipFieldV2::FillRule,
        ]
    );
    assert_eq!(
        SpatialPaintFieldV2::ALL,
        [
            SpatialPaintFieldV2::Owner,
            SpatialPaintFieldV2::ItemOrdinal,
            SpatialPaintFieldV2::Kind,
            SpatialPaintFieldV2::Image,
            SpatialPaintFieldV2::SourceX,
            SpatialPaintFieldV2::SourceY,
            SpatialPaintFieldV2::SourceWidth,
            SpatialPaintFieldV2::SourceHeight,
            SpatialPaintFieldV2::DestinationX,
            SpatialPaintFieldV2::DestinationY,
            SpatialPaintFieldV2::DestinationWidth,
            SpatialPaintFieldV2::DestinationHeight,
            SpatialPaintFieldV2::CoverageKind,
            SpatialPaintFieldV2::Shape,
            SpatialPaintFieldV2::FillRule,
            SpatialPaintFieldV2::StrokeWidth,
            SpatialPaintFieldV2::Brush,
            SpatialPaintFieldV2::Opacity,
            SpatialPaintFieldV2::Clip,
        ]
    );
    assert_eq!(
        SpatialHitFieldV2::ALL,
        [
            SpatialHitFieldV2::Owner,
            SpatialHitFieldV2::ItemOrdinal,
            SpatialHitFieldV2::CoverageKind,
            SpatialHitFieldV2::Shape,
            SpatialHitFieldV2::FillRule,
            SpatialHitFieldV2::StrokeWidth,
            SpatialHitFieldV2::Clip,
            SpatialHitFieldV2::InputPolicy,
        ]
    );
    assert_eq!(
        SpatialSemanticFieldV2::ALL,
        [
            SpatialSemanticFieldV2::Owner,
            SpatialSemanticFieldV2::ItemOrdinal,
            SpatialSemanticFieldV2::Shape,
            SpatialSemanticFieldV2::FillRule,
            SpatialSemanticFieldV2::Clip,
        ]
    );
    assert_eq!(
        SpatialOutputTableV2::ALL,
        [
            SpatialOutputTableV2::Geometry,
            SpatialOutputTableV2::Clip,
            SpatialOutputTableV2::Paint,
            SpatialOutputTableV2::Hit,
            SpatialOutputTableV2::Semantic,
        ]
    );
    assert_eq!(SpatialOutputFieldV2::ALL, expected_output_fields());
}

fn expected_output_fields() -> [SpatialOutputFieldV2; 25] {
    [
        SpatialOutputFieldV2::Key,
        SpatialOutputFieldV2::BaseX,
        SpatialOutputFieldV2::BaseY,
        SpatialOutputFieldV2::BaseWidth,
        SpatialOutputFieldV2::BaseHeight,
        SpatialOutputFieldV2::AffineA,
        SpatialOutputFieldV2::AffineB,
        SpatialOutputFieldV2::AffineC,
        SpatialOutputFieldV2::AffineD,
        SpatialOutputFieldV2::AffineTx,
        SpatialOutputFieldV2::AffineTy,
        SpatialOutputFieldV2::Determinant,
        SpatialOutputFieldV2::AabbEmpty,
        SpatialOutputFieldV2::AabbMinX,
        SpatialOutputFieldV2::AabbMinY,
        SpatialOutputFieldV2::AabbMaxX,
        SpatialOutputFieldV2::AabbMaxY,
        SpatialOutputFieldV2::Owner,
        SpatialOutputFieldV2::Parent,
        SpatialOutputFieldV2::Shape,
        SpatialOutputFieldV2::Brush,
        SpatialOutputFieldV2::Image,
        SpatialOutputFieldV2::Clip,
        SpatialOutputFieldV2::StackOrdinal,
        SpatialOutputFieldV2::ItemOrdinal,
    ]
}

#[test]
fn field_vocabularies_remain_exhaustively_matchable() {
    for value in SpatialColorChannelV2::ALL {
        match value {
            SpatialColorChannelV2::R
            | SpatialColorChannelV2::G
            | SpatialColorChannelV2::B
            | SpatialColorChannelV2::A => {}
        }
    }
    for value in SpatialPathFieldV2::ALL {
        match value {
            SpatialPathFieldV2::Key
            | SpatialPathFieldV2::VerbStart
            | SpatialPathFieldV2::VerbLength => {}
        }
    }
    for value in SpatialPathVerbFieldV2::ALL {
        match value {
            SpatialPathVerbFieldV2::Kind
            | SpatialPathVerbFieldV2::ControlX
            | SpatialPathVerbFieldV2::ControlY
            | SpatialPathVerbFieldV2::Control1X
            | SpatialPathVerbFieldV2::Control1Y
            | SpatialPathVerbFieldV2::Control2X
            | SpatialPathVerbFieldV2::Control2Y
            | SpatialPathVerbFieldV2::ToX
            | SpatialPathVerbFieldV2::ToY => {}
        }
    }
    for value in SpatialShapeFieldV2::ALL {
        match value {
            SpatialShapeFieldV2::Key
            | SpatialShapeFieldV2::Owner
            | SpatialShapeFieldV2::Kind
            | SpatialShapeFieldV2::RectX
            | SpatialShapeFieldV2::RectY
            | SpatialShapeFieldV2::RectWidth
            | SpatialShapeFieldV2::RectHeight
            | SpatialShapeFieldV2::CircleCenterX
            | SpatialShapeFieldV2::CircleCenterY
            | SpatialShapeFieldV2::CircleRadius
            | SpatialShapeFieldV2::PolygonPointStart
            | SpatialShapeFieldV2::PolygonPointLength
            | SpatialShapeFieldV2::Path => {}
        }
    }
    for value in SpatialPolygonPointFieldV2::ALL {
        match value {
            SpatialPolygonPointFieldV2::X | SpatialPolygonPointFieldV2::Y => {}
        }
    }
    assert_remaining_fields_exhaustive();
}

fn assert_remaining_fields_exhaustive() {
    for value in SpatialBrushFieldV2::ALL {
        match value {
            SpatialBrushFieldV2::Key
            | SpatialBrushFieldV2::Kind
            | SpatialBrushFieldV2::GradientStopStart
            | SpatialBrushFieldV2::GradientStopLength
            | SpatialBrushFieldV2::ColorR
            | SpatialBrushFieldV2::ColorG
            | SpatialBrushFieldV2::ColorB
            | SpatialBrushFieldV2::ColorA
            | SpatialBrushFieldV2::GradientStartX
            | SpatialBrushFieldV2::GradientStartY
            | SpatialBrushFieldV2::GradientEndX
            | SpatialBrushFieldV2::GradientEndY => {}
        }
    }
    for value in SpatialGradientStopFieldV2::ALL {
        match value {
            SpatialGradientStopFieldV2::Offset
            | SpatialGradientStopFieldV2::R
            | SpatialGradientStopFieldV2::G
            | SpatialGradientStopFieldV2::B
            | SpatialGradientStopFieldV2::A => {}
        }
    }
    for value in SpatialImageFieldV2::ALL {
        match value {
            SpatialImageFieldV2::Key
            | SpatialImageFieldV2::Width
            | SpatialImageFieldV2::Height
            | SpatialImageFieldV2::Stride
            | SpatialImageFieldV2::ByteLength
            | SpatialImageFieldV2::Pixel => {}
        }
    }
    for value in SpatialClipFieldV2::ALL {
        match value {
            SpatialClipFieldV2::Key
            | SpatialClipFieldV2::Owner
            | SpatialClipFieldV2::Parent
            | SpatialClipFieldV2::Shape
            | SpatialClipFieldV2::FillRule => {}
        }
    }
    for value in SpatialPaintFieldV2::ALL {
        match value {
            SpatialPaintFieldV2::Owner
            | SpatialPaintFieldV2::ItemOrdinal
            | SpatialPaintFieldV2::Kind
            | SpatialPaintFieldV2::Image
            | SpatialPaintFieldV2::SourceX
            | SpatialPaintFieldV2::SourceY
            | SpatialPaintFieldV2::SourceWidth
            | SpatialPaintFieldV2::SourceHeight
            | SpatialPaintFieldV2::DestinationX
            | SpatialPaintFieldV2::DestinationY
            | SpatialPaintFieldV2::DestinationWidth
            | SpatialPaintFieldV2::DestinationHeight
            | SpatialPaintFieldV2::CoverageKind
            | SpatialPaintFieldV2::Shape
            | SpatialPaintFieldV2::FillRule
            | SpatialPaintFieldV2::StrokeWidth
            | SpatialPaintFieldV2::Brush
            | SpatialPaintFieldV2::Opacity
            | SpatialPaintFieldV2::Clip => {}
        }
    }
    for value in SpatialHitFieldV2::ALL {
        match value {
            SpatialHitFieldV2::Owner
            | SpatialHitFieldV2::ItemOrdinal
            | SpatialHitFieldV2::CoverageKind
            | SpatialHitFieldV2::Shape
            | SpatialHitFieldV2::FillRule
            | SpatialHitFieldV2::StrokeWidth
            | SpatialHitFieldV2::Clip
            | SpatialHitFieldV2::InputPolicy => {}
        }
    }
    for value in SpatialSemanticFieldV2::ALL {
        match value {
            SpatialSemanticFieldV2::Owner
            | SpatialSemanticFieldV2::ItemOrdinal
            | SpatialSemanticFieldV2::Shape
            | SpatialSemanticFieldV2::FillRule
            | SpatialSemanticFieldV2::Clip => {}
        }
    }
    for value in SpatialOutputTableV2::ALL {
        match value {
            SpatialOutputTableV2::Geometry
            | SpatialOutputTableV2::Clip
            | SpatialOutputTableV2::Paint
            | SpatialOutputTableV2::Hit
            | SpatialOutputTableV2::Semantic => {}
        }
    }
    for value in SpatialOutputFieldV2::ALL {
        match value {
            SpatialOutputFieldV2::Key
            | SpatialOutputFieldV2::BaseX
            | SpatialOutputFieldV2::BaseY
            | SpatialOutputFieldV2::BaseWidth
            | SpatialOutputFieldV2::BaseHeight
            | SpatialOutputFieldV2::AffineA
            | SpatialOutputFieldV2::AffineB
            | SpatialOutputFieldV2::AffineC
            | SpatialOutputFieldV2::AffineD
            | SpatialOutputFieldV2::AffineTx
            | SpatialOutputFieldV2::AffineTy
            | SpatialOutputFieldV2::Determinant
            | SpatialOutputFieldV2::AabbEmpty
            | SpatialOutputFieldV2::AabbMinX
            | SpatialOutputFieldV2::AabbMinY
            | SpatialOutputFieldV2::AabbMaxX
            | SpatialOutputFieldV2::AabbMaxY
            | SpatialOutputFieldV2::Owner
            | SpatialOutputFieldV2::Parent
            | SpatialOutputFieldV2::Shape
            | SpatialOutputFieldV2::Brush
            | SpatialOutputFieldV2::Image
            | SpatialOutputFieldV2::Clip
            | SpatialOutputFieldV2::StackOrdinal
            | SpatialOutputFieldV2::ItemOrdinal => {}
        }
    }
}
