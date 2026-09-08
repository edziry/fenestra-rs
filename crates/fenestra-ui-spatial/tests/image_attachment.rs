use fenestra_ui_layout::prototype::{
    LayoutEngineErrorV1, LayoutEngineV1, LayoutOutputV1, LayoutRecordV1, LayoutRectV1,
    ReferenceStackEngineV1, ValidatedLayoutInputV1,
};
use fenestra_ui_spatial::prototype::{
    REGISTERED_SPATIAL_LIMITS_V2, ReferenceRasterLimitsV2, SpatialClipKeyV2,
    SpatialImageAttachmentErrorV2, SpatialImageDestinationRectV2, SpatialImageKeyV2,
    SpatialImagePaintAttachmentV2, SpatialImageV2, SpatialLimitKindV2, SpatialLimitsV2,
    SpatialNodeKeyV2, SpatialPaintContentV2, SpatialResolvedSnapshotV2, SpatialScalarV2,
    resolve_spatial_v2,
};

#[path = "image_attachment/fixture.rs"]
mod fixture;

fn attachment(
    owner: u32,
    rgba: [u8; 4],
    rect: [i32; 4],
    clip: Option<u32>,
) -> SpatialImagePaintAttachmentV2 {
    SpatialImagePaintAttachmentV2::new(
        SpatialNodeKeyV2::new(owner),
        SpatialImageV2::new(SpatialImageKeyV2::new(999), 1, 1, 4, Box::new(rgba)),
        SpatialImageDestinationRectV2::new(
            scalar(rect[0]),
            scalar(rect[1]),
            scalar(rect[2]),
            scalar(rect[3]),
        ),
        clip.map(SpatialClipKeyV2::new),
    )
}

fn scalar(value: i32) -> SpatialScalarV2 {
    SpatialScalarV2::new(i64::from(value) * 65_536)
}

fn pixels(snapshot: &SpatialResolvedSnapshotV2) -> Vec<u8> {
    snapshot
        .rasterize_reference(ReferenceRasterLimitsV2::new(8))
        .unwrap()
        .bytes()
        .to_vec()
}

#[test]
fn attachment_enters_owner_paint_order_before_overlapping_children_and_siblings() {
    let original = fixture::resolved();
    let before = pixels(&original);
    let augmented = original
        .with_image_paints(
            Box::new([attachment(1, [255; 4], [0, 0, 4, 2], None)]),
            REGISTERED_SPATIAL_LIMITS_V2,
        )
        .unwrap();

    assert_eq!(
        &pixels(&augmented)[..16],
        &[
            0, 0, 200, 255, 0, 64, 100, 255, 127, 191, 127, 255, 255, 255, 255, 255,
        ]
    );
    assert_eq!(pixels(&original), before);
    assert_eq!(original.output().geometry(), augmented.output().geometry());
    assert_eq!(original.output().clips(), augmented.output().clips());
    assert_eq!(original.output().hits(), augmented.output().hits());
    assert_eq!(
        original.output().semantics(),
        augmented.output().semantics()
    );
    assert_eq!(original.viewport(), augmented.viewport());
    let headers: Vec<_> = augmented
        .paint_frame()
        .paint_items()
        .iter()
        .map(|paint| (paint.owner().get(), paint.item_ordinal()))
        .collect();
    assert_eq!(headers, [(1, 0), (1, 1), (2, 0), (3, 0)]);
    assert_eq!(augmented.paint_frame().images()[0].key().get(), 0);
    drop(original);
    assert_eq!(&pixels(&augmented)[12..16], &[255; 4]);
}

#[test]
fn same_owner_order_and_transparency_survive_unsorted_attachment_owners() {
    let original = fixture::resolved();
    let augmented = original
        .with_image_paints(
            Box::new([
                attachment(3, [128, 0, 0, 128], [0, 0, 1, 1], None),
                attachment(1, [255; 4], [0, 0, 4, 2], None),
                attachment(3, [0, 0, 0, 0], [0, 0, 1, 1], None),
            ]),
            REGISTERED_SPATIAL_LIMITS_V2,
        )
        .unwrap();
    assert_eq!(&pixels(&augmented)[4..8], &[128, 32, 50, 255]);
    let headers: Vec<_> = augmented
        .paint_frame()
        .paint_items()
        .iter()
        .map(|paint| (paint.owner().get(), paint.item_ordinal()))
        .collect();
    assert_eq!(headers, [(1, 0), (1, 1), (2, 0), (3, 0), (3, 1), (3, 2)]);
}

#[test]
fn attachments_obey_ancestor_clips_and_negative_offscreen_destinations() {
    let original = fixture::resolved();
    let augmented = original
        .with_image_paints(
            Box::new([
                attachment(2, [255, 255, 0, 255], [-1, -1, 4, 4], Some(0)),
                attachment(1, [255; 4], [40, 40, 2, 2], None),
            ]),
            REGISTERED_SPATIAL_LIMITS_V2,
        )
        .unwrap();
    assert_eq!(
        &pixels(&augmented)[..12],
        &[255, 255, 0, 255, 0, 64, 100, 255, 100, 64, 0, 255]
    );
    assert_eq!(
        &pixels(&augmented)[16..24],
        &[255, 255, 0, 255, 200, 0, 0, 255]
    );
}

