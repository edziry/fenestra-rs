use crate::brush::SpatialRgba8V2;
use crate::model::SpatialPointV2;

use super::apply_opacity_p1;
use super::image_paint_model::ValidatedImagePaintP5;

pub(super) fn sample_image_p6(
    paint: &ValidatedImagePaintP5<'_, '_>,
    point: SpatialPointV2,
) -> Option<SpatialRgba8V2> {
    let bounds = paint.local_bounds();
    let point_x = point.x().raw();
    let point_y = point.y().raw();
    if point_x < bounds.min_x().raw()
        || point_x >= bounds.max_x().raw()
        || point_y < bounds.min_y().raw()
        || point_y >= bounds.max_y().raw()
    {
        return None;
    }

    let source = paint.source();
    let destination = paint.destination();
    let source_x = mapped_source_coordinate(
        point_x,
        destination.x().raw(),
        destination.width().raw(),
        source.x(),
        source.width(),
    );
    let source_y = mapped_source_coordinate(
        point_y,
        destination.y().raw(),
        destination.height().raw(),
        source.y(),
        source.height(),
    );
    let byte_ordinal =
        u128::from(source_y) * u128::from(paint.image_stride()) + u128::from(source_x) * 4;
    let byte_ordinal = usize::try_from(byte_ordinal)
        .expect("a P4 image proof keeps every sampled byte ordinal representable");
    let bytes = paint.image_bytes();
    let premultiplied = SpatialRgba8V2::new(
        bytes[byte_ordinal],
        bytes[byte_ordinal + 1],
        bytes[byte_ordinal + 2],
        bytes[byte_ordinal + 3],
    );
    Some(apply_opacity_p1(premultiplied, paint.opacity()))
}

fn mapped_source_coordinate(
    point: i64,
    destination_near: i64,
    destination_extent: i64,
    source_near: u32,
    source_extent: u32,
) -> u32 {
    let destination_offset = i128::from(point) - i128::from(destination_near);
    let source_offset =
        destination_offset * i128::from(source_extent) / i128::from(destination_extent);
    u32::try_from(i128::from(source_near) + source_offset)
        .expect("a P5 image paint proof keeps sampled source coordinates in bounds")
}
