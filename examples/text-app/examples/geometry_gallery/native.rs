use std::io;

use fenestra_ui::native::{self, ImeContext, WindowContent, WindowEvent, WindowOptions};
use fenestra_ui::{Raster, Size};

use super::{
    GalleryError,
    scene::{self, Gallery},
};

pub(super) fn present(gallery: Gallery) -> Result<Gallery, GalleryError> {
    let mut content = Preview {
        gallery,
        presented: false,
    };
    native::run(
        &mut content,
        WindowOptions::new("Fenestra text geometry")
            .size(900, 540)
            .smoke(true),
    )?;
    if !content.presented {
        return Err("native geometry preview did not present".into());
    }
    Ok(content.gallery)
}

struct Preview {
    gallery: Gallery,
    presented: bool,
}

impl WindowContent for Preview {
    type Error = io::Error;

    fn resize(&mut self, width: u32, height: u32) -> io::Result<()> {
        self.gallery = scene::render(Size::new(width, height)).map_err(io::Error::other)?;
        Ok(())
    }

    fn event(&mut self, _event: WindowEvent) -> io::Result<()> {
        Ok(())
    }

    fn frame(&self) -> io::Result<Raster> {
        Ok(self.gallery.raster.clone())
    }

    fn ime_context(&self) -> io::Result<Option<ImeContext>> {
        let (x, y, width, height) = self.gallery.ime_caret;
        ImeContext::active(1, x, y, width, height)
            .map(Some)
            .map_err(io::Error::other)
    }

    fn presented(&mut self) -> io::Result<()> {
        self.presented = true;
        Ok(())
    }
}
