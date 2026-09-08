use crate::{Error, Raster};

#[cfg(test)]
mod tests;

#[derive(Default)]
pub(super) struct RasterCache;

impl RasterCache {
    pub(super) fn get_or_render(
        &self,
        _generation: u64,
        render: impl FnOnce() -> Result<Raster, Error>,
    ) -> Result<Raster, Error> {
        render()
    }

    pub(super) fn clear(&mut self) {}
}
