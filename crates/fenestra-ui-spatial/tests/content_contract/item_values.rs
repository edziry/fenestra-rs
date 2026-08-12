use fenestra_ui_spatial::prototype::{
    SpatialClipKeyV2, SpatialCoverageV2, SpatialFillRuleV2, SpatialNodeKeyV2, SpatialScalarV2,
    SpatialShapeKeyV2,
};

use crate::*;

#[test]
fn image_rectangles_round_trip_raw_coordinates_without_validation() {
    let source = SpatialImageSourceRectV2::new(1, 2, 3, 4);
    assert_eq!(source.x(), 1);
    assert_eq!(source.y(), 2);
    assert_eq!(source.width(), 3);
    assert_eq!(source.height(), 4);

    let destination = SpatialImageDestinationRectV2::new(
        scalar(i64::MIN),
        scalar(i64::MAX),
        scalar(-1),
        scalar(0),
    );
    assert_eq!(destination.x().raw(), i64::MIN);
    assert_eq!(destination.y().raw(), i64::MAX);
    assert_eq!(destination.width().raw(), -1);
    assert_eq!(destination.height().raw(), 0);
}

#[test]
fn every_paint_payload_and_record_round_trips_independently() {
    let coverage = SpatialPaintContentV2::CoveragePaint {
        coverage: SpatialCoverageV2::RoundStroke {
            shape: SpatialShapeKeyV2::new(1),
            width: scalar(-2),
        },
        brush: SpatialBrushKeyV2::new(u32::MAX),
        opacity: 0,
        clip: Some(SpatialClipKeyV2::new(u32::MAX)),
    };
    let image = SpatialPaintContentV2::ImagePaint {
        image: SpatialImageKeyV2::new(3),
        source: SpatialImageSourceRectV2::new(4, 5, 0, 0),
        destination: SpatialImageDestinationRectV2::new(
            scalar(6),
            scalar(7),
            scalar(-8),
            scalar(-9),
        ),
        opacity: u8::MAX,
        clip: None,
    };

    match coverage {
        SpatialPaintContentV2::CoveragePaint {
            coverage,
            brush,
            opacity,
            clip,
        } => {
            assert!(matches!(coverage, SpatialCoverageV2::RoundStroke { .. }));
            assert_eq!(brush.get(), u32::MAX);
            assert_eq!(opacity, 0);
            assert_eq!(clip.map(SpatialClipKeyV2::get), Some(u32::MAX));
        }
        _ => panic!("expected coverage paint"),
    }
    match image {
        SpatialPaintContentV2::ImagePaint {
            image,
            source,
            destination,
            opacity,
            clip,
        } => {
            assert_eq!(image.get(), 3);
            assert_eq!(source, SpatialImageSourceRectV2::new(4, 5, 0, 0));
            assert_eq!(destination.width().raw(), -8);
            assert_eq!(opacity, u8::MAX);
            assert_eq!(clip, None);
        }
        _ => panic!("expected image paint"),
    }

    for (ordinal, content) in [coverage, image].into_iter().enumerate() {
        let owner = SpatialNodeKeyV2::new(if ordinal == 0 { 0 } else { u32::MAX });
        let paint = SpatialPaintV2::new(owner, u32::MAX - ordinal as u32, content);
        assert_eq!(paint.owner(), owner);
        assert_eq!(paint.item_ordinal(), u32::MAX - ordinal as u32);
        assert_eq!(paint.content(), content);
    }
}

#[test]
fn hit_and_semantic_records_keep_independent_geometry_and_clips() {
    let coverage = SpatialCoverageV2::Fill {
        shape: SpatialShapeKeyV2::new(10),
        rule: SpatialFillRuleV2::EvenOdd,
    };
    let hit = SpatialHitV2::new(
        SpatialNodeKeyV2::new(0),
        u32::MAX,
        coverage,
        Some(SpatialClipKeyV2::new(11)),
        SpatialInputPolicyV2::Ignore,
    );
    assert_eq!(hit.owner().get(), 0);
    assert_eq!(hit.item_ordinal(), u32::MAX);
    assert_eq!(hit.coverage(), coverage);
    assert_eq!(hit.clip().map(SpatialClipKeyV2::get), Some(11));
    assert_eq!(hit.input_policy(), SpatialInputPolicyV2::Ignore);

    let alternate_coverage = SpatialCoverageV2::RoundStroke {
        shape: SpatialShapeKeyV2::new(14),
        width: scalar(-15),
    };
    let accepting_hit = SpatialHitV2::new(
        SpatialNodeKeyV2::new(16),
        17,
        alternate_coverage,
        None,
        SpatialInputPolicyV2::Accept,
    );
    assert_eq!(accepting_hit.owner().get(), 16);
    assert_eq!(accepting_hit.item_ordinal(), 17);
    assert_eq!(accepting_hit.coverage(), alternate_coverage);
    assert_eq!(accepting_hit.clip(), None);
    assert_eq!(accepting_hit.input_policy(), SpatialInputPolicyV2::Accept);

    let semantic = SpatialSemanticGeometryV2::new(
        SpatialNodeKeyV2::new(12),
        0,
        SpatialShapeKeyV2::new(13),
        SpatialFillRuleV2::NonZero,
        None,
    );
    assert_eq!(semantic.owner().get(), 12);
    assert_eq!(semantic.item_ordinal(), 0);
    assert_eq!(semantic.shape().get(), 13);
    assert_eq!(semantic.fill_rule(), SpatialFillRuleV2::NonZero);
    assert_eq!(semantic.clip(), None);

    let clipped_semantic = SpatialSemanticGeometryV2::new(
        SpatialNodeKeyV2::new(0),
        u32::MAX,
        SpatialShapeKeyV2::new(18),
        SpatialFillRuleV2::EvenOdd,
        Some(SpatialClipKeyV2::new(u32::MAX)),
    );
    assert_eq!(clipped_semantic.owner().get(), 0);
    assert_eq!(clipped_semantic.item_ordinal(), u32::MAX);
    assert_eq!(clipped_semantic.shape().get(), 18);
    assert_eq!(clipped_semantic.fill_rule(), SpatialFillRuleV2::EvenOdd);
    assert_eq!(
        clipped_semantic.clip().map(SpatialClipKeyV2::get),
        Some(u32::MAX)
    );
}

fn scalar(raw: i64) -> SpatialScalarV2 {
    SpatialScalarV2::new(raw)
}
