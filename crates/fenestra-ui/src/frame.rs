use crate::Error;

/// Physical pixel size of an application viewport or raster.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Size {
    width: u32,
    height: u32,
}

impl Size {
    /// Describes a viewport; application creation validates its bounds.
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Returns the pixel width.
    #[must_use]
    pub const fn width(self) -> u32 {
        self.width
    }

    /// Returns the pixel height.
    #[must_use]
    pub const fn height(self) -> u32 {
        self.height
    }

    pub(crate) fn pixel_count(self) -> Result<usize, Error> {
        if self.width == 0
            || self.height == 0
            || self.width > i32::MAX as u32
            || self.height > i32::MAX as u32
        {
            return Err(Error::InvalidViewport {
                width: self.width,
                height: self.height,
            });
        }
        (self.width as usize)
            .checked_mul(self.height as usize)
            .ok_or(Error::CapacityOverflow)
    }
}

/// Owned premultiplied RGBA8 pixels in row-major order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Raster {
    size: Size,
    bytes: Vec<u8>,
}

impl Raster {
    /// Creates a raster after checking its pixel and byte counts.
    pub fn new(size: Size, bytes: Vec<u8>) -> Result<Self, Error> {
        let length = size
            .pixel_count()?
            .checked_mul(4)
            .ok_or(Error::CapacityOverflow)?;
        if bytes.len() != length {
            return Err(Error::InvalidRaster);
        }
        Ok(Self { size, bytes })
    }

    /// Returns the size represented by these pixels.
    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    /// Borrows the premultiplied RGBA8 pixel data.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
