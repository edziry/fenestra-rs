use fenestra_ui::{Raster, Size, TextRect};

use super::GalleryError;

pub(super) struct Canvas {
    size: Size,
    pixels: Vec<u8>,
}

impl Canvas {
    pub(super) fn new(size: Size, background: [u8; 4]) -> Result<Self, GalleryError> {
        let count = usize::try_from(u64::from(size.width()) * u64::from(size.height()))?;
        if count > 4_194_304 {
            return Err("gallery pixel limit exceeded".into());
        }
        Ok(Self {
            size,
            pixels: background.repeat(count),
        })
    }

    pub(super) fn fill(&mut self, rect: TextRect, color: [u8; 4]) {
        let x0 = rect.x0().floor().clamp(0.0, f64::from(self.size.width())) as u32;
        let y0 = rect.y0().floor().clamp(0.0, f64::from(self.size.height())) as u32;
        let x1 = rect.x1().ceil().clamp(0.0, f64::from(self.size.width())) as u32;
        let y1 = rect.y1().ceil().clamp(0.0, f64::from(self.size.height())) as u32;
        for y in y0..y1 {
            for x in x0..x1 {
                self.blend(x, y, color);
            }
        }
    }

    pub(super) fn layer(&mut self, raster: &Raster, x: u32, y: u32) {
        let width = raster
            .size()
            .width()
            .min(self.size.width().saturating_sub(x));
        let height = raster
            .size()
            .height()
            .min(self.size.height().saturating_sub(y));
        for row in 0..height {
            for column in 0..width {
                let index = (row as usize * raster.size().width() as usize + column as usize) * 4;
                self.blend(
                    x + column,
                    y + row,
                    raster.bytes()[index..index + 4].try_into().unwrap(),
                );
            }
        }
    }

    fn blend(&mut self, x: u32, y: u32, source: [u8; 4]) {
        let index = (y as usize * self.size.width() as usize + x as usize) * 4;
        let destination = &mut self.pixels[index..index + 4];
        for channel in 0..4 {
            let behind = u16::from(destination[channel]) * u16::from(255 - source[3]);
            destination[channel] = source[channel] + ((behind + 127) / 255) as u8;
        }
    }

    pub(super) fn finish(self) -> Result<Raster, GalleryError> {
        Ok(Raster::new(self.size, self.pixels)?)
    }
}
