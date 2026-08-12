use fenestra_ui_spatial::prototype::{
    Affine2V2, SpatialBrushKeyV2, SpatialClipKeyV2, SpatialImageKeyV2, SpatialNodeKeyV2,
    SpatialScalarV2, SpatialShapeKeyV2,
};

use crate::*;

#[test]
fn output_aabb_round_trips_malformed_raw_values_without_validation() {
    let empty = SpatialOutputAabbV2::new(
        true,
        scalar(i64::MIN),
        scalar(i64::MAX),
        scalar(-1),
        scalar(1),
    );
    assert!(empty.is_empty());
    assert_eq!(empty.min_x().raw(), i64::MIN);
    assert_eq!(empty.min_y().raw(), i64::MAX);
    assert_eq!(empty.max_x().raw(), -1);
    assert_eq!(empty.max_y().raw(), 1);

    let inverted = SpatialOutputAabbV2::new(false, scalar(40), scalar(30), scalar(20), scalar(10));
    assert!(!inverted.is_empty());
    assert_eq!(inverted.min_x().raw(), 40);
    assert_eq!(inverted.min_y().raw(), 30);
    assert_eq!(inverted.max_x().raw(), 20);
    assert_eq!(inverted.max_y().raw(), 10);
}

#[test]
fn geometry_and_clip_records_round_trip_every_raw_field() {
    let geometry = geometry_record(7);
    assert_eq!(geometry.key().get(), 7);
    assert_eq!(geometry.base_x().raw(), -11);
    assert_eq!(geometry.base_y().raw(), 12);
    assert_eq!(geometry.base_width().raw(), -13);
    assert_eq!(geometry.base_height().raw(), 14);
    assert_eq!(geometry.world_from_local(), affine(20));
    assert_eq!(geometry.world_determinant(), i128::MIN + 15);
    assert_eq!(geometry.world_aabb(), aabb(30));

    let clip = clip_record(8, Some(18));
    assert_eq!(clip.key().get(), 8);
    assert_eq!(clip.world_from_local(), affine(40));
    assert_eq!(clip.world_determinant(), i128::MAX - 16);
    assert_eq!(clip.primitive_world_aabb(), aabb(50));
    assert_eq!(clip.owner().get(), 17);
    assert_eq!(clip.parent().map(SpatialClipKeyV2::get), Some(18));
    assert_eq!(clip.shape().get(), 19);

    let parentless = clip_record(9, None);
    assert_eq!(parentless.key().get(), 9);
    assert_eq!(parentless.parent(), None);
}

#[test]
fn every_item_record_and_paint_reference_variant_round_trips() {
    let coverage_reference = SpatialPaintOutputReferenceV2::Coverage {
        shape: SpatialShapeKeyV2::new(21),
        brush: SpatialBrushKeyV2::new(22),
    };
    let coverage = paint_record(23, coverage_reference, Some(24));
    assert_paint(coverage, 23, Some(24));
    match coverage.reference() {
        SpatialPaintOutputReferenceV2::Coverage { shape, brush } => {
            assert_eq!(shape.get(), 21);
            assert_eq!(brush.get(), 22);
        }
        SpatialPaintOutputReferenceV2::Image { .. } => panic!("expected coverage reference"),
    }

    let image_reference = SpatialPaintOutputReferenceV2::Image {
        image: SpatialImageKeyV2::new(u32::MAX),
    };
    let image = paint_record(u32::MAX, image_reference, None);
    assert_paint(image, u32::MAX, None);
    match image.reference() {
        SpatialPaintOutputReferenceV2::Image { image } => assert_eq!(image.get(), u32::MAX),
        SpatialPaintOutputReferenceV2::Coverage { .. } => panic!("expected image reference"),
    }

    let hit = hit_record(25, None);
    assert_eq!(hit.key(), 25);
    assert_eq!(hit.world_from_local(), affine(60));
    assert_eq!(hit.world_determinant(), -26);
    assert_eq!(hit.world_aabb(), aabb(70));
    assert_eq!(hit.owner().get(), 27);
    assert_eq!(hit.shape().get(), 28);
    assert_eq!(hit.clip(), None);
    assert_eq!(hit.stack_ordinal(), 29);
    assert_eq!(hit.item_ordinal(), 30);

    let clipped_hit = hit_record(26, Some(42));
    assert_eq!(clipped_hit.clip().map(SpatialClipKeyV2::get), Some(42));

    let semantic = semantic_record(31, Some(35));
    assert_eq!(semantic.key(), 31);
    assert_eq!(semantic.world_from_local(), affine(80));
    assert_eq!(semantic.world_determinant(), 32);
    assert_eq!(semantic.world_aabb(), aabb(90));
    assert_eq!(semantic.owner().get(), 33);
    assert_eq!(semantic.shape().get(), 34);
    assert_eq!(semantic.clip().map(SpatialClipKeyV2::get), Some(35));
    assert_eq!(semantic.stack_ordinal(), 36);
    assert_eq!(semantic.item_ordinal(), 37);

    let unclipped_semantic = semantic_record(32, None);
    assert_eq!(unclipped_semantic.clip(), None);
}

