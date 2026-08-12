use crate::aabb::SpatialAabbV2;
use crate::image::{SpatialImageDestinationRectV2, SpatialImageSourceRectV2};

use super::image_model::ValidatedImageP4;

#[derive(Clone)]
pub(crate) struct PreclipImagePaintP5<'image> {
    paint_index: u32,
    image: ValidatedImageP4<'image>,
    source: SpatialImageSourceRectV2,
    destination: SpatialImageDestinationRectV2,
    opacity: u8,
}

impl<'image> PreclipImagePaintP5<'image> {
    pub(super) const fn new(
        paint_index: u32,
        image: ValidatedImageP4<'image>,
        source: SpatialImageSourceRectV2,
        destination: SpatialImageDestinationRectV2,
        opacity: u8,
    ) -> Self {
        Self {
            paint_index,
            image,
            source,
            destination,
            opacity,
        }
    }

    pub(super) const fn paint_index(&self) -> u32 {
        self.paint_index
    }

    pub(super) const fn destination(&self) -> SpatialImageDestinationRectV2 {
        self.destination
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        u32,
        ValidatedImageP4<'image>,
        SpatialImageSourceRectV2,
        SpatialImageDestinationRectV2,
        u8,
    ) {
        (
            self.paint_index,
            self.image,
            self.source,
            self.destination,
            self.opacity,
        )
    }

    #[cfg(test)]
    pub(crate) fn facts(
        &self,
    ) -> (
        SpatialImageSourceRectV2,
        SpatialImageDestinationRectV2,
        u8,
        &[u8],
    ) {
        (
            self.source,
            self.destination,
            self.opacity,
            self.image.bytes(),
        )
    }
}

pub(crate) struct ValidatedImagePaintP5<'image> {
    preclip: PreclipImagePaintP5<'image>,
    local_bounds: SpatialAabbV2,
}

impl<'image> ValidatedImagePaintP5<'image> {
    pub(super) const fn new(
        preclip: PreclipImagePaintP5<'image>,
        local_bounds: SpatialAabbV2,
    ) -> Self {
        Self {
            preclip,
            local_bounds,
        }
    }

    pub(crate) const fn source(&self) -> SpatialImageSourceRectV2 {
        self.preclip.source
    }

    pub(crate) const fn destination(&self) -> SpatialImageDestinationRectV2 {
        self.preclip.destination
    }

    pub(crate) const fn opacity(&self) -> u8 {
        self.preclip.opacity
    }

    pub(super) const fn image_width(&self) -> u32 {
        self.preclip.image.width()
    }

    pub(super) const fn image_height(&self) -> u32 {
        self.preclip.image.height()
    }

    pub(super) const fn image_stride(&self) -> u32 {
        self.preclip.image.stride()
    }

    pub(crate) fn image_bytes(&self) -> &[u8] {
        self.preclip.image.bytes()
    }

    pub(crate) const fn local_bounds(&self) -> SpatialAabbV2 {
        self.local_bounds
    }

    pub(crate) fn into_parts(self) -> (PreclipImagePaintP5<'image>, SpatialAabbV2) {
        (self.preclip, self.local_bounds)
    }
}
