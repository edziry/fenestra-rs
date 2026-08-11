use crate::image::SpatialImageV2;

#[derive(Clone, Copy)]
pub(crate) struct ValidatedImageP4<'a> {
    image: &'a SpatialImageV2,
}

impl<'a> ValidatedImageP4<'a> {
    pub(super) const fn new(image: &'a SpatialImageV2) -> Self {
        Self { image }
    }

    pub(super) const fn width(&self) -> u32 {
        self.image.width()
    }

    pub(super) const fn height(&self) -> u32 {
        self.image.height()
    }

    pub(super) const fn stride(&self) -> u32 {
        self.image.stride()
    }

    pub(super) fn bytes(&self) -> &[u8] {
        self.image.bytes()
    }
}
