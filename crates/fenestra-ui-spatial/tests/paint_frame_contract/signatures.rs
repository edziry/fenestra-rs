use fenestra_ui_spatial::prototype::{
    ReferenceRasterErrorV2, ReferenceRasterLimitsV2, ReferenceRasterV2, SpatialAabbV2,
    SpatialBrushV2, SpatialClipOutputRecordV2, SpatialClipV2, SpatialGradientStopV2,
    SpatialImageV2, SpatialPaintOutputRecordV2, SpatialPaintV2, SpatialPathV2, SpatialPathVerbV2,
    SpatialPointV2, SpatialShapeV2, SpatialViewportV2,
};

use crate::{SpatialPaintFrameV2, SpatialResolvedSnapshotV2};

#[test]
fn paint_frame_signatures_are_exact() {
    assert_signatures(&());
}

fn assert_signatures<'a>(_: &'a ()) {
    let _: fn(&'a SpatialResolvedSnapshotV2) -> SpatialPaintFrameV2<'a> =
        SpatialResolvedSnapshotV2::paint_frame;
    let _: fn(SpatialPaintFrameV2<'a>) -> SpatialViewportV2 = SpatialPaintFrameV2::viewport;
    let _: fn(SpatialPaintFrameV2<'a>) -> &'a [SpatialPointV2] =
        SpatialPaintFrameV2::polygon_points;
    let _: fn(SpatialPaintFrameV2<'a>) -> &'a [SpatialPathVerbV2] = SpatialPaintFrameV2::path_verbs;
    let _: fn(SpatialPaintFrameV2<'a>) -> &'a [SpatialPathV2] = SpatialPaintFrameV2::paths;
    let _: fn(SpatialPaintFrameV2<'a>) -> &'a [SpatialShapeV2] = SpatialPaintFrameV2::shapes;
    let _: fn(SpatialPaintFrameV2<'a>) -> &'a [SpatialClipV2] =
        SpatialPaintFrameV2::clip_primitives;
    let _: fn(SpatialPaintFrameV2<'a>) -> &'a [SpatialGradientStopV2] =
        SpatialPaintFrameV2::gradient_stops;
    let _: fn(SpatialPaintFrameV2<'a>) -> &'a [SpatialBrushV2] = SpatialPaintFrameV2::brushes;
    let _: fn(SpatialPaintFrameV2<'a>) -> &'a [SpatialImageV2] = SpatialPaintFrameV2::images;
    let _: fn(SpatialPaintFrameV2<'a>) -> &'a [SpatialPaintV2] = SpatialPaintFrameV2::paint_items;
    let _: fn(SpatialPaintFrameV2<'a>) -> &'a [SpatialClipOutputRecordV2] =
        SpatialPaintFrameV2::resolved_clips;
    let _: fn(SpatialPaintFrameV2<'a>) -> &'a [SpatialAabbV2] =
        SpatialPaintFrameV2::effective_clip_aabbs;
    let _: fn(SpatialPaintFrameV2<'a>) -> &'a [SpatialPaintOutputRecordV2] =
        SpatialPaintFrameV2::resolved_paints;
    let _: fn(
        SpatialPaintFrameV2<'a>,
        ReferenceRasterLimitsV2,
    ) -> Result<ReferenceRasterV2, ReferenceRasterErrorV2> =
        SpatialPaintFrameV2::rasterize_reference;
}
