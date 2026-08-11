use crate::aabb::SpatialAabbV2;
use crate::image::{SpatialImageDestinationRectV2, SpatialImageSourceRectV2};
use crate::vocabulary::{SpatialAxisV2, SpatialExtentV2};

use super::image_model::ValidatedImageP4;
use super::image_paint_error::{PaintP5Error, PaintP5ErrorKind, PaintP5Field, PaintP5ImageKind};
use super::image_paint_model::{PreclipImagePaintP5, ValidatedImagePaintP5};

pub(super) fn prepare_image_paint_p5<'image>(
    paint_index: u32,
    image: &ValidatedImageP4<'image>,
    source: SpatialImageSourceRectV2,
    destination: SpatialImageDestinationRectV2,
    opacity: u8,
) -> Result<PreclipImagePaintP5<'image>, PaintP5Error> {
    if source.width() == 0 {
        return Err(image_error(
            paint_index,
            PaintP5ImageKind::EmptySource,
            PaintP5Field::SourceWidth,
        ));
    }
    if source.height() == 0 {
        return Err(image_error(
            paint_index,
            PaintP5ImageKind::EmptySource,
            PaintP5Field::SourceHeight,
        ));
    }

    validate_source_axis(
        paint_index,
        source.x(),
        source.width(),
        image.width(),
        PaintP5Field::SourceX,
        PaintP5Field::SourceWidth,
    )?;
    validate_source_axis(
        paint_index,
        source.y(),
        source.height(),
        image.height(),
        PaintP5Field::SourceY,
        PaintP5Field::SourceHeight,
    )?;

    for (scalar, field) in [
        (destination.x(), PaintP5Field::DestinationX),
        (destination.y(), PaintP5Field::DestinationY),
        (destination.width(), PaintP5Field::DestinationWidth),
        (destination.height(), PaintP5Field::DestinationHeight),
    ] {
        if !scalar.is_in_domain() {
            return Err(PaintP5Error::new(
                PaintP5ErrorKind::ScalarOutOfDomain,
                paint_index,
                field,
            ));
        }
    }

    if destination.width().raw() < 0 {
        return Err(image_error(
            paint_index,
            PaintP5ImageKind::NegativeDestinationExtent(SpatialExtentV2::Width),
            PaintP5Field::DestinationWidth,
        ));
    }
    if destination.height().raw() < 0 {
        return Err(image_error(
            paint_index,
            PaintP5ImageKind::NegativeDestinationExtent(SpatialExtentV2::Height),
            PaintP5Field::DestinationHeight,
        ));
    }
    if destination.width().raw() == 0 {
        return Err(image_error(
            paint_index,
            PaintP5ImageKind::EmptyDestination,
            PaintP5Field::DestinationWidth,
        ));
    }
    if destination.height().raw() == 0 {
        return Err(image_error(
            paint_index,
            PaintP5ImageKind::EmptyDestination,
            PaintP5Field::DestinationHeight,
        ));
    }

    Ok(PreclipImagePaintP5::new(
        paint_index,
        *image,
        source,
        destination,
        opacity,
    ))
}

pub(super) fn finish_image_paint_bounds_after_item_phase_p5(
    preclip: PreclipImagePaintP5<'_>,
) -> Result<ValidatedImagePaintP5<'_>, PaintP5Error> {
    let destination = preclip.destination();
    let max_x = destination
        .x()
        .checked_add(destination.width())
        .ok_or_else(|| {
            bounds_error(
                preclip.paint_index(),
                SpatialAxisV2::X,
                PaintP5Field::DestinationWidth,
            )
        })?;
    let max_y = destination
        .y()
        .checked_add(destination.height())
        .ok_or_else(|| {
            bounds_error(
                preclip.paint_index(),
                SpatialAxisV2::Y,
                PaintP5Field::DestinationHeight,
            )
        })?;
    let local_bounds =
        match SpatialAabbV2::from_edges(destination.x(), destination.y(), max_x, max_y) {
            Some(bounds) => bounds,
            None => unreachable!("P5 proof keeps canonical positive destination edges"),
        };
    Ok(ValidatedImagePaintP5::new(preclip, local_bounds))
}

fn validate_source_axis(
    paint_index: u32,
    near: u32,
    extent: u32,
    image_extent: u32,
    near_field: PaintP5Field,
    extent_field: PaintP5Field,
) -> Result<(), PaintP5Error> {
    if near >= image_extent {
        return Err(image_error(
            paint_index,
            PaintP5ImageKind::SourceOutOfBounds,
            near_field,
        ));
    }
    if u64::from(near) + u64::from(extent) > u64::from(image_extent) {
        return Err(image_error(
            paint_index,
            PaintP5ImageKind::SourceOutOfBounds,
            extent_field,
        ));
    }
    Ok(())
}

fn image_error(paint_index: u32, kind: PaintP5ImageKind, field: PaintP5Field) -> PaintP5Error {
    PaintP5Error::new(PaintP5ErrorKind::InvalidImage(kind), paint_index, field)
}

fn bounds_error(paint_index: u32, axis: SpatialAxisV2, field: PaintP5Field) -> PaintP5Error {
    PaintP5Error::new(
        PaintP5ErrorKind::LocalBoundsOutOfDomain(axis),
        paint_index,
        field,
    )
}
