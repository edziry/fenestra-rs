use super::super::super::model::{
    PreparedBrushContent, PreparedCoverage, PreparedPaintContent, PreparedShapeGeometry,
    PreparedSpatialState,
};
use super::SpatialResolvedSnapshotV2;
use crate::model::{Affine2V2, SpatialScalarV2};
use crate::output_aabb::SpatialOutputAabbV2;

/// Every coverage edge and image texel boundary lies on the integer pixel grid.
/// A solid rectangle, one-to-one image, or full single-texel image supplies the
/// same color at all sixteen registered samples. The single texel maps to every
/// point within its half-open destination, including when enlarged. Integer clip
/// boundaries, accepted bounds, and translations preserve that property through
/// ordered source-over and scalar-domain rejection. Averaging is then exact.
pub(super) fn pixel_samples_are_constant(snapshot: &SpatialResolvedSnapshotV2) -> bool {
    let state = &snapshot.prepared.state;
    snapshot
        .paints
        .iter()
        .zip(state.paints.iter())
        .all(|(row, paint)| {
            integer_translation(row.world_from_local())
                && integer_output_bounds(row.world_aabb())
                && match &paint.content {
                    PreparedPaintContent::Coverage {
                        coverage: PreparedCoverage::Fill { shape, .. },
                        brush,
                        ..
                    } => {
                        integer_rectangle(state, *shape)
                            && matches!(
                                state.brushes[*brush as usize].content,
                                PreparedBrushContent::Solid(_)
                            )
                    }
                    PreparedPaintContent::Image {
                        image,
                        source,
                        destination,
                        ..
                    } => {
                        let plan = &state.images[*image as usize];
                        let single_texel = plan.width == 1
                            && plan.height == 1
                            && source.x() == 0
                            && source.y() == 0
                            && source.width() == 1
                            && source.height() == 1;
                        let one_to_one = destination.width().raw()
                            == i64::from(source.width()) * SpatialScalarV2::SCALE
                            && destination.height().raw()
                                == i64::from(source.height()) * SpatialScalarV2::SCALE;
                        [
                            destination.x(),
                            destination.y(),
                            destination.width(),
                            destination.height(),
                        ]
                        .into_iter()
                        .all(integer)
                            && (one_to_one || single_texel)
                    }
                    PreparedPaintContent::Coverage { .. } => false,
                }
        })
        && snapshot
            .clips
            .iter()
            .zip(state.clips.iter())
            .all(|(row, clip)| {
                integer_translation(row.world_from_local())
                    && integer_output_bounds(row.primitive_world_aabb())
                    && integer_rectangle(state, clip.shape)
            })
        && state.effective_clip_aabbs.iter().all(|bounds| {
            [
                bounds.min_x(),
                bounds.min_y(),
                bounds.max_x(),
                bounds.max_y(),
            ]
            .into_iter()
            .all(integer)
        })
}

fn integer_rectangle(state: &PreparedSpatialState, shape: u32) -> bool {
    match state.shapes[shape as usize].geometry {
        PreparedShapeGeometry::Rect { rect } => [
            rect.origin().x(),
            rect.origin().y(),
            rect.width(),
            rect.height(),
        ]
        .into_iter()
        .all(integer),
        _ => false,
    }
}

fn integer_translation(world: Affine2V2) -> bool {
    world.a().raw() == SpatialScalarV2::SCALE
        && world.b().raw() == 0
        && world.c().raw() == 0
        && world.d().raw() == SpatialScalarV2::SCALE
        && integer(world.tx())
        && integer(world.ty())
}

fn integer_output_bounds(bounds: SpatialOutputAabbV2) -> bool {
    [
        bounds.min_x(),
        bounds.min_y(),
        bounds.max_x(),
        bounds.max_y(),
    ]
    .into_iter()
    .all(integer)
}

fn integer(value: SpatialScalarV2) -> bool {
    value.raw() % SpatialScalarV2::SCALE == 0
}
