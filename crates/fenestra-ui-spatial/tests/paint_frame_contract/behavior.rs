use fenestra_ui_spatial::prototype::{
    ReferenceRasterErrorKindV2, ReferenceRasterLimitKindV2, ReferenceRasterLimitsV2,
    SpatialPaintContentV2, SpatialPaintOutputReferenceV2, SpatialShapeGeometryV2,
};

use super::fixture::{ExpectedTables, resolved};

#[test]
fn paint_frame_borrows_the_exact_retained_source_and_resolved_slices() {
    let (snapshot, expected) = resolved();
    let frame = snapshot.paint_frame();

    assert_eq!(frame.viewport(), snapshot.viewport());
    assert_source_identity(frame, &expected);
    assert_eq!(
        frame.images()[0].bytes().as_ptr(),
        expected.image_bytes.pointer
    );

    let output = snapshot.output();
    assert_identity(frame.resolved_clips(), output.clips());
    assert_identity(
        frame.effective_clip_aabbs(),
        snapshot.effective_clip_aabbs(),
    );
    assert_identity(frame.resolved_paints(), output.paints());

    let again = snapshot.paint_frame();
    assert_identity(frame.resolved_clips(), again.resolved_clips());
    assert_identity(frame.resolved_paints(), again.resolved_paints());
    assert_identity(frame.images(), again.images());
}

#[test]
fn paint_frame_pairs_dense_clip_and_paint_rows_without_exposing_other_projections() {
    let (snapshot, _) = resolved();
    let frame = snapshot.paint_frame();

    assert_eq!(frame.clip_primitives().len(), frame.resolved_clips().len());
    assert_eq!(
        frame.resolved_clips().len(),
        frame.effective_clip_aabbs().len()
    );
    for (index, (primitive, resolved)) in frame
        .clip_primitives()
        .iter()
        .zip(frame.resolved_clips())
        .enumerate()
    {
        assert_eq!(primitive.key().get(), index as u32);
        assert_eq!(resolved.key(), primitive.key());
        assert_eq!(resolved.owner(), primitive.owner());
        assert_eq!(resolved.parent(), primitive.parent());
        assert_eq!(resolved.shape(), primitive.shape());
    }

    assert_eq!(frame.paint_items().len(), frame.resolved_paints().len());
    for (index, (item, resolved)) in frame
        .paint_items()
        .iter()
        .zip(frame.resolved_paints())
        .enumerate()
    {
        assert_eq!(resolved.key(), index as u32);
        assert_eq!(resolved.owner(), item.owner());
        assert_eq!(resolved.item_ordinal(), item.item_ordinal());
        assert_eq!(resolved.stack_ordinal(), item.owner().get());
        match (item.content(), resolved.reference()) {
            (
                SpatialPaintContentV2::CoveragePaint {
                    coverage, brush, ..
                },
                SpatialPaintOutputReferenceV2::Coverage {
                    shape,
                    brush: output_brush,
                },
            ) => {
                let input_shape = match coverage {
                    fenestra_ui_spatial::prototype::SpatialCoverageV2::Fill { shape, .. }
                    | fenestra_ui_spatial::prototype::SpatialCoverageV2::RoundStroke {
                        shape,
                        ..
                    } => shape,
                };
                assert_eq!(input_shape, shape);
                assert_eq!(brush, output_brush);
            }
            _ => panic!("paint row must preserve its registered reference kind"),
        }
    }

    assert!(matches!(
        frame.shapes()[0].geometry(),
        SpatialShapeGeometryV2::Rect { .. }
    ));
    assert_eq!(snapshot.output().hits().len(), 0);
    assert_eq!(snapshot.output().semantics().len(), 0);
}

#[test]
fn paint_frame_raster_delegates_exact_success_and_error_results() {
    let (snapshot, _) = resolved();
    let frame = snapshot.paint_frame();
    let limits = ReferenceRasterLimitsV2::new(2);
    let from_snapshot = snapshot
        .rasterize_reference(limits)
        .expect("snapshot raster succeeds");
    let from_frame = frame
        .rasterize_reference(limits)
        .expect("frame raster succeeds");

    assert_eq!(from_frame.width(), from_snapshot.width());
    assert_eq!(from_frame.height(), from_snapshot.height());
    assert_eq!(from_frame.stride(), from_snapshot.stride());
    assert_eq!(from_frame.bytes(), from_snapshot.bytes());
    assert_eq!(from_frame.bytes(), &[128, 64, 0, 255, 128, 64, 0, 255]);

    let snapshot_error = match snapshot.rasterize_reference(ReferenceRasterLimitsV2::new(1)) {
        Ok(_) => panic!("snapshot limit must fail"),
        Err(error) => error,
    };
    let frame_error = match frame.rasterize_reference(ReferenceRasterLimitsV2::new(1)) {
        Ok(_) => panic!("frame limit must fail"),
        Err(error) => error,
    };
    assert_eq!(frame_error, snapshot_error);
    assert_eq!(
        frame_error.kind(),
        ReferenceRasterErrorKindV2::LimitExceeded(ReferenceRasterLimitKindV2::Pixels)
    );
    assert_eq!(frame_error.observed(), Some(2));
    assert_eq!(frame_error.maximum(), Some(1));
}

fn assert_source_identity(frame: crate::SpatialPaintFrameV2<'_>, expected: &ExpectedTables) {
    expected.polygon_points.assert(frame.polygon_points());
    expected.path_verbs.assert(frame.path_verbs());
    expected.paths.assert(frame.paths());
    expected.shapes.assert(frame.shapes());
    expected.clips.assert(frame.clip_primitives());
    expected.gradient_stops.assert(frame.gradient_stops());
    expected.brushes.assert(frame.brushes());
    expected.images.assert(frame.images());
    expected.paints.assert(frame.paint_items());
}

fn assert_identity<T>(left: &[T], right: &[T]) {
    assert_eq!(left.as_ptr(), right.as_ptr());
    assert_eq!(left.len(), right.len());
}