#[test]
fn output_view_borrows_each_exact_nonempty_slice_in_table_order() {
    let geometry = [geometry_record(1), geometry_record(2)];
    let clips = [clip_record(3, Some(18))];
    let paints = [paint_record(
        4,
        SpatialPaintOutputReferenceV2::Image {
            image: SpatialImageKeyV2::new(5),
        },
        None,
    )];
    let hits = [
        hit_record(6, None),
        hit_record(7, Some(42)),
        hit_record(8, None),
    ];
    let semantics = [semantic_record(9, Some(35)), semantic_record(10, None)];
    let output = SpatialOutputV2::new(&geometry, &clips, &paints, &hits, &semantics);

    assert!(std::ptr::eq(output.geometry(), geometry.as_slice()));
    assert!(std::ptr::eq(output.clips(), clips.as_slice()));
    assert!(std::ptr::eq(output.paints(), paints.as_slice()));
    assert!(std::ptr::eq(output.hits(), hits.as_slice()));
    assert!(std::ptr::eq(output.semantics(), semantics.as_slice()));
    assert_eq!(output.geometry().len(), 2);
    assert_eq!(output.clips().len(), 1);
    assert_eq!(output.paints().len(), 1);
    assert_eq!(output.hits().len(), 3);
    assert_eq!(output.semantics().len(), 2);
}

fn geometry_record(key: u32) -> SpatialGeometryOutputRecordV2 {
    SpatialGeometryOutputRecordV2::new(
        SpatialNodeKeyV2::new(key),
        scalar(-11),
        scalar(12),
        scalar(-13),
        scalar(14),
        affine(20),
        i128::MIN + 15,
        aabb(30),
    )
}

fn clip_record(key: u32, parent: Option<u32>) -> SpatialClipOutputRecordV2 {
    SpatialClipOutputRecordV2::new(
        SpatialClipKeyV2::new(key),
        affine(40),
        i128::MAX - 16,
        aabb(50),
        SpatialNodeKeyV2::new(17),
        parent.map(SpatialClipKeyV2::new),
        SpatialShapeKeyV2::new(19),
    )
}

fn paint_record(
    key: u32,
    reference: SpatialPaintOutputReferenceV2,
    clip: Option<u32>,
) -> SpatialPaintOutputRecordV2 {
    SpatialPaintOutputRecordV2::new(
        key,
        affine(100),
        -38,
        aabb(110),
        SpatialNodeKeyV2::new(39),
        reference,
        clip.map(SpatialClipKeyV2::new),
        40,
        41,
    )
}

fn assert_paint(record: SpatialPaintOutputRecordV2, key: u32, clip: Option<u32>) {
    assert_eq!(record.key(), key);
    assert_eq!(record.world_from_local(), affine(100));
    assert_eq!(record.world_determinant(), -38);
    assert_eq!(record.world_aabb(), aabb(110));
    assert_eq!(record.owner().get(), 39);
    assert_eq!(record.clip().map(SpatialClipKeyV2::get), clip);
    assert_eq!(record.stack_ordinal(), 40);
    assert_eq!(record.item_ordinal(), 41);
}

fn hit_record(key: u32, clip: Option<u32>) -> SpatialHitOutputRecordV2 {
    SpatialHitOutputRecordV2::new(
        key,
        affine(60),
        -26,
        aabb(70),
        SpatialNodeKeyV2::new(27),
        SpatialShapeKeyV2::new(28),
        clip.map(SpatialClipKeyV2::new),
        29,
        30,
    )
}

fn semantic_record(key: u32, clip: Option<u32>) -> SpatialSemanticOutputRecordV2 {
    SpatialSemanticOutputRecordV2::new(
        key,
        affine(80),
        32,
        aabb(90),
        SpatialNodeKeyV2::new(33),
        SpatialShapeKeyV2::new(34),
        clip.map(SpatialClipKeyV2::new),
        36,
        37,
    )
}

fn affine(seed: i64) -> Affine2V2 {
    Affine2V2::new(
        scalar(seed),
        scalar(seed + 1),
        scalar(seed + 2),
        scalar(seed + 3),
        scalar(seed + 4),
        scalar(seed + 5),
    )
}

fn aabb(seed: i64) -> SpatialOutputAabbV2 {
    SpatialOutputAabbV2::new(
        seed % 2 == 0,
        scalar(seed),
        scalar(seed + 1),
        scalar(seed - 1),
        scalar(seed - 2),
    )
}

fn scalar(raw: i64) -> SpatialScalarV2 {
    SpatialScalarV2::new(raw)
}
