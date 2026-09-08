use std::cell::RefCell;

use crate::{Error, Raster};

#[cfg(test)]
mod tests;

#[derive(Default)]
pub(super) struct RasterCache(RefCell<Option<(u64, Raster)>>);

impl RasterCache {
    pub(super) fn get_or_render(
        &self,
        generation: u64,
        render: impl FnOnce() -> Result<Raster, Error>,
    ) -> Result<Raster, Error> {
        if let Some((cached_generation, raster)) = self.0.borrow().as_ref()
            && *cached_generation == generation
        {
            return Ok(raster.clone());
        }
        // An obsolete frame must not survive a failed render of the new state.
        self.0.borrow_mut().take();
        let raster = render()?;
        *self.0.borrow_mut() = Some((generation, raster.clone()));
        Ok(raster)
    }

    pub(super) fn clear(&mut self) {
        self.0.get_mut().take();
    }
}