#[test]
fn repeated_attachment_keeps_existing_images_and_appends_dense_keys() {
    let first = fixture::resolved()
        .with_image_paints(
            Box::new([attachment(2, [255; 4], [0, 0, 1, 1], None)]),
            REGISTERED_SPATIAL_LIMITS_V2,
        )
        .unwrap();
    let second = first
        .with_image_paints(
            Box::new([attachment(2, [0, 255, 0, 255], [0, 0, 1, 1], None)]),
            REGISTERED_SPATIAL_LIMITS_V2,
        )
        .unwrap();
    assert_eq!(second.paint_frame().images().len(), 2);
    assert_eq!(second.paint_frame().images()[1].key().get(), 1);
    assert_eq!(&pixels(&second)[..4], &[0, 255, 0, 255]);
    assert!(
        matches!(second.paint_frame().paint_items()[3].content(), SpatialPaintContentV2::ImagePaint { image, .. } if image.get() == 1)
    );
}

#[test]
fn invalid_attachment_is_rejected_without_changing_the_original() {
    let original = fixture::resolved();
    let before = pixels(&original);
    for invalid in [
        attachment(0, [255; 4], [0, 0, 1, 1], None),
        attachment(4, [255; 4], [0, 0, 1, 1], None),
        attachment(1, [255, 0, 0, 10], [0, 0, 1, 1], None),
        attachment(1, [255; 4], [0, 0, 0, 1], None),
        attachment(1, [255; 4], [0, 0, 1, 1], Some(99)),
    ] {
        assert!(
            original
                .with_image_paints(Box::new([invalid]), REGISTERED_SPATIAL_LIMITS_V2)
                .is_err()
        );
        assert_eq!(pixels(&original), before);
    }
}

#[test]
fn aggregate_image_and_paint_budgets_are_inclusive() {
    let original = fixture::resolved();
    for (kind, limit) in [
        (SpatialLimitKindV2::Images, 0),
        (SpatialLimitKindV2::ImagePixelsTotal, 0),
        (SpatialLimitKindV2::ImageEdge, 0),
        (SpatialLimitKindV2::PaintItems, 3),
        (SpatialLimitKindV2::PaintItemsPerNode, 1),
    ] {
        let mut limits =
            SpatialLimitKindV2::ALL.map(|kind| REGISTERED_SPATIAL_LIMITS_V2.limit(kind));
        limits[SpatialLimitKindV2::ALL
            .iter()
            .position(|value| *value == kind)
            .unwrap()] = limit;
        assert!(
            original
                .with_image_paints(
                    Box::new([attachment(1, [255; 4], [0, 0, 1, 1], None)]),
                    SpatialLimitsV2::new(limits),
                )
                .is_err()
        );
    }
    let mut limits = SpatialLimitKindV2::ALL.map(|kind| REGISTERED_SPATIAL_LIMITS_V2.limit(kind));
    for (kind, value) in [
        (SpatialLimitKindV2::Images, 1),
        (SpatialLimitKindV2::ImagePixelsTotal, 1),
        (SpatialLimitKindV2::PaintItems, 4),
        (SpatialLimitKindV2::PaintItemsPerNode, 2),
    ] {
        limits[SpatialLimitKindV2::ALL
            .iter()
            .position(|entry| *entry == kind)
            .unwrap()] = value;
    }
    assert!(
        original
            .with_image_paints(
                Box::new([attachment(1, [255; 4], [0, 0, 1, 1], None)]),
                SpatialLimitsV2::new(limits)
            )
            .is_ok()
    );
}

struct ShiftedLayout;

impl LayoutEngineV1 for ShiftedLayout {
    fn compute(
        &self,
        input: ValidatedLayoutInputV1<'_>,
    ) -> Result<LayoutOutputV1, LayoutEngineErrorV1> {
        let reference = ReferenceStackEngineV1::new().compute(input)?;
        Ok(LayoutOutputV1::new(
            reference
                .records()
                .iter()
                .map(|record| {
                    let bounds = record.bounds();
                    LayoutRecordV1::new(
                        record.key(),
                        LayoutRectV1::new(
                            bounds.x() + i32::from(record.key().get() != 0),
                            bounds.y(),
                            bounds.width(),
                            bounds.height(),
                        ),
                    )
                })
                .collect(),
        ))
    }
}

#[test]
fn attachment_rejects_reresolution_that_changes_accepted_geometry() {
    let original = resolve_spatial_v2(
        &ShiftedLayout,
        fixture::source(),
        REGISTERED_SPATIAL_LIMITS_V2,
    )
    .unwrap();
    let result = original.with_image_paints(
        Box::new([attachment(1, [255; 4], [0, 0, 1, 1], None)]),
        REGISTERED_SPATIAL_LIMITS_V2,
    );
    assert!(matches!(
        result,
        Err(SpatialImageAttachmentErrorV2::GeometryChanged)
    ));
}
